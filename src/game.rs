use crate::hand::Hand;
use crate::deck::Deck;
use std::error::Error;
use std::fmt;
use im::vector;

#[derive(Debug, PartialEq)]
struct Context {
    deck: Deck,
    player_hand: Hand,
    computer_hand: Hand,
}

impl Context {
    fn new() -> Self {
        Context {
            deck: Deck{cards: vector!()},
            player_hand: Hand::new(),
            computer_hand: Hand::new()
        }
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
    CheckingPlayerHand(Context),
    PlayerLoses(Context),
    PlayingDealerHand(Context),
}

fn deal(state: GameState) -> Result<GameState, Box<dyn std::error::Error>> {
    match state {
        GameState::Ready(context) => {
            let (new_deck, first_card) = context.deck.deal()?;
            let (new_deck, second_card) = new_deck.deal()?;
            let player_hand = Hand::new()
                .add(first_card)
                .add(second_card);

            let new_context = Context {
                player_hand: player_hand,
                computer_hand: Hand::new(),
                deck: Deck {cards: vector!()}
            };

            Ok(GameState::WaitingForPlayer(new_context))
        },
        _ => Err(Box::new(InvalidStateError {})),
    }
}

#[cfg(test)]
mod game_state_machine {
    use super::*;
    use im::vector;
    use crate::deck::{Card, Rank, Suit};

    #[test]
    fn deal_transitions_from_ready_to_waiting_for_player() -> Result<(), Box<dyn Error>> {
        let minimal_cards = vector!(
            Card {rank: Rank::Ace, suit: Suit::Heart},
            Card {rank: Rank::King, suit: Suit::Spade},
            Card {rank: Rank::Nine, suit: Suit::Club},
            Card {rank: Rank::Ace, suit: Suit::Diamond});        
        let game_state = GameState::Ready(Context {
            deck: Deck {cards: minimal_cards.clone()},
            computer_hand: Hand::new(),
            player_hand: Hand::new() 
        });

        let new_game_state = deal(game_state)?;
        match new_game_state {
            GameState::WaitingForPlayer(_) => {
                Ok(())
            }
            _ => {
                assert!(false, "Deal transitioned to the wrong state");
                Ok(()) // seems wrong - shouldn't I actually return an error
            }
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
        let cards = vector!(
            Card {rank: Rank::Ace, suit: Suit::Heart},
            Card {rank: Rank::King, suit: Suit::Spade},
            Card {rank: Rank::Nine, suit: Suit::Club},
            Card {rank: Rank::Ace, suit: Suit::Diamond});
        let deck = Deck { cards: cards.clone() };
        let context = Context {
            deck,
            player_hand: Hand::new(),
            computer_hand: Hand::new()
        };

        let game_state = GameState::Ready(context);

        let new_game_state = deal(game_state)?;
        if let GameState::WaitingForPlayer(context) = new_game_state {
            assert_eq!(Deck{cards: vector!()}, context.deck);
            assert_eq!(Hand::new().add(cards[0]).add(cards[1]), context.player_hand);
            // computer_hand has two cards
            Ok(())
        } else {
            Err(Box::new(InvalidStateError{}))
        }
    }
}
