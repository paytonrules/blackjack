use im::Vector;

#[derive(Clone)]
struct Card {
    suit: Suit,
    rank: Rank,
}

#[derive(Clone)]
enum Suit {
    Heart,
    Diamond,
    Spade,
    Club,
}

#[derive(Clone, Copy)]
enum Rank {
    One,
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
    fn to_value(self) -> Value {
        match self {
            Rank::One => Value(1),
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

struct Deck {
    cards: Vec<Card>,
}

#[derive(Clone, PartialEq, Debug)]
struct Value(u8);

#[derive(Clone)]
struct Hand(Vector<Card>);

impl Hand {
    fn new() -> Self {
        Hand(Vector::<Card>::new())
    }

    fn add(self, card: Card) -> Self {
        let mut new_hand = self.clone();
        new_hand.0.push_back(card);
        new_hand
    }

    fn value(self) -> u8 {
        if self.0.len() == 1 {
            self.0.head().unwrap().rank.to_value().0
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_hand_has_value_of_zero() {
        let hand = Hand::new();
        let value = hand.value();

        assert_eq!(value, 0);
    }

    #[test]
    fn a_hand_with_one_card_has_a_value_of_that_cards_rank() {
        let value = Hand::new()
            .add(Card {
                rank: Rank::One,
                suit: Suit::Heart,
            })
            .value();

        assert_eq!(value, 1)
    }

    #[test]
    fn numeric_card_ranks_are_their_values() {
        assert_eq!(Rank::One.to_value(), Value(1));
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
    fn a_hand_with_two_cards_adds_their_value() {
        let value = Hand::new()
            .add(Card {
                rank: Rank::One,
                suit: Suit::Heart,
            })
            .add(Card {
                rank: Rank::Three,
                suit: Suit::Heart,
            })
            .value();

        assert_eq!(value, 4)
    }
}
