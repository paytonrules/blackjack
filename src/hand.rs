use im::{vector, Vector};
use crate::deck::{Card, Rank};

#[derive(PartialEq, Debug)]
struct Score(u8);

#[derive(Clone, Debug, PartialEq)]
pub struct Hand(Vector<Card>);

impl Hand {
    pub fn new() -> Self {
        Hand(vector!())
    }

    pub fn add(self, card: Card) -> Self {
        let mut new_hand = self.clone();
        new_hand.0.push_back(card);
        new_hand
    }

    fn score(self) -> Score {
        let hard_value = self.0.iter().map(|card| card.rank.to_value().0).sum();

        let mut soft_value = hard_value;
        for _ in 0..self.ace_count() {
            if soft_value > 21 {
                soft_value -= 10;
                if soft_value <= 21 {
                    break;
                }
            }
        }
        Score(soft_value)
    }

    fn ace_count(self) -> usize {
        self.0
            .iter()
            .map(|card| card.rank)
            .filter(|rank| *rank == Rank::Ace)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{Suit};

    #[test]
    fn an_empty_hand_has_a_score_of_zero() {
        let hand = Hand::new();
        let score = hand.score();

        assert_eq!(score, Score(0));
    }

    #[test]
    fn a_hand_with_one_card_has_a_score_of_that_cards_rank() {
        let score = Hand::new()
            .add(Card {
                rank: Rank::One,
                suit: Suit::Heart,
            })
            .score();

        assert_eq!(score, Score(1))
    }

    #[test]
    fn a_hand_with_two_cards_adds_their_values() {
        let score = Hand::new()
            .add(Card {
                rank: Rank::One,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Three,
                suit: Suit::Heart,
            })
            .score();

        assert_eq!(score, Score(4))
    }

    #[test]
    fn a_hand_with_ten_ace_is_twenty_one() {
        let score = Hand::new()
            .add(Card {
                rank: Rank::Ace,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            })
            .score();

        assert_eq!(score, Score(21))
    }

    #[test]
    fn a_hand_with_ten_ace_ace_is_twelve() {
        let score = Hand::new()
            .add(Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Ace,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Ace,
                suit: Suit::Heart,
            })
            .score();

        assert_eq!(score, Score(12))
    }

    #[test]
    fn a_hand_can_still_bust_with_aces() {
        let score = Hand::new()
            .add(Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Ace,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Ace,
                suit: Suit::Heart,
            })
            .score();

        assert_eq!(score, Score(22))
    }
}