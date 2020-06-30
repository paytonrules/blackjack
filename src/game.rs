use crate::deck::{Card, Deck};
use crate::hand::{DealerHand, Hand, Score};
use im::Vector;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum GameState {
    Ready(Context),
    WaitingForPlayer(Context),
    DealerWins(Context),
    PlayerWins(Context),
    Draw(Context),
}

impl GameState {
    pub fn new() -> Self {
        GameState::Ready(Context::new_hand())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    deck: Deck,
    pub player_hand: Hand,
    pub dealer_hand: DealerHand,
}

impl Context {
    fn new(deck: Deck) -> Self {
        Context {
            deck,
            player_hand: Hand::new(),
            dealer_hand: DealerHand::new(),
        }
    }

    fn empty() -> Self {
        Context::new(Deck::new())
    }

    fn new_with_cards(cards: Vector<Card>) -> Self {
        Context::new(Deck::new_with_cards(cards))
    }

    fn new_hand() -> Self {
        Context::new(Deck::standard_deck().shuffle())
    }

    fn deal_initial_hands(&self) -> Result<Context, Box<dyn Error>> {
        let (new_deck, first_card) = self.deck.deal()?;
        let (new_deck, second_card) = new_deck.deal()?;
        let (new_deck, third_card) = new_deck.deal()?;
        let (new_deck, fourth_card) = new_deck.deal()?;
        let player_hand = Hand::new().add(first_card).add(third_card);
        let dealer_hand = DealerHand::new().add(second_card).add(fourth_card);

        Ok(Context {
            player_hand,
            dealer_hand,
            deck: new_deck,
        })
    }

    fn deal_player_card(&self) -> Result<Context, Box<dyn Error>> {
        let (deck, card) = self.deck.deal()?;
        let player_hand = self.player_hand.add(card);

        Ok(Context {
            player_hand,
            dealer_hand: self.dealer_hand.clone(),
            deck,
        })
    }

    fn play_dealer_hand(&self) -> Result<Context, Box<dyn Error>> {
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

pub fn deal(state: &GameState) -> Result<GameState, Box<dyn Error>> {
    match state {
        GameState::Ready(context) => {
            let new_context = context.deal_initial_hands()?;

            Ok(match new_context {
                _ if new_context.double_blackjack() => GameState::Draw(new_context),
                _ if new_context.dealer_blackjack() => GameState::DealerWins(new_context),
                _ if new_context.player_blackjack() => GameState::PlayerWins(new_context),
                _ => GameState::WaitingForPlayer(new_context),
            })
        },
        GameState::DealerWins(_) | GameState::PlayerWins(_) | GameState::Draw(_) => {
            let start = GameState::Ready(Context::new_hand());
            deal(&start)
        },
        _ => Err(Box::new(InvalidStateError {})),
    }
}

pub fn hit(state: &GameState) -> Result<GameState, Box<dyn Error>> {
    match state {
        GameState::WaitingForPlayer(context) => {
            let new_context = context.deal_player_card()?;

            Ok(match new_context {
                _ if new_context.player_blackjack() => {
                    stand(&GameState::WaitingForPlayer(new_context))?
                }
                _ if new_context.player_busts() => GameState::DealerWins(new_context),
                _ => GameState::WaitingForPlayer(new_context),
            })
        }
        _ => Err(Box::new(InvalidStateError {})),
    }
}

pub fn stand(state: &GameState) -> Result<GameState, Box<dyn Error>> {
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
    use crate::deck::{Rank, Suit};
    use im::{vector, HashSet};

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
    fn context_deal_intial_hand_deals_in_the_right_order() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(Rank::Ace, Rank::Two, Rank::Three, Rank::Four));
        let context = Context::new_with_cards(cards.clone());

        let new_context = context.deal_initial_hands()?;

        assert_eq!(
            new_context.player_hand,
            Hand::new().add(cards[0]).add(cards[2])
        );
        assert_eq!(
            new_context.dealer_hand,
            DealerHand::new().add(cards[1]).add(cards[3])
        );
        assert_eq!(*new_context.dealer_hand.hidden_card().unwrap(), cards[1]);
        Ok(())
    }

