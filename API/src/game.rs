use crate::card::{self, Card, Hand};
use serde_json::Value;

#[derive(Default)]
pub struct Game {
    players: usize,
    cards: Vec<Vec<Card>>, // last element contains hidden cards
    turn: usize,
    bid: usize,
    mult: usize,
    landlord: usize,
    last_idx: usize,
    last_play: Hand,
    passes: usize,
    winner: Option<usize>,
    played_mask: usize, // 3 bits representing landlord play, peasants play, landlord play 2
}

impl Game {
    pub fn new(players: usize) -> Self {
        Self {
            players,
            cards: Hand::deal_hands(players),
            turn: rand::random_range(..players),
            bid: 0,
            mult: 1,
            landlord: players,
            last_play: Hand::PASS,
            last_idx: 0,
            passes: 0,
            winner: None,
            played_mask: 0,
        }
    }

    // getter functions
    pub fn landlord(&self) -> usize {
        self.landlord
    }
    pub fn playing(&self) -> bool {
        self.landlord != self.players
    }
    pub fn score_delta(&self) -> usize {
        self.mult * self.bid
    }
    pub fn turn(&self) -> usize {
        self.turn
    }
    pub fn winner(&self) -> Option<usize> {
        self.winner
    }
    pub fn played_mask(&self) -> usize {
        self.played_mask
    }

    // Ok(true) means redeal
    pub fn bid(&mut self, idx: usize, val: usize) -> Result<bool, String> {
        // check phase
        if self.playing() {
            return Err("bidding is over".to_string());
        }
        // check turn
        if self.turn != idx {
            return Err("not your turn".to_string());
        }

        // bid of 0 denotes pass
        if val == 0 {
            self.passes += 1;
        } else {
            if val <= self.bid {
                return Err("must bid higher than previous bid".to_string());
            }
            self.passes = 0;
            self.bid = val;
            self.last_idx = idx;
        }

        // end bidding phase
        if (self.passes == self.players - 1 && self.bid > 0) || self.bid == 3 {
            self.passes = 0;
            self.landlord = self.last_idx;
            self.turn = self.landlord;

            // landlord receives new cards
            let hidden = self.cards[self.players].clone();
            self.cards[self.landlord].extend(hidden);
            self.cards[self.landlord].sort();
        } else if self.passes == self.players {
            // no one bid, so redeal
            return Ok(true);
        } else {
            self.turn = (idx + 1) % self.players;
        }
        Ok(false)
    }

    pub fn play(&mut self, idx: usize, hand: Hand) -> Result<(), String> {
        // check phase
        if !self.playing() {
            return Err("still bidding".to_string());
        }
        if self.winner.is_some() {
            return Err("game finished".to_string());
        }
        // check turn
        if self.turn != idx {
            return Err("not your turn".to_string());
        }

        // check hand exists in cards
        let mut i = 0;
        let mut j = 0;
        while j < hand.cards().len() {
            while i < self.cards[idx].len() && self.cards[idx][i] < hand.cards()[j] {
                i += 1;
            }
            if i == self.cards[idx].len() || self.cards[idx][i] != hand.cards()[j] {
                return Err("cards not in hand".to_string());
            }

            i += 1;
            j += 1;
        }

        // try to play hand
        if hand.is_pass() {
            if self.last_play.is_pass() || self.passes == self.players - 1 {
                return Err("cannot pass".to_string());
            }
            self.passes += 1;
            if self.passes == self.players - 1 {
                self.passes = 0;
                self.last_play = Hand::PASS;
            }
        } else {
            hand.can_play(&self.last_play)?;
            // remove cards
            for card in hand.cards() {
                let pos = self.cards[idx]
                    .iter()
                    .position(|c| c == card)
                    .expect("card check done earlier");
                self.cards[idx].remove(pos);
            }

            // score keeping
            if self.cards[idx].is_empty() {
                self.winner = Some(idx);
            }
            if hand.is_double(self.players) {
                self.mult *= 2;
            }
            self.played_mask |= if idx == self.landlord {
                if self.played_mask & 1 != 0 {
                    4
                } else {
                    1
                }
            } else {
                2
            };

            // update game state
            self.passes = 0;
            self.last_idx = idx;
            self.last_play = hand;
        }

        self.turn = (idx + 1) % self.players;
        Ok(())
    }

    pub fn serialize_cards(&self, idx: usize) -> Value {
        if idx == self.players {
            Value::from(card::join(&self.cards[self.players]))
        } else {
            serde_json::to_value(&self.cards[idx]).unwrap()
        }
    }

    pub fn serialize(&self) -> Value {
        let mut game = serde_json::Map::new();

        game.insert("turn".to_string(), Value::from(self.turn));
        game.insert("bid".to_string(), Value::from(self.bid));
        game.insert("mult".to_string(), Value::from(self.mult));
        game.insert("passes".to_string(), Value::from(self.passes));
        game.insert(
            "cards_left".to_string(),
            Value::from(
                self.cards[..self.players]
                    .iter()
                    .map(|c| c.len())
                    .collect::<Vec<_>>(),
            ),
        );
        game.insert("last_idx".to_string(), Value::from(self.last_idx));

        if self.playing() {
            game.insert(
                "last_play".to_string(),
                serde_json::to_value(&self.last_play).unwrap(),
            );
            game.insert("landlord".to_string(), Value::from(self.landlord));
            game.insert("bonus".to_string(), self.serialize_cards(self.players));
        }

        // finished
        if let Some(winner) = self.winner {
            game.insert("winner".to_string(), Value::from(winner));
        }

        Value::from(game)
    }
}
