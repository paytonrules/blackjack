use crate::deck::Deck;
use crate::hand::{DealerHand, Hand, Score};
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum GameState {
    Ready(Context),
    WaitingForPlayer(Context),
    DealerWins(Context),
    PlayerWins(Context),
    Draw(Context)
}

impl GameState {
    fn new() -> Self {
        let shuffled_deck = Deck::standard_deck().shuffle();
        let context = Context::new(shuffled_deck);
        GameState::Ready(context)
    }
}

#[derive(Debug, PartialEq)]
struct Context {
    deck: Deck,
    player_hand: Hand,
    computer_hand: DealerHand,
}

impl Context {
    fn empty() -> Self {
        Context {
            deck: Deck::new(),
            player_hand: Hand::new(),
            computer_hand: DealerHand::new(),
        }
    }

    fn new(deck: Deck) -> Self {
        Context {
            deck,
            player_hand: Hand::new(),
            computer_hand: DealerHand::new(),
        }
    }

    fn double_blackjack(&self) -> bool {
        self.player_blackjack() && self.dealer_blackjack()
    }

    fn player_blackjack(&self) -> bool {
        self.player_hand.score() == Score(21)
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

fn deal(state: &GameState) -> Result<GameState, Box<dyn std::error::Error>> {
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
                deck: new_deck,
            };

            Ok(match new_context {
                _ if new_context.double_blackjack() => GameState::Draw(new_context),
                _ if new_context.dealer_blackjack() => GameState::DealerWins(new_context),
                _ if new_context.player_blackjack() => GameState::PlayerWins(new_context),
                _ => GameState::WaitingForPlayer(new_context)
            })
        }
        _ => Err(Box::new(InvalidStateError {})),
    }
}

#[cfg(test)]
mod game_state_machine {
    use super::*;
    use crate::deck::{Card, Rank, Suit};
    use im::{vector, Vector};

    fn cards(ranks: Vector<Rank>) -> Vector<Card> {
        ranks
            .iter()
            .map(|rank| Card {
                rank: *rank,
                suit: Suit::Heart,
            })
            .collect()
    }

    fn minimal_cards() -> Vector<Card> {
        cards(vector!(Rank::Nine, Rank::Ace, Rank::Nine, Rank::Ace))
    }

    #[test]
    fn deal_transitions_from_ready_to_waiting_for_player() -> Result<(), Box<dyn Error>> {
        let game_state = GameState::Ready(Context::new(Deck::new_with_cards(
            minimal_cards(),
        )));

        let new_game_state = deal(&game_state)?;
        match new_game_state {
            GameState::WaitingForPlayer(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }

    #[test]
    fn other_transitions_fail() {
        let game_state = GameState::WaitingForPlayer(Context::empty());

        let result = deal(&game_state);

        assert!(result.is_err(), "deal is only a transition from ready")
    }

    #[test]
    fn deal_gives_the_player_and_computer_hands() -> Result<(), Box<dyn Error>> {
        let cards = minimal_cards();
        let context = Context::new(Deck::new_with_cards(cards.clone()));
        let game_state = GameState::Ready(context);

        if let GameState::WaitingForPlayer(context) = deal(&game_state)? {
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
        let dealer_blackjack_hand = cards(vector!(Rank::Two, Rank::Two, Rank::Ace, Rank::Ten));
        let context = Context::new(Deck::new_with_cards(dealer_blackjack_hand));
        let game_state = GameState::Ready(context);

        let new_state = deal(&game_state)?;
        match new_state {
            GameState::DealerWins(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }

    #[test]
    fn deal_keeps_the_non_dealt_cards_in_the_deck() -> Result<(), Box<dyn Error>> {
        let typical_hand = cards(
            vector!(Rank::Two, Rank::Two, Rank::Two, Rank::Ten, Rank::Nine));
        let context = Context::new(Deck::new_with_cards(typical_hand));

        let game_state = GameState::Ready(context);

        if let GameState::WaitingForPlayer(context) = deal(&game_state)? {
            assert_eq!(Deck::new_with_cards(cards(vector!(Rank::Nine))), context.deck);
            Ok(())
        } else {
            panic!("Deal transitioned to the wrong state!");
        }
    }

    #[test]
    fn dealer_has_blackjack_and_player_has_blackjack_leads_to_draw() -> Result<(), Box<dyn Error>> {
        let double_blackjack = cards(
            vector!(Rank::Ace, Rank::Ten, Rank::Ace, Rank::Ten));
        let context = Context::new(Deck::new_with_cards(double_blackjack));

        let new_state = deal(&GameState::Ready(context))?;

        match new_state {
            GameState::Draw(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }

    #[test]
    fn player_wins_with_blackjack() ->  Result<(), Box<dyn Error>> {
        let player_blackjack = cards(
            vector!(Rank::Ace, Rank::Ten, Rank::Ace, Rank::Ace));
        let context = Context::new(Deck::new_with_cards(player_blackjack));

        let new_state = deal(&GameState::Ready(context))?;

        match new_state {
            GameState::PlayerWins(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }
}
