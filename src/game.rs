use crate::deck::Deck;
use crate::hand::{DealerHand, Hand, Score};
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
struct Context {
    deck: Deck,
    player_hand: Hand,
    computer_hand: DealerHand,
}

impl Context {
    fn new() -> Self {
        Context {
            deck: Deck::new(),
            player_hand: Hand::new(),
            computer_hand: DealerHand::new(),
        }
    }

    fn dealer_blackjack(&self) -> bool {
        self.computer_hand.score() == Score(21)
    }
}

#[derive(Debug)]
struct InvalidStateError;

impl Error for InvalidStateError {}

impl fmt::Display for InvalidStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The game is in an invalid state for this transition")
    }
}

#[derive(Debug, PartialEq)]
enum GameState {
    Ready(Context),
    WaitingForPlayer(Context),
    DealerWins(Context),
    PlayerWins(Context),
}

fn deal(state: GameState) -> Result<GameState, Box<dyn std::error::Error>> {
    match state {
        GameState::Ready(context) => {
            let (new_deck, first_card) = context.deck.deal()?;
            let (new_deck, second_card) = new_deck.deal()?;
            let (new_deck, third_card) = new_deck.deal()?;
            let (new_deck, fourth_card) = new_deck.deal()?;
            let player_hand = Hand::new().add(first_card).add(second_card);
            let computer_hand = DealerHand::new().add(third_card).add(fourth_card);
            
            let new_context = Context {
                player_hand,
                computer_hand,
                deck: Deck::new(),
            };
            
            if new_context.dealer_blackjack() {
                Ok(GameState::DealerWins(new_context))
            } else {
                Ok(GameState::WaitingForPlayer(new_context))
            }

        }
        _ => Err(Box::new(InvalidStateError {})),
    }
}

#[cfg(test)]
mod game_state_machine {
    use super::*;
    use crate::deck::{Card, Rank, Suit};
    use im::{vector, Vector};

    fn minimal_cards() -> Vector<Card> {
        vector!(
            Card {
                rank: Rank::Ace,
                suit: Suit::Heart
            },
            Card {
                rank: Rank::King,
                suit: Suit::Spade
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Club
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Diamond
            }
        )
    }

    #[test]
    fn deal_transitions_from_ready_to_waiting_for_player() -> Result<(), Box<dyn Error>> {
        let game_state = GameState::Ready(Context {
            deck: Deck::new_with_cards(minimal_cards().clone()),
            computer_hand: DealerHand::new(),
            player_hand: Hand::new(),
        });

        let new_game_state = deal(game_state)?;
        match new_game_state {
            GameState::WaitingForPlayer(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }

    #[test]
    fn other_transitions_fail() {
        let game_state = GameState::WaitingForPlayer(Context::new());

        let result = deal(game_state);

        assert!(result.is_err(), "deal is only a transition from ready")
    }

    #[test]
    fn deal_gives_the_player_and_computer_hands() -> Result<(), Box<dyn Error>> {
        let cards = minimal_cards();
        let context = Context {
            deck: Deck::new_with_cards(cards.clone()),
            player_hand: Hand::new(),
            computer_hand: DealerHand::new(),
        };

        let game_state = GameState::Ready(context);

        if let GameState::WaitingForPlayer(context) = deal(game_state)? {
            assert_eq!(Deck::new(), context.deck);
            assert_eq!(Hand::new().add(cards[0]).add(cards[1]), context.player_hand);
            assert_eq!(
                DealerHand::new().add(cards[2]).add(cards[3]),
                context.computer_hand
            );
            Ok(())
        } else {
            Err(Box::new(InvalidStateError {}))
        }
    }

    #[test]
    fn deal_goes_to_dealer_won_when_dealer_has_blackjack() -> Result<(), Box<dyn Error>> {
        let dealer_blackjack_hand = vector!(
            Card {
                suit: Suit::Heart,
                rank: Rank::One
            },
            Card {
                suit: Suit::Heart,
                rank: Rank::One
            },
            Card {
                suit: Suit::Heart,
                rank: Rank::Ace
            },
            Card {
                suit: Suit::Heart,
                rank: Rank::Ten
            },
        );
        let context = Context {
            deck: Deck::new_with_cards(dealer_blackjack_hand),
            player_hand: Hand::new(),
            computer_hand: DealerHand::new(),
        };

        let game_state = GameState::Ready(context);

        let new_state = deal(game_state)?;
        match new_state {
            GameState::DealerWins(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }

    // test function - card with ignored suit
    // factory function - context with deck (real func)
    // test a deck with more than four cards
    // Ready is the initial state - it doesn't context, it could start with one.
}
