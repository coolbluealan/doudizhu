use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use uuid::Uuid;

use crate::{game::Game, AppError, User};

struct Player {
    idx: u8,
    score: i32,
}

#[derive(Clone, Serialize)]
pub struct Msg {
    idx: u8,
    text: String,
    timestamp: u64,
}

pub struct Lobby {
    users: HashMap<Uuid, Player>,
    game: Game,
    chat: Vec<Msg>,
}
impl Lobby {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            game: Game::new(),
            chat: Vec::new(),
        }
    }

    pub fn join(&mut self, user: &User) -> Result<(), AppError> {
        if self.users.contains_key(&user.id) {
            return Err("Already joined the lobby ".into());
        }
        let idx = self.users.len().try_into().unwrap();
        if idx >= 3 {
            return Err("Lobby is full".into());
        }
        self.users.insert(user.id, Player { idx, score: 0 });
        Ok(())
    }

    pub fn leave(&mut self, user: &User) -> Result<(), AppError> {
        if self.users.remove(&user.id).is_some() {
            Ok(())
        } else {
            Err("User not in lobby".into())
        }
    }

    pub fn add_msg(&mut self, user: &User, text: String) -> Result<Msg, AppError> {
        let player = self.users.get(&user.id).ok_or("User not in lobby")?;
        let mut timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if let Some(last) = self.chat.last() {
            if timestamp < last.timestamp {
                timestamp = last.timestamp + 1;
            }
        }

        let msg = Msg {
            idx: player.idx,
            text,
            timestamp,
        };
        self.chat.push(msg.clone());
        Ok(msg)
    }

    pub fn chat_before(&self, timestamp: u64, limit: usize) -> Vec<Msg> {
        let pos = match self
            .chat
            .binary_search_by(|msg| msg.timestamp.cmp(&timestamp))
        {
            Ok(i) => i,
            Err(i) => i,
        };

        let start = pos.saturating_sub(limit);
        self.chat[start..pos].to_vec()
    }
}
