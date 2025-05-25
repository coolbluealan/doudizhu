use rand::seq::SliceRandom;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Card(usize);
impl Card {
    fn rank(&self) -> usize {
        self.0 / 4
    }
}
impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank() == other.rank()
    }
}
impl Eq for Card {}
impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
                name + " Chain"
            } else {
                name
            }),
        )
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Serialize)]
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
    fn new(cards: Vec<usize>) -> Result<Self, String> {
        if cards.len() == 0 {
            return Ok(HAND_PASS);
        }

        let cards: Vec<Card> = cards.iter().map(|c| Card(*c)).collect();
        // a normal user should not see these errors
        if !cards.iter().all(|c| c.0 < 54) {
            return Err("invalid cards".to_string());
        }
        if !cards.windows(2).all(|w| w[0].0 < w[1].0) {
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
            sort_key: [
                cnts[3].clone(),
                cnts[2].clone(),
                cnts[1].clone(),
                cnts[0].clone(),
            ]
            .concat(),
            cards,
        })
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
        cards.sort_by(|a, b| a.0.cmp(&b.0));
        cards
    })
}

pub struct Game {
    cards: [Vec<Card>; 4], // 4th element contains three hidden cards
    turn: usize,
    bid: usize,
    playing: bool,
    mult: usize,
    landlord: usize,
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
            playing: false,
            mult: 1,
            landlord: 0,
            last_play: HAND_PASS,
            passes: 0,
            winner: None,
        }
    }

    // getter functions
    pub fn playing(&self) -> bool {
        self.playing
    }
    pub fn landlord(&self) -> usize {
        self.landlord
    }
    pub fn winner(&self) -> Option<usize> {
        self.winner
    }
    pub fn score_delta(&self) -> usize {
        self.mult * self.bid
    }

    pub fn bid(&mut self, idx: usize, val: usize) -> Result<(), String> {
        // check phase
        if self.playing {
            return Err("Bidding is over".to_string());
        }
        // check turn
        if self.turn != idx {
            return Err("Not your turn".to_string());
        }

        if val == 0 {
            self.passes += 1;
            if self.passes == 2 {
                self.passes = 0;
                self.playing = true;
            }
        } else {
            if val <= self.bid {
                return Err("Must bid higher than previous bid".to_string());
            }
            self.passes = 0;
            self.bid = val;
            self.landlord = idx;
            if val == 3 {
                self.playing = true;
            }
        }
        Ok(())
    }

    pub fn play(&mut self, idx: usize, hand: Hand) -> Result<(), String> {
        // check phase
        if self.bid == 0 {
            return Err("In bidding phase".to_string());
        }
        // check turn
        if self.turn != idx {
            return Err("Not your turn".to_string());
        }
        // check hand exists in cards
        if !hand.cards.iter().all(|c| self.cards[self.turn].contains(c)) {
            return Err("Cards not in hand".to_string());
        }

        // check if hand id matches
        match hand.kind.name {
            HandName::Pass => {
                // cannot pass thrice in a row
                if self.passes == 2 {
                    return Err("Cannot pass".to_string());
                }
            }
            HandName::Bomb | HandName::Rocket => {}
            _ => {
                // hand ids must match
                if self.last_play.kind != PASS && self.last_play.kind != hand.kind {
                    return Err("Hand type does not match".to_string());
                }
            }
        }
        // check if hand beats last_play
        if hand <= self.last_play {
            return Err("Must beat last hand".to_string());
        }

        if hand.kind != PASS {
            // remove cards
            self.cards[self.turn] = self.cards[self.turn]
                .iter()
                .filter(|c| !hand.cards.contains(c))
                .cloned()
                .collect();
            if self.cards.is_empty() {
                self.winner = Some(self.turn);
            }

            // update game state
            if hand.kind.name == HandName::Bomb || hand.kind.name == HandName::Rocket {
                self.mult *= 2;
            }
            self.passes = 0;
            self.last_play = hand;
        } else {
            self.passes += 1;
            if self.passes == 2 {
                self.last_play = HAND_PASS;
            }
        }

        self.turn = (self.turn + 1) % 3;
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

        if self.playing {
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
