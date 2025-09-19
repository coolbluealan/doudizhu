use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::card::{self, Hand};
use crate::game::Game;
use crate::{AppError, User};

#[derive(Debug, Deserialize)]
pub enum ClientMsg {
    Chat(String),
    Start,
    Bid(usize),
    Play(Vec<usize>),
}

#[derive(Clone, Debug, Serialize)]
pub struct Msg {
    text: String,
    idx: usize, // player idx or 9 for game messages
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
            game: Default::default(),
            chat: Vec::new(),
            tx,
        }
    }

    pub fn user_idx(&self, user: &User) -> Option<usize> {
        self.users.get(&user.id).copied()
    }

    fn current_name(&self) -> &str {
        &self.players[self.game.turn()].name
    }

    pub fn join(&mut self, user: &User) -> Result<(), AppError> {
        // check whether possible to join
        if self.users.contains_key(&user.id) {
            return Err("already joined the lobby".into());
        }
        let idx = self.players.len();
        if idx >= 4 {
            return Err("lobby is full".into());
        }

        // add the user
        self.users.insert(user.id, idx);
        self.players.push(Player {
            name: user.username.clone(),
            score: 0,
        });
        self.send_msg(9, format!("{} joined the game.", user.username));

        self.send_state();
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), AppError> {
        match self.status {
            Status::Lobby => {
                if self.players.len() < 3 {
                    return Err("not enough players".into());
                }
            }
            Status::Bidding | Status::Playing => return Err("game in progress".into()),
            _ => {}
        }

        self.game = Game::new(self.players.len());
        self.status = Status::Bidding;
        self.send_msg(
            9,
            format!("Game started. {} begins the bidding.", self.current_name()),
        );
        self.send_state();
        Ok(())
    }

    pub fn bid(&mut self, idx: usize, val: usize) -> Result<(), AppError> {
        if self.game.bid(idx, val).map_err(AppError)? {
            self.game = Game::new(self.players.len());
            self.send_msg(
                9,
                format!(
                    "No one bid. Redealing cards. {} begins the bidding.",
                    self.current_name(),
                ),
            );
        } else {
            self.send_msg(
                9,
                format!(
                    "{} {}.",
                    self.players[idx].name,
                    match val {
                        0 => "passed".to_string(),
                        _ => format!("bid {}", val),
                    },
                ),
            );
            if self.game.playing() {
                self.status = Status::Playing;
                self.send_msg(
                    9,
                    format!(
                        "{} is the landlord! Bonus cards: {}",
                        self.players[self.game.landlord()].name,
                        self.game.landlord_bonus(),
                    ),
                );
            }
        }
        self.send_state();
        Ok(())
    }

    pub fn play(&mut self, idx: usize, cards: Vec<usize>) -> Result<(), AppError> {
        let hand = Hand::new(self.players.len(), cards).map_err(AppError)?;
        let action = if hand.is_pass() {
            "passed".to_string()
        } else {
            format!("played {}", card::join(hand.cards()))
        };
        self.game.play(idx, hand).map_err(AppError)?;
        self.send_msg(9, format!("{} {}.", self.players[idx].name, action));

        if let Some(winner) = self.game.winner() {
            self.send_msg(
                9,
                format!("{} played all their cards!", self.players[winner].name),
            );

            let landlord = self.game.landlord();
            let mask = self.game.played_mask();
            let mut delta = self.game.score_delta() as i32;
            let mut landlord_delta = delta * self.players.len() as i32;
            if mask < 6 {
                delta *= 2;
                self.send_msg(
                    9,
                    format!(
                        "{} DOMINATION! Score doubles.",
                        if mask & 4 != 0 { "LANDLORD" } else { "PEASANT" }
                    ),
                );
            }

            let msg;
            if winner == landlord {
                msg = format!(
                    "The landlord wins +{}. Peasants lose -{}.",
                    landlord_delta, delta
                );
            } else {
                msg = format!(
                    "Peasants win +{}. The landlord loses -{}.",
                    delta, landlord_delta
                );
                delta *= -1;
                landlord_delta *= -1;
            }
            self.send_msg(9, msg);

            self.players.iter_mut().for_each(|x| x.score -= delta);
            self.players[landlord].score += landlord_delta;

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
        Value::from(lobby)
    }

    pub fn serialize_idx(&self, mut state: Value, idx: usize) -> Value {
        state["idx"] = Value::from(idx);
        if self.status != Status::Lobby {
            state["hand"] = self.game.serialize_cards(idx);
        }
        state
    }

    pub fn send_state(&self) {
        let _ = self.tx.send(ServerMsg::State(self.serialize()));
    }
}
