use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use rand::seq::SliceRandom;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SUIT_MAP: [&str; 4] = ["♣️", "♦️", "♥️", "♠️"];
const RANK_MAP: [&str; 14] = [
    "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A", "2", "J",
];
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Card(usize);
impl Card {
    fn rank(&self) -> usize {
        self.0 / 4
    }
}
impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let suit = match self.0 {
            52 => "🃟",
            53 => "🃏",
            _ => SUIT_MAP[self.0 % 4],
        };
        write!(f, "{}{}", RANK_MAP[self.rank()], suit)
    }
}
pub fn join_cards(cards: &Vec<Card>) -> String {
    cards
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
enum HandName {
    Pass,
    Single,
    Straight,
    Pair,
    Triple,
    TripleSingle,
    TriplePair,
    QuadSingle,
    QuadPair,
    Bomb,
    Rocket,
}
impl Display for HandName {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let hand_name = match self {
            HandName::Pass => "Pass",
            HandName::Single => "Single",
            HandName::Straight => "Straight",
            HandName::Pair => "Pair",
            HandName::Triple => "Triple",
            HandName::TripleSingle => "Triple + Single",
            HandName::TriplePair => "Triple + Pair",
            HandName::QuadSingle => "Quad + Singles",
            HandName::QuadPair => "Quad + Pairs",
            HandName::Bomb => "Bomb",
            HandName::Rocket => "Rocket",
        };
        write!(f, "{}", hand_name)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
struct HandType {
    name: HandName,
    mult: usize,
}
const PASS: HandType = HandType {
    name: HandName::Pass,
    mult: 1,
};
impl Serialize for HandType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let name = self.name.to_string();
        serializer.serialize_str(
            &(if self.mult > 1 && self.name != HandName::Straight {
                name + " Chained"
            } else {
                name
            }),
        )
    }
}

#[derive(Debug, Serialize)]
pub struct Hand {
    kind: HandType,
    #[serde(skip)]
    sort_key: Vec<usize>,
    cards: Vec<Card>,
}
const HAND_PASS: Hand = Hand {
    kind: PASS,
    sort_key: Vec::new(),
    cards: Vec::new(),
};
impl Hand {
    pub fn new(cards: Vec<Card>) -> Result<Self, String> {
        if cards.len() == 0 {
            return Ok(HAND_PASS);
        }

        // a normal user should not see these errors
        if !cards.iter().all(|c| c.0 < 54) {
            return Err("invalid cards".to_string());
        }
        if !cards.is_sorted() {
            return Err("unsorted cards".to_string());
        }

        // count multiplicities in the hand
        let mut cnts: [Vec<usize>; 4] = Default::default();
        let mut count = 1;
        for i in 0..cards.len() - 1 {
            if cards[i].rank() == cards[i + 1].rank() {
                count += 1;
            } else {
                cnts[count - 1].push(cards[i].rank());
                count = 1;
            }
        }
        cnts[count - 1].push(cards[cards.len() - 1].rank());

        let single = cnts[0].len();
        let pair = cnts[1].len();
        let trip = cnts[2].len();
        let quad = cnts[3].len();

        let is_valid_chain = |v: &[usize], sz: usize| {
            v.len() >= sz && v[v.len() - 1] < 12 && v.windows(2).all(|w| w[0] + 1 == w[1])
        };

        let mut hand_type = PASS;
        if quad > 0 {
            if quad == 1 && trip == 0 {
                if single == 0 && pair == 0 {
                    hand_type.name = HandName::Bomb;
                } else if single == 2 && pair == 0 {
                    hand_type.name = HandName::QuadSingle;
                } else if single == 0 && pair == 2 {
                    hand_type.name = HandName::QuadPair;
                }
            }
        } else if trip > 0 {
            hand_type.mult = trip;
            if hand_type.mult == 1 || is_valid_chain(&cnts[2], 2) {
                if single == 0 && pair == 0 {
                    hand_type.name = HandName::Triple;
                } else if trip == single && pair == 0 {
                    hand_type.name = HandName::TripleSingle;
                } else if trip == pair && single == 0 {
                    hand_type.name = HandName::TriplePair;
                }
            }
        } else if pair > 0 {
            hand_type.mult = pair;
            if (hand_type.mult == 1 || is_valid_chain(&cnts[1], 3)) && single == 0 {
                hand_type.name = if cards[0].rank() == 13 {
                    HandName::Rocket
                } else {
                    HandName::Pair
                }
            }
        } else {
            hand_type.mult = single;
            if hand_type.mult == 1 {
                hand_type.name = HandName::Single;
            } else if is_valid_chain(&cnts[0], 5) {
                hand_type.name = HandName::Straight;
            }
        }

        if hand_type.name == HandName::Pass {
            return Err("cards cannot from a hand".to_string());
        }

        Ok(Self {
            kind: hand_type,
            sort_key: cnts
                .iter()
                .rev()
                .flat_map(|v| v.iter().rev().copied())
                .collect(),
            cards,
        })
    }

