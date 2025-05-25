use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::game::{Game, Hand};
use crate::{AppError, User};

#[derive(Debug, Deserialize)]
pub enum ClientMsg {
    Chat(String),
    Move(Hand),
}

#[derive(Clone, Debug, Serialize)]
pub struct Msg {
    text: String,
    idx: usize, // player idx or 3 for game messages
    time: u64,
}

#[derive(Clone, Debug, Serialize)]
pub enum ServerMsg {
    Chat(Msg),
    State(Value),
}

#[derive(Clone, Debug, Serialize)]
struct Player {
    name: String,
    score: i32,
}

#[derive(PartialEq, Serialize)]
enum Status {
    Lobby,
    Bidding,
    Playing,
    Finished,
}

pub struct Lobby {
    status: Status,
    users: HashMap<Uuid, usize>,
    players: Vec<Player>,
    game: Game,
    chat: Vec<Msg>,
    tx: broadcast::Sender<ServerMsg>,
}
impl Lobby {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(50);
        Self {
            status: Status::Lobby,
            users: HashMap::new(),
            players: Vec::new(),
            game: Game::new(),
            chat: Vec::new(),
            tx,
        }
    }

    pub fn user_idx(&self, user: &User) -> Option<usize> {
        self.users.get(&user.id).copied()
    }

    pub fn join(&mut self, user: &User) -> Result<(), AppError> {
        // check whether possible to join
        if self.users.contains_key(&user.id) {
            return Err("Already joined the lobby".into());
        }
        let idx = self.players.len();
        if idx >= 3 {
            return Err("Lobby is full".into());
        }

        // add the user
        self.users.insert(user.id, idx);
        self.players.push(Player {
            name: user.username.clone(),
            score: 0,
        });

        if idx == 2 {
            self.status = Status::Bidding;
        }
        self.send_msg(3, format!("{} joined the game", user.username));
        self.send_state();
        Ok(())
    }

    pub fn bid(&mut self, idx: usize, val: usize) -> Result<(), AppError> {
        self.game.bid(idx, val).map_err(AppError)?;
        if self.game.playing() {
            self.status = Status::Playing;
            self.send_msg(
                3,
                format!(
                    "{} is the landlord!",
                    self.players[self.game.landlord()].name
                ),
            );
        }
        self.send_state();
        Ok(())
    }

    pub fn play(&mut self, idx: usize, hand: Hand) -> Result<(), AppError> {
        self.game.play(idx, hand).map_err(AppError)?;
        if let Some(winner) = self.game.winner() {
            self.send_msg(
                3,
                format!("{} played all their cards!", self.players[winner].name),
            );

            let landlord = self.game.landlord();
            let mut delta: i32 = self.game.score_delta().try_into().unwrap();
            let msg;

            if winner == landlord {
                msg = format!(
                    "The landlord wins +{}. Peasants lose -{}.",
                    2 * delta,
                    delta
                );
            } else {
                msg = format!("Peasants win +{}. The landlord loses {}.", delta, 2 * delta);
                delta *= -1;
            }
            self.send_msg(3, msg);

            self.players[landlord].score += 2 * delta;
            self.players[(landlord + 1) % 3].score -= delta;
            self.players[(landlord + 2) % 3].score -= delta;

            self.status = Status::Finished;
        }
        self.send_state();
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerMsg> {
        self.tx.subscribe()
    }

    pub fn send_msg(&mut self, idx: usize, text: String) {
        let mut time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // make timestamp unique
        if let Some(last) = self.chat.last() {
            if time <= last.time {
                time = last.time + 1;
            }
        }

        let msg = Msg { text, idx, time };
        self.chat.push(msg.clone());
        let _ = self.tx.send(ServerMsg::Chat(msg));
    }

    pub fn chat_before(&self, time: Option<u64>, limit: usize) -> Vec<Msg> {
        // if time is None then take latest
        let pos = match time {
            Some(time) => match self.chat.binary_search_by(|msg| msg.time.cmp(&time)) {
                Ok(i) => i,
                Err(i) => i,
            },
            None => self.chat.len(),
        };

        let start = pos.saturating_sub(limit);
        self.chat[start..pos].to_vec()
    }

    pub fn serialize(&self) -> Value {
        let mut lobby = serde_json::Map::new();
        // status
        lobby.insert(
            "status".to_string(),
            serde_json::to_value(&self.status).unwrap(),
        );
        // player list
        lobby.insert(
            "players".to_string(),
            serde_json::to_value(&self.players).unwrap(),
        );
        // game state
        if self.status != Status::Lobby {
            lobby.insert("game".to_string(), self.game.serialize());
        }
        Value::Object(lobby)
    }

    pub fn serialize_idx(&self, mut state: Value, idx: Option<usize>) -> Value {
        if let Some(idx) = idx {
            if self.status != Status::Lobby {
                state["hand"] = self.game.serialize_cards(idx);
            }
        }
        state["idx"] = Value::from(idx);
        state
    }

    pub fn send_state(&self) {
        let _ = self.tx.send(ServerMsg::State(self.serialize()));
    }
}
