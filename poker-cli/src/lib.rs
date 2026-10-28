use itertools::Itertools;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Spades,
    Clubs,
    Hearts,
    Diamonds,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HandType {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hand {
    HighCard(Rank),
    Pair(Rank),
    TwoPair { high_pair: Rank, low_pair: Rank },
    ThreeOfAKind(Rank),
    Straight { high: Rank },
    Flush,
    FullHouse { three_of: Rank, two_of: Rank },
    FourOfAKind(Rank),
    StraightFlush { high: Rank },
}

impl TryFrom<usize> for Rank {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Ace),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            9 => Ok(Self::Nine),
            10 => Ok(Self::Ten),
            11 => Ok(Self::Jack),
            12 => Ok(Self::Queen),
            13 => Ok(Self::King),
            14 => Ok(Self::Ace), // again
            0 | 15.. => Err(()),
        }
    }
}

impl Rank {
    pub fn name(self) -> &'static str {
        match self {
            Self::Ace => "Ace",
            Self::Two => "Two",
            Self::Three => "Three",
            Self::Four => "Four",
            Self::Five => "Five",
            Self::Six => "Six",
            Self::Seven => "Seven",
            Self::Eight => "Eight",
            Self::Nine => "Nine",
            Self::Ten => "Ten",
            Self::Jack => "Jack",
            Self::Queen => "Queen",
            Self::King => "King",
        }
    }
    pub fn all() -> impl Iterator<Item = Self> {
        return [
            Self::Two,
            Self::Three,
            Self::Four,
            Self::Five,
            Self::Six,
            Self::Seven,
            Self::Eight,
            Self::Nine,
            Self::Ten,
            Self::Jack,
            Self::Queen,
            Self::King,
            Self::Ace,
        ]
        .into_iter();
    }
}

impl Suit {
    pub fn name(self) -> &'static str {
        match self {
            Self::Spades => "Spades",
            Self::Clubs => "Clubs",
            Self::Hearts => "Hearts",
            Self::Diamonds => "Diamonds",
        }
    }
}

impl Card {
    pub fn shares_rank(self, other: Self) -> bool {
        self.rank == other.rank
    }

    pub fn shares_suit(self, other: Self) -> bool {
        self.suit == other.suit
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.rank.name())?;
        f.write_str("of")?;
        f.write_str(self.suit.name())?;
        Ok(())
    }
}

impl From<Hand> for HandType {
    fn from(value: Hand) -> Self {
        match value {
            Hand::Flush => Self::Flush,
            Hand::FourOfAKind(_) => Self::FourOfAKind,
            Hand::FullHouse { .. } => Self::FullHouse,
            Hand::HighCard(_) => Self::HighCard,
            Hand::Pair(_) => Self::Pair,
            Hand::Straight { .. } => Self::Straight,
            Hand::StraightFlush { .. } => Self::StraightFlush,
            Hand::ThreeOfAKind(_) => Self::ThreeOfAKind,
            Hand::TwoPair { .. } => Self::TwoPair,
        }
    }
}

impl Hand {
    pub fn classify(mut cards: [Card; 5]) -> Self {
        cards.sort_by_key(|c| c.rank);

        let flush = (0..5).all(|i| cards[i].suit == cards[i + 1].suit);
        let straight = (0..5).all(|i| cards[i].rank as usize + 1 == cards[i + 1].rank as usize);
        let (counts, max_rank): ([usize; 15], _) = {
            let mut counts = [0; 15];
            let mut max_rank = Rank::Two;
            for card in cards {
                counts[card.rank as usize] += 1;
                if card.rank > max_rank {
                    max_rank = card.rank
                }
            }
            (counts, max_rank)
        };
        let (max_pair_size, max_size_pair_rank): (usize, Rank) = {
            let mut max_pair = counts[2];
            let mut max_pair_rank = Rank::Two;
            for i in 3..counts.len() {
                if counts[i] > max_pair {
                    max_pair = counts[i];
                    max_pair_rank = i.try_into().unwrap();
                }
            }
            (max_pair, max_pair_rank)
        };

        if straight && flush {
            return Self::StraightFlush { high: max_rank };
        }
        if max_pair_size == 4 {
            return Self::FourOfAKind(max_size_pair_rank);
        }
        if max_pair_size == 3 {
            let two_rank: Option<Rank> = counts
                .iter()
                .skip(2)
                .zip(Rank::all())
                .find(|(x, _r)| **x == 2)
                .map(|(_x, r)| r);
            if let Some(two_rank) = two_rank {
                return Self::FullHouse {
                    three_of: max_size_pair_rank,
                    two_of: two_rank,
                };
            }
        }
        if flush {
            return Self::Flush;
        }
        if straight {
            return Self::Straight { high: max_rank };
        }
        if max_pair_size == 3 {
            return Self::ThreeOfAKind(max_size_pair_rank);
        }
        match counts
            .iter()
            .skip(2)
            .zip(Rank::all())
            .filter(|(x, _r)| **x == 2)
            .fold((0usize, [None, None]), |(n, [a, b]), (&x, r)| {
                if x == 2 {
                    if n == 0 {
                        (1, [Some(r), b])
                    } else {
                        (2, [a, Some(r)])
                    }
                } else {
                    (n, [a, b])
                }
            }) {
            (3.., _) => unreachable!("only 0..=2 are ever written"),
            (2, [Some(a), Some(b)]) => Self::TwoPair {
                high_pair: b,
                low_pair: a,
            },
            (2, _) => unreachable!("the array has Some(_) elements equal to the number"),
            (1, _) => Self::Pair(max_size_pair_rank),
            (0, _) => Self::HighCard(max_rank),
        }
    }

    pub fn find_best(hand: [Card; 2], flop: [Card; 3], rest: &[Card]) -> (Self, [Card; 5]) {
        let mut best_hand = Self::classify([hand[0], hand[1], flop[0], flop[1], flop[2]]);
        let mut best_cards = [hand[0], hand[1], flop[0], flop[1], flop[2]];
        for extra_threes in flop.iter().chain(rest.iter()).permutations(3) {
            let &[&a, &b, &c] = &*extra_threes else {
                debug_assert!(false, "unreachable");
                continue;
            };
            let cards = [hand[0], hand[1], a, b, c];
            let hand = Self::classify(cards);
            if hand > best_hand {
                best_hand = hand;
                best_cards = cards;
            }
        }
        (best_hand, best_cards)
    }
}