    #[test]
    fn context_new_hand_creates_new_context_with_new_shuffled_deck() {
        let context = Context::new_hand();

        let full_deck = Deck::standard_deck();
        assert_ne!(context.deck.cards, full_deck.cards);

        let shuffled_deck_set = full_deck.cards.into_iter().collect::<HashSet<Card>>();
        let new_deck_set = context.deck.cards.into_iter().collect::<HashSet<Card>>();
        assert_eq!(new_deck_set, shuffled_deck_set);

        assert_eq!(context.player_hand, Hand::new());
        assert_eq!(context.dealer_hand, DealerHand::new());
    }

    #[test]
    fn deal_transitions_from_ready_to_waiting_for_player() -> Result<(), Box<dyn Error>> {
        let game_state = GameState::Ready(Context::new_with_cards(minimal_cards()));

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
        let context = Context::new_with_cards(cards.clone());
        let game_state = GameState::Ready(context);

        if let GameState::WaitingForPlayer(context) = deal(&game_state)? {
            assert_eq!(Deck::new(), context.deck);
            assert_eq!(Hand::new().add(cards[0]).add(cards[2]), context.player_hand);
            assert_eq!(
                DealerHand::new().add(cards[1]).add(cards[3]),
                context.dealer_hand
            );
            Ok(())
        } else {
            Err(Box::new(InvalidStateError {}))
        }
    }

    #[test]
    fn deal_goes_to_dealer_won_when_dealer_has_blackjack() -> Result<(), Box<dyn Error>> {
        let dealer_blackjack_hand = cards(vector!(Rank::Two, Rank::Ace, Rank::Two, Rank::Ten));
        let context = Context::new_with_cards(dealer_blackjack_hand);
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
        let context = Context::new_with_cards(typical_hand);

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
        let double_blackjack = cards(vector!(Rank::Ace, Rank::Ace, Rank::Ten, Rank::Ten));
        let context = Context::new_with_cards(double_blackjack);

        let new_state = deal(&GameState::Ready(context))?;

        match new_state {
            GameState::Draw(_) => Ok(()),
            _ => panic!("Deal transitioned to the wrong state!"),
        }
    }

    #[test]
    fn player_wins_with_blackjack() -> Result<(), Box<dyn Error>> {
        let player_blackjack = cards(vector!(Rank::Ace, Rank::Ace, Rank::Ten, Rank::Ace));
        let context = Context::new_with_cards(player_blackjack);

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
            Rank::Four,
            Rank::Ten,
            Rank::Four
        ));
        let context = Context::new_with_cards(cards);
        let game = deal(&GameState::Ready(context))?;

        let player_hits = hit(&game)?;

        match player_hits {
            GameState::WaitingForPlayer(context) => {
                assert_eq!(context.player_score(), Score(19));
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
            Rank::Six,
            Rank::Ten,
            Rank::Eight
        ));
        let context = Context::new_with_cards(cards);
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
    fn player_hits_and_gets_blackjack_transitions_to_endgame() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ten,
            Rank::Ten,
            Rank::Ten,
            Rank::Seven,
            Rank::Ace
        ));
        let context = Context::new_with_cards(cards);
        let game = deal(&GameState::Ready(context))?;

        let player_hits = hit(&game)?;

        match player_hits {
            GameState::PlayerWins(context) => {
                assert_eq!(context.player_score(), Score(21));
                Ok(())
            }
            _ => panic!("game state transitioned to wrong state"),
        }
    }

    #[test]
    fn player_stands_with_twenty_and_dealer_has_seventeen_player_wins() -> Result<(), Box<dyn Error>>
    {
        let cards = cards(vector!(Rank::Ten, Rank::Ten, Rank::Ten, Rank::Seven));
        let context = Context::new_with_cards(cards);
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
        let cards = cards(vector!(Rank::Ten, Rank::Ten, Rank::Seven, Rank::Ten));
        let context = Context::new_with_cards(cards);
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
        let context = Context::new_with_cards(cards);
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
        let context = Context::new_with_cards(cards);
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
        let context = Context::new_with_cards(cards);
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
    fn dealer_plays_their_hand_if_player_gets_blackjack_on_hit() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ten,
            Rank::Ten,
            Rank::Ten,
            Rank::Two,
            Rank::Ace,
            Rank::Nine
        ));

        let context = Context::new_with_cards(cards);
        let game = deal(&GameState::Ready(context))?;

        let player_hits = hit(&game)?;

        match player_hits {
            GameState::Draw(context) => {
                assert_eq!(context.dealer_score(), Score(21));
                assert_eq!(context.player_score(), Score(21));
                Ok(())
            }
            _ => panic!("game state transitioned to wrong state"),
        }
    }
}
