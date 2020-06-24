use crate::deck::{Card, Rank};
use im::{vector, Vector};

#[derive(PartialEq, Debug, PartialOrd)]
pub struct Score(pub u8);

#[derive(Clone, Debug, PartialEq)]
pub struct Hand(Vector<Card>);

impl Hand {
    pub fn new() -> Self {
        Hand(vector!())
    }

    pub fn add(&self, card: Card) -> Self {
        let mut new_hand = self.clone();
        new_hand.0.push_back(card);
        new_hand
    }

    pub fn score(&self) -> Score {
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

    fn ace_count(&self) -> usize {
        self.0
            .iter()
            .map(|card| card.rank)
            .filter(|rank| *rank == Rank::Ace)
            .count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DealerHand {
    hand: Hand
}

impl DealerHand {
    pub fn new() -> Self {
        DealerHand {hand: Hand::new()}
    }

    pub fn add(&self, card: Card) -> Self {
        let mut new_hand = self.clone();
        new_hand.hand = new_hand.hand.add(card);
        new_hand
    }

    pub fn score(&self) -> Score {
        self.hand.score()
    }

    pub fn hidden_card(&self) -> Option<&Card> {
        self.hand.0.front() 
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::Suit;

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
                rank: Rank::Two,
                suit: Suit::Heart,
            })
            .score();

        assert_eq!(score, Score(2))
    }

    #[test]
    fn a_hand_with_two_cards_adds_their_values() {
        let score = Hand::new()
            .add(Card {
                rank: Rank::Two,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Three,
                suit: Suit::Heart,
            })
            .score();

        assert_eq!(score, Score(5))
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

    #[test]
    fn a_dealer_hands_first_card_is_invisible() {
        let dealer_hand = DealerHand::new()
            .add(Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            });

        assert_eq!(
            dealer_hand.hidden_card(),
            Some(&Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            }));
    }

    #[test]
    fn a_dealer_hands_score_includes_its_invisible_card() {
        let dealer_hand = DealerHand::new()
            .add(Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            });

        assert_eq!(dealer_hand.score(), Score(20));
    }
}
