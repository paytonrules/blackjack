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
    Draw(Context),
}

impl GameState {
    fn new() -> Self {
        let shuffled_deck = Deck::standard_deck().shuffle();
        let context = Context::new(shuffled_deck);
        GameState::Ready(context)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Context {
    deck: Deck,
    player_hand: Hand,
    dealer_hand: DealerHand,
}

impl Context {
    fn empty() -> Self {
        Context {
            deck: Deck::new(),
            player_hand: Hand::new(),
            dealer_hand: DealerHand::new(),
        }
    }

    fn new(deck: Deck) -> Self {
        Context {
            deck,
            player_hand: Hand::new(),
            dealer_hand: DealerHand::new(),
        }
    }

    fn deal_initial_hands(&self) -> Result<Context, Box<dyn std::error::Error>> {
        let (new_deck, first_card) = self.deck.deal()?;
        let (new_deck, second_card) = new_deck.deal()?;
        let (new_deck, third_card) = new_deck.deal()?;
        let (new_deck, fourth_card) = new_deck.deal()?;
        let player_hand = Hand::new().add(first_card).add(second_card);
        let dealer_hand = DealerHand::new().add(third_card).add(fourth_card);

        Ok(Context {
            player_hand,
            dealer_hand,
            deck: new_deck,
        })
    }

    fn deal_player_card(&self) -> Result<Context, Box<dyn std::error::Error>> {
        let (deck, card) = self.deck.deal()?;
        let player_hand = self.player_hand.add(card);

        Ok(Context {
            player_hand,
            dealer_hand: self.dealer_hand.clone(),
            deck,
        })
    }

    fn play_dealer_hand(&self) -> Result<Context, Box<dyn std::error::Error>> {
        let mut new_context = self.clone();
        while new_context.dealer_score() < Score(17) {
            let (deck, card) = new_context.deck.deal()?;
            new_context.deck = deck;
            new_context.dealer_hand = new_context.dealer_hand.add(card);
        }
        Ok(new_context)
    }

    fn double_blackjack(&self) -> bool {
        self.player_blackjack() && self.dealer_blackjack()
    }

    fn player_blackjack(&self) -> bool {
        self.player_score() == Score(21)
    }

    fn dealer_blackjack(&self) -> bool {
        self.dealer_score() == Score(21)
    }

    fn player_score(&self) -> Score {
        self.player_hand.score()
    }

    fn dealer_score(&self) -> Score {
        self.dealer_hand.score()
    }

    fn player_busts(&self) -> bool {
        self.player_score() > Score(21)
    }

    fn player_wins(&self) -> bool {
        self.player_score() > self.dealer_score()
    }

    fn dealer_wins(&self) -> bool {
        self.dealer_score() > self.player_score()
    }

    fn draw(&self) -> bool {
        self.player_score() == self.dealer_score()
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
            let new_context = context.deal_initial_hands()?;

            Ok(match new_context {
                _ if new_context.double_blackjack() => GameState::Draw(new_context),
                _ if new_context.dealer_blackjack() => GameState::DealerWins(new_context),
                _ if new_context.player_blackjack() => GameState::PlayerWins(new_context),
                _ => GameState::WaitingForPlayer(new_context),
            })
        }
        _ => Err(Box::new(InvalidStateError {})),
    }
}

fn hit(state: &GameState) -> Result<GameState, Box<dyn std::error::Error>> {
    match state {
        GameState::WaitingForPlayer(context) => {
            let new_context = context.deal_player_card()?;
            Ok(match new_context {
                _ if new_context.player_busts() => GameState::DealerWins(new_context),
                _ => GameState::WaitingForPlayer(new_context),
            })
        }
        _ => Err(Box::new(InvalidStateError {})),
    }
}

fn stand(state: &GameState) -> Result<GameState, Box<dyn std::error::Error>> {
    match state {
        GameState::WaitingForPlayer(context) => {
            let new_context = context.play_dealer_hand()?;
            Ok(match new_context {
                _ if new_context.player_wins() => GameState::PlayerWins(new_context),
                _ if new_context.dealer_wins() => GameState::DealerWins(new_context),
                _ if new_context.draw() => GameState::Draw(new_context),
                _ => GameState::WaitingForPlayer(new_context),
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
        let game_state = GameState::Ready(Context::new(Deck::new_with_cards(minimal_cards())));

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
    fn deal_gives_the_player_and_dealer_hands() -> Result<(), Box<dyn Error>> {
        let cards = minimal_cards();
        let context = Context::new(Deck::new_with_cards(cards.clone()));
        let game_state = GameState::Ready(context);

        if let GameState::WaitingForPlayer(context) = deal(&game_state)? {
            assert_eq!(Deck::new(), context.deck);
            assert_eq!(Hand::new().add(cards[0]).add(cards[1]), context.player_hand);
            assert_eq!(
                DealerHand::new().add(cards[2]).add(cards[3]),
                context.dealer_hand
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
        let typical_hand = cards(vector!(
            Rank::Two,
            Rank::Two,
            Rank::Two,
            Rank::Ten,
            Rank::Nine
        ));
        let context = Context::new(Deck::new_with_cards(typical_hand));

        let game_state = GameState::Ready(context);

        if let GameState::WaitingForPlayer(context) = deal(&game_state)? {
            assert_eq!(
                Deck::new_with_cards(cards(vector!(Rank::Nine))),
                context.deck
            );
            Ok(())
        } else {
            panic!("Deal transitioned to the wrong state!");
        }
    }

    #[test]
    fn dealer_has_blackjack_and_player_has_blackjack_leads_to_draw() -> Result<(), Box<dyn Error>> {
        let double_blackjack = cards(vector!(Rank::Ace, Rank::Ten, Rank::Ace, Rank::Ten));
        let context = Context::new(Deck::new_with_cards(double_blackjack));

        let new_state = deal(&GameState::Ready(context))?;

        match new_state {
            GameState::Draw(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }

    #[test]
    fn player_wins_with_blackjack() -> Result<(), Box<dyn Error>> {
        let player_blackjack = cards(vector!(Rank::Ace, Rank::Ten, Rank::Ace, Rank::Ace));
        let context = Context::new(Deck::new_with_cards(player_blackjack));

        let new_state = deal(&GameState::Ready(context))?;

        match new_state {
            GameState::PlayerWins(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }

    #[test]
    fn player_hits_hand_and_gets_one_card() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ace,
            Rank::Two,
            Rank::Ten,
            Rank::Ten,
            Rank::Four
        ));
        let context = Context::new(Deck::new_with_cards(cards));
        let game = deal(&GameState::Ready(context))?;

        let player_hits = hit(&game)?;

        match player_hits {
            GameState::WaitingForPlayer(context) => {
                assert_eq!(context.player_score(), Score(17));
                Ok(())
            }
            _ => panic!("game state should not have transitioned!"),
        }
    }

    #[test]
    fn player_hits_and_busts_transitions_to_player_lose() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ten,
            Rank::Ten,
            Rank::Ace,
            Rank::Ace,
            Rank::Four
        ));
        let context = Context::new(Deck::new_with_cards(cards));
        let game = deal(&GameState::Ready(context))?;

        let player_hits = hit(&game)?;

        match player_hits {
            GameState::DealerWins(context) => {
                assert_eq!(context.player_score(), Score(24));
                Ok(())
            }
            _ => panic!("game state transitioned to wrong state"),
        }
    }

    #[test]
    fn player_stands_with_twenty_and_dealer_has_seventeen_player_wins() -> Result<(), Box<dyn Error>>
    {
        let cards = cards(vector!(Rank::Ten, Rank::Ten, Rank::Ten, Rank::Seven));
        let context = Context::new(Deck::new_with_cards(cards));
        let game = deal(&GameState::Ready(context))?;

        let player_stands = stand(&game)?;

        match player_stands {
            GameState::PlayerWins(context) => {
                assert_eq!(context.player_score(), Score(20));
                assert_eq!(context.dealer_score(), Score(17));
                Ok(())
            }
            _ => panic!("game state transitioned to wrong state"),
        }
    }

    #[test]
    fn player_stands_with_seventeen_and_dealer_has_twenty_dealer_wins() -> Result<(), Box<dyn Error>>
    {
        let cards = cards(vector!(Rank::Ten, Rank::Seven, Rank::Ten, Rank::Ten));
        let context = Context::new(Deck::new_with_cards(cards));
        let game = deal(&GameState::Ready(context))?;

        let player_stands = stand(&game)?;

        match player_stands {
            GameState::DealerWins(context) => {
                assert_eq!(context.player_score(), Score(17));
                assert_eq!(context.dealer_score(), Score(20));
                Ok(())
            }
            _ => panic!("game state transitioned to wrong state"),
        }
    }

    #[test]
    fn player_stands_with_twenty_and_dealer_has_twenty_draw() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(Rank::Ten, Rank::Ten, Rank::Ten, Rank::Ten));
        let context = Context::new(Deck::new_with_cards(cards));
        let game = deal(&GameState::Ready(context))?;

        let player_stands = stand(&game)?;

        match player_stands {
            GameState::Draw(context) => {
                assert_eq!(context.player_score(), Score(20));
                assert_eq!(context.dealer_score(), Score(20));
                Ok(())
            }
            _ => panic!("game state transitioned to wrong state"),
        }
    }

    #[test]
    fn dealer_hits_if_under_seventeen() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ten,
            Rank::Ten,
            Rank::Ten,
            Rank::Six,
            Rank::Ace
        ));
        let context = Context::new(Deck::new_with_cards(cards));
        let game = deal(&GameState::Ready(context))?;

        let player_stands = stand(&game)?;

        match player_stands {
            GameState::PlayerWins(context) => {
                assert_eq!(context.dealer_score(), Score(17));
                Ok(())
            }
            _ => panic!("game state transitioned to wrong state"),
        }
    }

    #[test]
    fn dealer_hits_until_they_get_to_seventeen() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ten,
            Rank::Ten,
            Rank::Ten,
            Rank::Two,
            Rank::Ace,
            Rank::Four
        ));
        let context = Context::new(Deck::new_with_cards(cards));
        let game = deal(&GameState::Ready(context))?;

        let player_stands = stand(&game)?;

        match player_stands {
            GameState::PlayerWins(context) => {
                assert_eq!(context.dealer_score(), Score(17));
                Ok(())
            }
            _ => panic!("game state transitioned to wrong state"),
        }
    }
}
