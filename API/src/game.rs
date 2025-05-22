use rand::seq::SliceRandom;
use std::fmt::Display;

#[derive(Clone, PartialEq, Debug)]
pub struct Card(u8);
impl Card {
    pub fn rank(&self) -> u8 {
        self.0 / 4
    }
    pub fn suit(&self) -> u8 {
        self.0 % 4
    }
}

#[derive(Clone, PartialEq, Debug)]
enum TrickType {
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
impl Display for TrickType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let trick_name = match self {
            TrickType::Pass => "Pass",
            TrickType::Single => "Single card",
            TrickType::Straight => "Straight",
            TrickType::Pair => "Pair",
            TrickType::Triple => "Triple",
            TrickType::TripleSingle => "Triple with single attached",
            TrickType::TriplePair => "Triple with pair attached",
            TrickType::QuadSingle => "Quad with singles attached",
            TrickType::QuadPair => "Quad with pairs attached",
            TrickType::Bomb => "Bomb",
            TrickType::Rocket => "Rocket",
        };
        write!(f, "{}", trick_name)
    }
}
#[derive(PartialEq, Debug)]
pub struct Trick {
    id: TrickType,
    mult: usize,
    cards: Vec<Card>,
}
impl Trick {
    pub fn new(cards: Vec<Card>) -> Result<Self, String> {
        if !cards.iter().all(|c| c.0 < 54) {
            return Err("Invalid cards".to_string());
        }
        if !cards
            .windows(2)
            .all(|w| w[0].rank() >= w[1].rank() && w[0].0 != w[1].0)
        {
            return Err("Unsorted cards".to_string());
        }

        // count multiplicities in the trick
        let mut cnts: [Vec<u8>; 4] = Default::default();
        let mut count = 1;
        for i in 1..cards.len() {
            if cards[i].rank() == cards[i - 1].rank() {
                count += 1;
            } else {
                cnts[count - 1].push(cards[i].rank());
                count = 1;
            }
        }
        let single = cnts[0].len();
        let pair = cnts[1].len();
        let trip = cnts[2].len();
        let quad = cnts[3].len();

        let is_valid_chain = |v: &[u8], sz: usize| {
            v.len() >= sz
                && v.first().is_some_and(|&first| first < 12)
                && v.windows(2).all(|w| w[0] == w[1] + 1)
        };

        let mut id = TrickType::Pass;
        let mut mult = 1;
        if quad > 0 {
            if quad == 1 && trip == 0 {
                if single == 0 && pair == 0 {
                    id = TrickType::Bomb;
                } else if single == 2 && pair == 0 {
                    id = TrickType::QuadSingle;
                } else if single == 0 && pair == 2 {
                    id = TrickType::QuadPair;
                }
            }
        } else if trip > 0 {
            mult = trip;
            if mult == 1 || is_valid_chain(&cnts[2], 2) {
                if single == 0 && pair == 0 {
                    id = TrickType::Triple;
                } else if trip == single && pair == 0 {
                    id = TrickType::TripleSingle;
                } else if trip == pair && single == 0 {
                    id = TrickType::TriplePair;
                }
            }
        } else if pair > 0 {
            mult = pair;
            if mult == 1 || is_valid_chain(&cnts[1], 3) && single == 0 {
                id = if cards[0].rank() == 13 {
                    TrickType::Rocket
                } else {
                    TrickType::Pair
                }
            }
        } else {
            mult = single;
            if mult == 1 {
                id = TrickType::Single;
            } else if is_valid_chain(&cnts[0], 5) {
                id = TrickType::Straight;
            }
        }

        if !cards.is_empty() && id == TrickType::Pass {
            return Err("Cards cannot from a trick".to_string());
        }

        Ok(Self { id, mult, cards })
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
}

pub struct Game {
    hands: [Vec<Card>; 4], // 4th hand is hidden three cards
    start: usize,
    turn: usize,
    bid: u8,
    mult: u32,
    history: Vec<Vec<Trick>>,
}
impl Game {
    pub fn new() -> Self {
        let start = rand::random_range(..3);
        Self {
            hands: generate_random_hands(),
            start,
            turn: start,
            bid: 0,
            mult: 1,
            history: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.history.last().is_none_or(|s| match &s[..] {
            [.., a, b] => a.id == TrickType::Pass && b.id == TrickType::Pass,
            _ => false,
        })
    }

    fn current_trick(&self) -> &Trick {
        self.history.last().unwrap().first().unwrap() // first trick in the stack
    }

    pub fn play(&mut self, trick: Trick) -> Result<(), String> {
        // check trick exists in hand
        if !trick
            .cards
            .iter()
            .all(|c| self.hands[self.turn].contains(c))
        {
            return Err("Cards not in hand".to_string());
        }

        // check if trick can be played
        let is_empty = self.is_empty();
        match &trick.id {
            TrickType::Pass => {
                // cannot pass thrice in a row
                if is_empty {
                    return Err("Cannot pass".to_string());
                }
            }
            TrickType::Bomb | TrickType::Rocket => self.mult *= 2,
            id => {
                if !is_empty {
                    // trick type and multiplicity must match
                    let current_trick = self.current_trick();
                    if current_trick.id != *id || current_trick.mult != trick.mult {
                        return Err("Trick type does not match".to_string());
                    }
                }
            }
        }

        self.turn = (self.turn + 1) % 3;
        if trick.id != TrickType::Pass {
            // remove cards
            self.hands[self.turn] = self.hands[self.turn]
                .iter()
                .filter(|c| !trick.cards.contains(c))
                .cloned()
                .collect();
        }
        if is_empty {
            self.history.push(vec![trick]);
        } else {
            self.history.last_mut().unwrap().push(trick);
        }
        Ok(())
    }
}
