use std::fmt::{Display, Formatter};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

const SUIT_MAP: [&str; 4] = ["♣️", "♦️", "♥️", "♠️"];
const RANK_MAP: [&str; 15] = [
    "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A", "2", "J", "J",
];
#[derive(Eq, PartialEq, PartialOrd, Ord, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Card(usize);
impl Card {
    fn rank(&self) -> usize {
        match self.0 {
            53 => 14,
            _ => self.0 / 4,
        }
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
pub fn join(cards: &Vec<Card>) -> String {
    cards
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Eq, PartialEq, Debug)]
enum HandName {
    Pass,
    Single,
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

#[derive(Eq, PartialEq, Debug)]
struct HandType {
    name: HandName,
    mult: usize, // length of chain or number of cards in bomb
}
impl HandType {
    const PASS: HandType = HandType {
        name: HandName::Pass,
        mult: 1,
    };
}
impl Serialize for HandType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut name = self.name.to_string();
        serializer.serialize_str(if self.mult == 1 || self.name == HandName::Bomb {
            &name
        } else {
            if self.name == HandName::Single {
                "Straight"
            } else {
                name += "Chained";
                &name
            }
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Hand {
    kind: HandType,
    #[serde(skip)]
    sort_key: Vec<usize>,
    cards: Vec<Card>,
}
impl Default for Hand {
    fn default() -> Self {
        Self::PASS
    }
}
impl Hand {
    pub const PASS: Self = Self {
        kind: HandType::PASS,
        sort_key: Vec::new(),
        cards: Vec::new(),
    };

    pub fn cards(&self) -> &Vec<Card> {
        &self.cards
    }

    pub fn new(players: usize, cards: Vec<usize>) -> Result<Self, String> {
        if cards.len() == 0 {
            return Ok(Self::PASS);
        }

        // a normal user should not see these errors
        if !cards.iter().all(|c| *c < 54) {
            return Err("invalid cards".to_string());
        }
        if !cards.is_sorted() {
            return Err("unsorted cards".to_string());
        }

        // count multiplicities in the hand
        let cards: Vec<Card> = cards.iter().map(|c| Card(*c)).collect();
        let mut cnts: [Vec<usize>; 9] = Default::default();
        let mut count = 1;
        for i in 0..cards.len() - 1 {
            if cards[i].rank() == cards[i + 1].rank() {
                count += 1;
            } else {
                cnts[count].push(cards[i].rank());
                count = 1;
            }
        }
        cnts[count].push(cards[cards.len() - 1].rank());

        let is_valid_chain = |v: &[usize], sz: usize| {
            // note chain cannot include 2
            v.len() == 1
                || (v.len() >= sz && v[v.len() - 1] < 12 && v.windows(2).all(|w| w[0] + 1 == w[1]))
        };

        let single = cnts[1].len();
        let pair = cnts[2].len();
        let trip = cnts[3].len();
        let bomb: usize = cnts[4..].iter().map(|v| v.len()).sum();

        let mut hand_type = HandType::PASS;
        if bomb > 0 {
            if bomb == 1 && trip == 0 {
                if single == 0 && pair == 0 {
                    hand_type.name = HandName::Bomb;
                    hand_type.mult = cards.len();
                } else if players == 3 {
                    if single == 2 && pair == 0 {
                        hand_type.name = HandName::QuadSingle;
                    } else if single == 0 && pair == 2 {
                        hand_type.name = HandName::QuadPair;
                    }
                }
            }
        } else if trip > 0 && cnts[3][0] < 13 {
            if is_valid_chain(&cnts[3], 2) {
                hand_type.mult = trip;
                if single == 0 && pair == 0 {
                    hand_type.name = HandName::Triple;
                } else if trip == single && pair == 0 && players == 3 {
                    hand_type.name = HandName::TripleSingle;
                } else if trip == pair && single == 0 {
                    hand_type.name = HandName::TriplePair;
                }
            }
        } else if pair > 0 {
            if single == 0 {
                if pair == 2 && cnts[2][0] == 13 {
                    hand_type.name = HandName::Rocket;
                } else if is_valid_chain(&cnts[2], 3) {
                    hand_type.mult = pair;
                    hand_type.name = HandName::Pair
                }
            }
        } else {
            if single == 2 && cnts[1][0] == 13 && players == 3 {
                hand_type.name = HandName::Rocket;
            } else if is_valid_chain(&cnts[1], 5) {
                hand_type.mult = single;
                hand_type.name = HandName::Single;
            }
        }

        if hand_type.name == HandName::Pass {
            return Err("cards cannot from a hand".to_string());
        }

        Ok(Self {
            kind: hand_type,
            sort_key: cnts[1..]
                .iter()
                .rev()
                .flat_map(|v| v.iter().rev().copied())
                .collect(),
            cards,
        })
    }

    pub fn deal_hands(players: usize) -> Vec<Vec<Card>> {
        let mut deck: Vec<usize> = if players == 3 {
            (0..54).collect()
        } else {
            (0..54).chain(0..54).collect()
        };
        deck.shuffle(&mut rand::rng());

        if players == 3 {
            [
                deck.drain(..17).collect(),
                deck.drain(..17).collect(),
                deck.drain(..17).collect(),
                deck,
            ]
            .to_vec()
        } else {
            [
                deck.drain(..25).collect(),
                deck.drain(..25).collect(),
                deck.drain(..25).collect(),
                deck.drain(..25).collect(),
                deck,
            ]
            .to_vec()
        }
        .into_iter()
        .map(|mut cards| {
            cards.sort();
            cards.into_iter().map(Card).collect()
        })
        .collect()
    }

    pub fn is_pass(&self) -> bool {
        self.kind.name == HandName::Pass
    }

    pub fn is_double(&self, players: usize) -> bool {
        self.kind.name == HandName::Rocket
            || (self.kind.name == HandName::Bomb && (players == 3 || self.kind.mult >= 6))
    }

    pub fn can_play(&self, last_play: &Self) -> Result<(), String> {
        if self.kind.name != last_play.kind.name {
            if last_play.is_pass()
                || self.kind.name == HandName::Rocket
                || (self.kind.name == HandName::Bomb && last_play.kind.name != HandName::Rocket)
            {
                Ok(())
            } else {
                Err("hand type does not match".to_string())
            }
        } else if self.kind.name == HandName::Bomb {
            if self.kind.mult < last_play.kind.mult {
                Err("bomb has less cards than previous play".to_string())
            } else if self.kind.mult == last_play.kind.mult && self.sort_key <= last_play.sort_key {
                Err("bomb is lower than previous play".to_string())
            } else {
                Ok(())
            }
        } else {
            if self.kind.mult != last_play.kind.mult {
                Err("number of cards do not match".to_string())
            } else if self.sort_key <= last_play.sort_key {
                Err("hand is lower than previous play".to_string())
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_pass() {
        let h = Hand::new(3, vec![]).unwrap();
        assert!(h.is_pass());
    }

    #[test]
    fn create_single() {
        let h = Hand::new(3, vec![10]).unwrap();
        assert_eq!(h.kind.name, HandName::Single);
        assert_eq!(h.kind.mult, 1);

        let sj = Hand::new(3, vec![52]).unwrap();
        let bj = Hand::new(3, vec![53]).unwrap();
        assert!(sj.can_play(&h).is_ok());
        assert!(bj.can_play(&sj).is_ok());
    }

    #[test]
    fn create_straight() {
        let h = Hand::new(3, vec![10, 14, 18, 22, 26]).unwrap();
        assert_eq!(h.kind.name, HandName::Single);
        assert_eq!(h.kind.mult, 5);
        Hand::new(3, vec![10, 14, 18, 22]).unwrap_err();
    }

    #[test]
    fn create_pair() {
        let h = Hand::new(3, vec![10, 11]).unwrap();
        assert_eq!(h.kind.name, HandName::Pair);
        assert_eq!(h.kind.mult, 1);

        let sjs = Hand::new(4, vec![52, 52]).unwrap();
        assert_eq!(sjs.kind.name, HandName::Pair);
        Hand::new(4, vec![52, 53]).unwrap_err();
    }

    #[test]
    fn create_pair_chain() {
        let h = Hand::new(3, vec![10, 11, 12, 13, 16, 17]).unwrap();
        assert_eq!(h.kind.name, HandName::Pair);
        assert_eq!(h.kind.mult, 3);
        Hand::new(3, vec![10, 11, 12, 23]).unwrap_err();
    }

    #[test]
    fn create_triple() {
        let h = Hand::new(3, vec![0, 1, 2]).unwrap();
        assert_eq!(h.kind.name, HandName::Triple);
        assert_eq!(h.kind.mult, 1);
        Hand::new(4, vec![52, 52, 53]).unwrap_err();
    }

    #[test]
    fn create_triple_chain() {
        let h = Hand::new(3, vec![0, 1, 2, 4, 5, 6]).unwrap();
        assert_eq!(h.kind.name, HandName::Triple);
        assert_eq!(h.kind.mult, 2);
        Hand::new(3, vec![0, 1, 2, 8, 9, 10]).unwrap_err();
    }

    #[test]
    fn create_triple_single() {
        let h = Hand::new(3, vec![0, 1, 2, 10]).unwrap();
        assert_eq!(h.kind.name, HandName::TripleSingle);
        assert_eq!(h.kind.mult, 1);
        Hand::new(4, vec![0, 1, 2, 10]).unwrap_err();
        Hand::new(3, vec![0, 1, 2, 10, 12]).unwrap_err();
    }
    #[test]
    fn create_triple_single_chain() {
        let h = Hand::new(3, vec![0, 1, 2, 4, 5, 6, 10, 12]).unwrap();
        assert_eq!(h.kind.name, HandName::TripleSingle);
        assert_eq!(h.kind.mult, 2);
        Hand::new(3, vec![0, 1, 2, 4, 5, 6, 10, 11]).unwrap_err();
    }

    #[test]
    fn create_triple_pair() {
        let h = Hand::new(3, vec![0, 1, 2, 10, 11]).unwrap();
        assert_eq!(h.kind.name, HandName::TriplePair);
        assert_eq!(h.kind.mult, 1);
        Hand::new(4, vec![0, 1, 2, 10, 11]).unwrap();
        Hand::new(4, vec![0, 1, 2, 52, 53]).unwrap_err();
    }

    #[test]
    fn create_triple_pair_chain() {
        let chain = Hand::new(3, vec![0, 1, 2, 4, 5, 6, 10, 11, 12, 13]).unwrap();
        assert_eq!(chain.kind.name, HandName::TriplePair);
        assert_eq!(chain.kind.mult, 2);
    }

    #[test]
    fn create_quad_single() {
        let h = Hand::new(3, vec![0, 1, 2, 3, 10, 12]).unwrap();
        assert_eq!(h.kind.name, HandName::QuadSingle);
        assert_eq!(h.kind.mult, 1);
        Hand::new(4, vec![0, 1, 2, 3, 10, 12]).unwrap_err();
    }

    #[test]
    fn create_quad_pair() {
        let h = Hand::new(3, vec![0, 1, 2, 3, 10, 11, 12, 13]).unwrap();
        assert_eq!(h.kind.name, HandName::QuadPair);
        assert_eq!(h.kind.mult, 1);
        Hand::new(4, vec![0, 1, 2, 3, 10, 11, 12, 13]).unwrap_err();
    }

    #[test]
    fn create_bomb() {
        let h = Hand::new(3, vec![0, 1, 2, 3]).unwrap();
        assert_eq!(h.kind.name, HandName::Bomb);
        assert_eq!(h.kind.mult, 4);
        assert!(h.is_double(3));

        let h = Hand::new(4, vec![0, 0, 1, 2, 3]).unwrap();
        assert_eq!(h.kind.name, HandName::Bomb);
        assert_eq!(h.kind.mult, 5);
        assert!(!h.is_double(4));

        let h = Hand::new(4, vec![0, 0, 1, 1, 2, 3]).unwrap();
        assert!(h.is_double(4));
    }

    #[test]
    fn create_rocket() {
        let h = Hand::new(3, vec![52, 53]).unwrap();
        assert_eq!(h.kind.name, HandName::Rocket);
        assert!(h.is_double(3));
        Hand::new(4, vec![52, 53]).unwrap_err();

        let h = Hand::new(4, vec![52, 52, 53, 53]).unwrap();
        assert_eq!(h.kind.name, HandName::Rocket);
        assert!(h.is_double(4));
    }
}