    pub fn is_pass(&self) -> bool {
        self.kind.name == HandName::Pass
    }

    pub fn join_cards(&self) -> String {
        join_cards(&self.cards)
    }
}
impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.sort_key == other.sort_key
    }
}
impl Eq for Hand {}
impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind
            .cmp(&other.kind)
            .then(self.sort_key.cmp(&other.sort_key))
    }
}
impl<'de> Deserialize<'de> for Hand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Hand::new(Vec::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

fn generate_random_hands() -> [Vec<Card>; 4] {
    let mut deck: Vec<Card> = (0..54).map(Card).collect();
    deck.shuffle(&mut rand::rng());
    [
        deck.drain(..17).collect(),
        deck.drain(..17).collect(),
        deck.drain(..17).collect(),
        deck,
    ]
    .map(|mut cards| {
        cards.sort();
        cards
    })
}

pub struct Game {
    cards: [Vec<Card>; 4], // 4th element contains three hidden cards
    turn: usize,
    bid: usize,
    mult: usize,
    landlord: usize,
    last_idx: usize,
    last_play: Hand,
    passes: usize,
    winner: Option<usize>,
}
impl Game {
    pub fn new() -> Self {
        Self {
            cards: generate_random_hands(),
            turn: rand::random_range(..3),
            bid: 0,
            mult: 1,
            landlord: 3,
            last_play: HAND_PASS,
            last_idx: 0,
            passes: 0,
            winner: None,
        }
    }

    // getter functions
    pub fn landlord(&self) -> usize {
        self.landlord
    }
    pub fn landlord_bonus(&self) -> String {
        join_cards(&self.cards[3])
    }
    pub fn playing(&self) -> bool {
        self.landlord != 3
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
        if (self.passes == 2 && self.bid > 0) || self.bid == 3 {
            self.passes = 0;
            self.landlord = self.last_idx;
            self.cards[self.landlord].extend(self.cards[3].clone());
            self.cards[self.landlord].sort();
            self.turn = self.landlord;
        } else if self.passes == 3 {
            // no one bid, so redeal
            return Ok(true);
        } else {
            self.turn = (idx + 1) % 3;
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
        if !hand.cards.iter().all(|c| self.cards[idx].contains(c)) {
            return Err("cards not in hand".to_string());
        }

        // check if hand id matches
        match hand.kind.name {
            HandName::Pass => {
                // cannot pass thrice in a row or on empty last_play
                if self.last_play.is_pass() || self.passes == 2 {
                    return Err("cannot pass".to_string());
                }
            }
            HandName::Bomb | HandName::Rocket => {}
            _ => {
                // hand ids must match
                if !self.last_play.is_pass() && self.last_play.kind != hand.kind {
                    return Err("hand type does not match".to_string());
                }
            }
        }

        if !hand.is_pass() {
            // check if hand beats last_play
            if hand <= self.last_play {
                // big joker has same rank as small joker but beats it
                if hand.cards[0].0 != 53 {
                    return Err("must beat last hand".to_string());
                }
            }

            // remove cards
            self.cards[idx].retain(|c| !hand.cards.binary_search(c).is_ok());
            if self.cards[idx].is_empty() {
                self.winner = Some(idx);
            }

            // update game state
            if hand.kind.name == HandName::Bomb || hand.kind.name == HandName::Rocket {
                self.mult *= 2;
            }
            self.passes = 0;
            self.last_idx = idx;
            self.last_play = hand;
        } else {
            self.passes += 1;
            if self.passes == 2 {
                self.passes = 0;
                self.last_play = HAND_PASS;
            }
        }

        self.turn = (idx + 1) % 3;
        Ok(())
    }

    pub fn serialize_cards(&self, idx: usize) -> Value {
        serde_json::to_value(&self.cards[idx]).unwrap()
    }

    pub fn serialize(&self) -> Value {
        let mut game = serde_json::Map::new();

        game.insert("turn".to_string(), Value::from(self.turn));
        game.insert("bid".to_string(), Value::from(self.bid));
        game.insert("mult".to_string(), Value::from(self.mult));
        game.insert("passes".to_string(), Value::from(self.passes));
        game.insert(
            "cards_left".to_string(),
            Value::from(self.cards[..3].iter().map(|c| c.len()).collect::<Vec<_>>()),
        );
        game.insert("last_idx".to_string(), Value::from(self.last_idx));

        if self.playing() {
            game.insert(
                "last_play".to_string(),
                serde_json::to_value(&self.last_play).unwrap(),
            );
            game.insert("landlord".to_string(), Value::from(self.landlord));
            game.insert("bonus".to_string(), self.serialize_cards(3));
        }

        // finished
        if let Some(winner) = self.winner {
            game.insert("winner".to_string(), Value::from(winner));
        }

        Value::Object(game)
    }
}
