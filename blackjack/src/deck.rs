use im::{vector, Vector};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::error::Error;
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(PartialEq, Debug)]
pub struct Value(pub u8);

#[derive(Clone, Copy, PartialEq, Debug, EnumIter, Hash, Eq)]
pub enum Suit {
    Heart,
    Diamond,
    Spade,
    Club,
}

#[derive(PartialEq, Clone, Debug, Copy, EnumIter, Display, Hash, Eq)]
pub enum Rank {
    Two,
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

impl Rank {
    pub fn to_value(self) -> Value {
        match self {
            Rank::Two => Value(2),
            Rank::Three => Value(3),
            Rank::Four => Value(4),
            Rank::Five => Value(5),
            Rank::Six => Value(6),
            Rank::Seven => Value(7),
            Rank::Eight => Value(8),
            Rank::Nine => Value(9),
            Rank::Ten | Rank::King | Rank::Queen | Rank::Jack => Value(10),
            Rank::Ace => Value(11),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Debug)]
pub struct EmptyDeckError;

impl Error for EmptyDeckError {}

impl fmt::Display for EmptyDeckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "You're dealing from an empty deck!")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deck {
    pub cards: Vector<Card>,
}

impl Deck {
    pub fn new_with_cards(cards: Vector<Card>) -> Self {
        Deck { cards }
    }

    pub fn standard_deck() -> Self {
        let mut cards = vector!();
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                cards.push_back(Card { suit, rank });
            }
        }
        Deck::new_with_cards(cards)
    }

    pub fn shuffle(&self) -> Self {
        let mut rng = thread_rng();
        let mut cards_as_vec = self.cards_to_vec();
        cards_as_vec.shuffle(&mut rng);
        Self::new_with_cards(Vector::from(cards_as_vec))
    }

    pub fn deal(&self) -> Result<(Deck, Card), EmptyDeckError> {
        let mut deck = self.clone();
        let card = deck.cards.pop_front().ok_or(EmptyDeckError)?;
        Ok((deck, card))
    }

    fn cards_to_vec(&self) -> Vec<Card> {
        self.cards.clone().into_iter().collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use im::HashSet;

    #[test]
    fn numeric_card_ranks_are_their_values() {
        assert_eq!(Rank::Two.to_value(), Value(2));
        assert_eq!(Rank::Three.to_value(), Value(3));
        assert_eq!(Rank::Four.to_value(), Value(4));
        assert_eq!(Rank::Five.to_value(), Value(5));
        assert_eq!(Rank::Six.to_value(), Value(6));
        assert_eq!(Rank::Seven.to_value(), Value(7));
        assert_eq!(Rank::Eight.to_value(), Value(8));
        assert_eq!(Rank::Nine.to_value(), Value(9));
        assert_eq!(Rank::Ten.to_value(), Value(10));
    }

    #[test]
    fn face_cards_are_value_ten() {
        assert_eq!(Rank::King.to_value(), Value(10));
        assert_eq!(Rank::Jack.to_value(), Value(10));
        assert_eq!(Rank::Queen.to_value(), Value(10));
    }

    #[test]
    fn aces_are_eleven() {
        assert_eq!(Rank::Ace.to_value(), Value(11));
    }

    #[test]
    fn deal_takes_the_top_card_off_the_deck() -> Result<(), EmptyDeckError> {
        let deck = Deck {
            cards: vector!(
                Card {
                    rank: Rank::Ace,
                    suit: Suit::Heart
                },
                Card {
                    rank: Rank::King,
                    suit: Suit::Heart
                }
            ),
        };

        let (new_deck, card) = deck.deal()?;

        assert_eq!(
            card,
            Card {
                rank: Rank::Ace,
                suit: Suit::Heart
            }
        );
        assert_eq!(
            new_deck,
            Deck {
                cards: vector!(Card {
                    rank: Rank::King,
                    suit: Suit::Heart
                })
            }
        );
        Ok(())
    }

    #[test]
    fn deal_is_not_okay_if_the_deck_is_empty() {
        let deck = Deck { cards: vector!() };

        let result = deck.deal();

        assert!(result.is_err(), "Cannot deal from an empty deck");
    }

    #[test]
    fn can_create_a_standard_deck() {
        let deck = Deck::standard_deck();

        assert_eq!(
            deck.cards.len(),
            Rank::iter().count() * Suit::iter().count()
        );
        assert_ne!(deck.cards.len(), 0);
        for rank in Rank::iter() {
            let suits: Vec<Suit> = deck
                .cards
                .iter()
                .filter_map(|card| {
                    if card.rank == rank {
                        Some(card.suit)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                suits,
                Suit::iter().collect::<Vec<Suit>>(),
                "rank {} had the wrong suits",
                rank
            );
        }
    }

    #[test]
    fn shuffle_reorders_the_deck_without_changing_entries() {
        let new_deck = Deck::standard_deck();

        let shuffled_deck = new_deck.shuffle();

        assert_ne!(new_deck.cards, shuffled_deck.cards);
        let new_deck_set = new_deck.cards.into_iter().collect::<HashSet<Card>>();
        let shuffled_deck_set = shuffled_deck.cards.into_iter().collect::<HashSet<Card>>();
        assert_eq!(new_deck_set, shuffled_deck_set);
    }
}
