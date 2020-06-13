use crate::hand::Hand;
use crate::deck::Deck;
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


#[derive(Debug, PartialEq)]
enum GameState {
    Ready(Context),
    WaitingForPlayer(Context),
    CheckingPlayerHand(Context),
    PlayerLoses(Context),
    PlayingDealerHand(Context),
}

#[derive(Debug)]
struct InvalidStateErr;

fn deal(state: GameState) -> Result<GameState, InvalidStateErr> {
    match state {
        GameState::Ready(context) => Ok(GameState::WaitingForPlayer(Context::new())),
        _ => Err(InvalidStateErr {}),
    }
}

#[cfg(test)]
mod game_state_machine {
    use super::*;
    use im::vector;

    #[test]
    fn deal_transitions_from_ready_to_waiting_for_player() -> Result<(), InvalidStateErr> {
        let game_state = GameState::Ready(Context::new());

        assert_eq!(deal(game_state)?, GameState::WaitingForPlayer(Context::new()));
        Ok(())
    }

    #[test]
    fn other_transitions_fail() {
        let game_state = GameState::WaitingForPlayer(Context::new());

        let result = deal(game_state);

        assert!(result.is_err(), "deal is only a transition from ready")
    }

    #[test]
    fn deal_gives_the_player_and_computer_hands() -> Result<(), InvalidStateErr> {
        let deck = Deck {cards: vector!() };
        let context = Context {
            deck: deck,
            player_hand: Hand::new(),
            computer_hand: Hand::new()
        };

        let game_state = GameState::Ready(context);
        let new_game_state = deal(game_state)?;

        // player_hand has two cards
        // computer_hand has two cards
        // deck has 4 cards missing

        Ok(())
    }
}
