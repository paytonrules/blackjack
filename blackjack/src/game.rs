use crate::deck::{Card, Deck};
use crate::hand::{DealerHand, Hand, Score};
use im::{vector, Vector};
use std::error::Error;
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    NewHand(Hand, DealerHand),
    NewPlayerCard(Card),
    NewDealerCards(Vector<Card>),
    PlayerWins,
    DealerWins,
    Draw,
    ShowDealerHoleCard(Card),
}

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

const BLACKJACK: Score = Score(21);

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
        self.player_score() == BLACKJACK
    }

    fn dealer_blackjack(&self) -> bool {
        self.dealer_score() == BLACKJACK
    }

    fn player_score(&self) -> Score {
        self.player_hand.score()
    }

    fn dealer_score(&self) -> Score {
        self.dealer_hand.score()
    }

    fn player_busts(&self) -> bool {
        self.player_score() > BLACKJACK
    }

    fn dealer_busts(&self) -> bool {
        self.dealer_score() > BLACKJACK
    }

    fn player_wins(&self) -> bool {
        self.player_score() > self.dealer_score() || self.dealer_busts()
    }

    fn dealer_wins(&self) -> bool {
        self.dealer_score() > self.player_score() && !self.dealer_busts()
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

#[derive(Debug)]
struct NotFoundError;

impl Error for NotFoundError {}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find object")
    }
}

pub fn deal(state: &GameState) -> Result<(GameState, Vector<Action>), Box<dyn Error>> {
    match state {
        GameState::Ready(context) => {
            let new_context = context.deal_initial_hands()?;

            Ok(match new_context {
                _ if new_context.double_blackjack() => {
                    let actions = vector![
                        Action::Draw,
                        Action::ShowDealerHoleCard(
                            new_context.dealer_hand.hole_card().unwrap().clone()
                        ),
                        new_hand_action(&new_context)
                    ];
                    (GameState::Draw(new_context), actions)
                }
                _ if new_context.dealer_blackjack() => {
                    let actions = vector![
                        Action::DealerWins,
                        Action::ShowDealerHoleCard(
                            new_context.dealer_hand.hole_card().unwrap().clone()
                        ),
                        new_hand_action(&new_context)
                    ];
                    (GameState::DealerWins(new_context), actions)
                }
                _ if new_context.player_blackjack() => {
                    let actions = vector![
                        Action::PlayerWins,
                        Action::ShowDealerHoleCard(
                            new_context.dealer_hand.hole_card().unwrap().clone()
                        ),
                        new_hand_action(&new_context)
                    ];
                    (GameState::PlayerWins(new_context), actions)
                }
                _ => {
                    let actions = vector![new_hand_action(&new_context)];

                    (GameState::WaitingForPlayer(new_context), actions)
                }
            })
        }
        GameState::DealerWins(_) | GameState::PlayerWins(_) | GameState::Draw(_) => {
            let start = GameState::Ready(Context::new_hand());
            deal(&start)
        }
        _ => Err(Box::new(InvalidStateError {})),
    }
}

pub fn hit(state: &GameState) -> Result<(GameState, Vector<Action>), Box<dyn Error>> {
    if let GameState::WaitingForPlayer(context) = state {
        let new_context = context.deal_player_card()?;
        let dealt_card = new_context
            .player_hand
            .cards()
            .last()
            .ok_or(NotFoundError {})?
            .clone();

        Ok(match new_context {
            _ if new_context.player_blackjack() => {
                let (final_state, mut actions) = stand(&GameState::WaitingForPlayer(new_context))?;
                actions.push_front(Action::NewPlayerCard(dealt_card));
                (final_state, actions)
            }
            _ if new_context.player_busts() => {
                let hole_card = new_context.dealer_hand.hole_card().unwrap().clone();

                (
                    GameState::DealerWins(new_context),
                    vector![
                        Action::NewPlayerCard(dealt_card),
                        Action::DealerWins,
                        Action::ShowDealerHoleCard(hole_card)
                    ],
                )
            }
            _ => (
                GameState::WaitingForPlayer(new_context),
                vector![Action::NewPlayerCard(dealt_card)],
            ),
        })
    } else {
        Err(Box::new(InvalidStateError {}))
    }
}

pub fn stand(state: &GameState) -> Result<(GameState, Vector<Action>), Box<dyn Error>> {
    match state {
        GameState::WaitingForPlayer(context) => {
            let new_context = context.play_dealer_hand()?;
            let next_dealer_cards = new_context.dealer_hand.cards().skip(2);
            let mut actions = if next_dealer_cards.len() > 0 {
                vector![
                    Action::ShowDealerHoleCard(
                        new_context.dealer_hand.hole_card().unwrap().clone()
                    ),
                    Action::NewDealerCards(next_dealer_cards)
                ]
            } else {
                vector![Action::ShowDealerHoleCard(
                    new_context.dealer_hand.hole_card().unwrap().clone()
                )]
            };
            Ok(match new_context {
                _ if new_context.dealer_wins() => {
                    actions.push_front(Action::DealerWins);
                    (GameState::DealerWins(new_context), actions)
                }
                _ if new_context.player_wins() => {
                    actions.push_front(Action::PlayerWins);
                    (GameState::PlayerWins(new_context), actions)
                }
                _ if new_context.draw() => {
                    actions.push_front(Action::Draw);
                    (GameState::Draw(new_context), actions)
                }
                _ => (GameState::WaitingForPlayer(new_context), Vector::new()),
            })
        }
        _ => Err(Box::new(InvalidStateError {})),
    }
}

fn new_hand_action(context: &Context) -> Action {
    Action::NewHand(context.player_hand.clone(), context.dealer_hand.clone())
}

#[cfg(test)]
mod game_state_machine {
    use super::*;
    use crate::deck::{Card, Rank, Suit};
    use im::{vector, HashSet, Vector};

    #[derive(Debug)]
    struct InvalidActionError {}

    impl Error for InvalidActionError {}

    impl fmt::Display for InvalidActionError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Expected Action was not the actual one")
        }
    }

    impl Deck {
        pub fn new() -> Self {
            Deck { cards: vector!() }
        }
    }

    impl Context {
        fn empty() -> Self {
            Context::new(Deck::new())
        }

        fn new_with_cards(cards: Vector<Card>) -> Self {
            Context::new(Deck::new_with_cards(cards))
        }
    }

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
        assert_eq!(*new_context.dealer_hand.hole_card().unwrap(), cards[1]);
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

        let (new_game_state, _) = deal(&game_state)?;
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

        if let (GameState::WaitingForPlayer(context), _) = deal(&game_state)? {
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

    fn assert_actions_contains_new_hand(
        actions: &Vector<Action>,
        cards: &Vector<Card>,
    ) -> Result<(), Box<dyn Error>> {
        let new_hand = actions
            .iter()
            .find(|action| match action {
                Action::NewHand(_, _) => true,
                _ => false,
            })
            .ok_or(NotFoundError {})?;

        match new_hand {
            Action::NewHand(player_hand, dealer_hand) => {
                assert_eq!(&Hand::new().add(cards[0]).add(cards[2]), player_hand);
                assert_eq!(&DealerHand::new().add(cards[1]).add(cards[3]), dealer_hand);
                Ok(())
            }
            _ => Err(Box::new(InvalidActionError {})),
        }
    }

    #[test]
    fn deal_also_issues_a_new_hand_action() -> Result<(), Box<dyn Error>> {
        let cards = minimal_cards();
        let context = Context::new_with_cards(cards.clone());
        let game_state = GameState::Ready(context);

        let (_, actions) = deal(&game_state)?;
        assert_eq!(actions.len(), 1);
        assert_actions_contains_new_hand(&actions, &cards)
    }

    #[test]
    fn deal_goes_to_dealer_won_when_dealer_has_blackjack() -> Result<(), Box<dyn Error>> {
        let dealer_blackjack_hand = cards(vector!(Rank::Two, Rank::Ace, Rank::Two, Rank::Ten));
        let context = Context::new_with_cards(dealer_blackjack_hand);
        let game_state = GameState::Ready(context);

        let (new_state, _) = deal(&game_state)?;
        match new_state {
            GameState::DealerWins(_) => Ok(()),
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn deal_has_three_actions_when_dealer_wins_with_blackjack() -> Result<(), Box<dyn Error>> {
        let dealer_blackjack_hand = cards(vector!(Rank::Two, Rank::Ace, Rank::Two, Rank::Ten));
        let context = Context::new_with_cards(dealer_blackjack_hand.clone());
        let game_state = GameState::Ready(context);

        let (_, actions) = deal(&game_state)?;

        assert_eq!(3, actions.len());
        assert!(actions.contains(&Action::DealerWins));
        assert!(actions.contains(&Action::ShowDealerHoleCard(dealer_blackjack_hand[1])));
        assert_actions_contains_new_hand(&actions, &dealer_blackjack_hand)
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

        if let (GameState::WaitingForPlayer(context), _) = deal(&game_state)? {
            assert_eq!(
                Deck::new_with_cards(cards(vector!(Rank::Nine))),
                context.deck
            );
            Ok(())
        } else {
            Err(Box::new(InvalidStateError))
        }
    }

    #[test]
    fn dealer_has_blackjack_and_player_has_blackjack_leads_to_draw() -> Result<(), Box<dyn Error>> {
        let double_blackjack = cards(vector!(Rank::Ace, Rank::Ace, Rank::Ten, Rank::Ten));
        let context = Context::new_with_cards(double_blackjack);

        let (new_state, _) = deal(&GameState::Ready(context))?;

        match new_state {
            GameState::Draw(_) => Ok(()),
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn dealer_has_blackjack_and_player_has_blackjack_returns_new_hand_and_draw_actions(
    ) -> Result<(), Box<dyn Error>> {
        let double_blackjack = cards(vector!(Rank::Ace, Rank::Ace, Rank::Ten, Rank::Ten));
        let context = Context::new_with_cards(double_blackjack.clone());

        let (_, actions) = deal(&GameState::Ready(context))?;
        assert_eq!(3, actions.len());
        assert!(actions.contains(&Action::Draw));
        assert!(actions.contains(&Action::ShowDealerHoleCard(double_blackjack[1])));
        assert_actions_contains_new_hand(&actions, &double_blackjack)
    }

    #[test]
    fn player_wins_with_blackjack() -> Result<(), Box<dyn Error>> {
        let player_blackjack = cards(vector!(Rank::Ace, Rank::Ace, Rank::Ten, Rank::Ace));
        let context = Context::new_with_cards(player_blackjack);

        let (new_state, _) = deal(&GameState::Ready(context))?;

        match new_state {
            GameState::PlayerWins(_) => Ok(()),
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn player_wins_with_blackjack_has_player_wins_action() -> Result<(), Box<dyn Error>> {
        let player_blackjack = cards(vector!(Rank::Ace, Rank::Ace, Rank::Ten, Rank::Ace));
        let context = Context::new_with_cards(player_blackjack.clone());

        let (_, actions) = deal(&GameState::Ready(context))?;
        assert_eq!(3, actions.len());
        assert!(actions.contains(&Action::PlayerWins));
        assert!(actions.contains(&Action::ShowDealerHoleCard(player_blackjack[1])));
        assert_actions_contains_new_hand(&actions, &player_blackjack)
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
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_hits, actions) = hit(&game)?;

        match player_hits {
            GameState::WaitingForPlayer(context) => {
                assert_eq!(context.player_score(), Score(19));
                assert_eq!(actions.len(), 1);
                if let Action::NewPlayerCard(new_card) =
                    actions.front().ok_or(InvalidActionError {})?
                {
                    assert_eq!(&cards[4], new_card);
                }
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError {})),
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
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_hits, actions) = hit(&game)?;

        match player_hits {
            GameState::DealerWins(context) => {
                assert_eq!(context.player_score(), Score(24));
                assert_eq!(actions.len(), 3);
                assert!(actions.contains(&Action::DealerWins));
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                let new_card_action = actions
                    .iter()
                    .find(|action| match action {
                        Action::NewPlayerCard(_) => true,
                        _ => false,
                    })
                    .ok_or(NotFoundError)?;
                if let Action::NewPlayerCard(card) = new_card_action {
                    assert_eq!(&cards[4], card);
                }

                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn player_hits_and_gets_blackjack_transitions_to_stand() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ten,
            Rank::Ten,
            Rank::Ten,
            Rank::Seven,
            Rank::Ace
        ));
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_hits, actions) = hit(&game)?;

        match player_hits {
            GameState::PlayerWins(context) => {
                assert_eq!(context.player_score(), BLACKJACK);
                assert_eq!(actions.len(), 3);
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                assert!(actions.contains(&Action::PlayerWins));
                let new_card_action = actions
                    .iter()
                    .find(|action| match action {
                        Action::NewPlayerCard(_) => true,
                        _ => false,
                    })
                    .ok_or(NotFoundError)?;
                if let Action::NewPlayerCard(card) = new_card_action {
                    assert_eq!(&cards[4], card);
                }
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn player_stands_with_twenty_and_dealer_has_seventeen_player_wins() -> Result<(), Box<dyn Error>>
    {
        let cards = cards(vector!(Rank::Ten, Rank::Ten, Rank::Ten, Rank::Seven));
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_stands, actions) = stand(&game)?;

        match player_stands {
            GameState::PlayerWins(context) => {
                assert_eq!(context.player_score(), Score(20));
                assert_eq!(context.dealer_score(), Score(17));
                assert_eq!(actions.len(), 2);
                assert!(actions.contains(&Action::PlayerWins));
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn player_stands_with_seventeen_and_dealer_has_twenty_dealer_wins() -> Result<(), Box<dyn Error>>
    {
        let cards = cards(vector!(Rank::Ten, Rank::Ten, Rank::Seven, Rank::Ten));
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_stands, actions) = stand(&game)?;

        match player_stands {
            GameState::DealerWins(context) => {
                assert_eq!(context.player_score(), Score(17));
                assert_eq!(context.dealer_score(), Score(20));
                assert_eq!(actions.len(), 2);
                assert!(actions.contains(&Action::DealerWins));
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn player_stands_with_seventeen_and_dealer_hits_to_win() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ten,
            Rank::Ten,
            Rank::Seven,
            Rank::Six,
            Rank::Three
        ));
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_stands, actions) = stand(&game)?;

        match player_stands {
            GameState::DealerWins(context) => {
                assert_eq!(context.player_score(), Score(17));
                assert_eq!(context.dealer_score(), Score(19));
                assert_eq!(actions.len(), 3);
                assert!(actions.contains(&Action::DealerWins));
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                assert_new_dealer_cards_are(actions, vector![Rank::Three]);
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn player_stands_with_twenty_and_dealer_has_twenty_draw() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(Rank::Ten, Rank::Ten, Rank::Ten, Rank::Ten));
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_stands, actions) = stand(&game)?;

        match player_stands {
            GameState::Draw(context) => {
                assert_eq!(context.player_score(), Score(20));
                assert_eq!(context.dealer_score(), Score(20));
                assert_eq!(actions.len(), 2);
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                assert!(actions.contains(&Action::Draw));
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    fn assert_new_dealer_cards_are(actions: Vector<Action>, expected_cards: Vector<Rank>) {
        let new_cards: Vector<Rank> = actions
            .iter()
            .filter_map(|action| match action {
                Action::NewDealerCards(cards) => Some(cards.clone()),
                _ => None,
            })
            .flatten()
            .map(|card| card.rank)
            .collect();

        assert_eq!(new_cards, expected_cards);
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
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_stands, actions) = stand(&game)?;

        match player_stands {
            GameState::PlayerWins(context) => {
                assert_eq!(context.dealer_score(), Score(17));
                assert_eq!(actions.len(), 3);
                assert!(actions.contains(&Action::PlayerWins));
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                assert_new_dealer_cards_are(actions, vector![Rank::Ace]);
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
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
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_stands, actions) = stand(&game)?;

        match player_stands {
            GameState::PlayerWins(context) => {
                assert_eq!(context.dealer_score(), Score(17));
                assert_eq!(actions.len(), 3);
                assert!(actions.contains(&Action::PlayerWins));
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                assert_new_dealer_cards_are(actions, vector![Rank::Ace, Rank::Four]);
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
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

        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (player_hits, actions) = hit(&game)?;

        match player_hits {
            GameState::Draw(context) => {
                assert_eq!(context.dealer_score(), BLACKJACK);
                assert_eq!(context.player_score(), BLACKJACK);
                assert_eq!(actions.len(), 4);
                assert!(actions.contains(&Action::Draw));
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                assert!(actions.contains(&Action::NewPlayerCard(Card {
                    rank: Rank::Ace,
                    suit: Suit::Heart
                })));
                assert_new_dealer_cards_are(actions, vector![Rank::Nine]);
                Ok(())
            }
            _ => Err(Box::new(InvalidStateError)),
        }
    }

    #[test]
    fn dealer_loses_if_they_bust() -> Result<(), Box<dyn Error>> {
        let cards = cards(vector!(
            Rank::Ten,
            Rank::Ten,
            Rank::Ten,
            Rank::Six,
            Rank::Six
        ));
        let context = Context::new_with_cards(cards.clone());
        let (game, _) = deal(&GameState::Ready(context))?;

        let (dealer_busts, actions) = stand(&game)?;

        match dealer_busts {
            GameState::PlayerWins(context) => {
                assert_eq!(context.dealer_score(), Score(22));
                assert_eq!(context.player_score(), Score(20));
                assert_eq!(actions.len(), 3);
                assert!(actions.contains(&Action::ShowDealerHoleCard(cards[1])));
                assert!(actions.contains(&Action::PlayerWins));
                assert_new_dealer_cards_are(actions, vector![Rank::Six]);
                Ok(())
            }
            GameState::DealerWins(_) => Err(Box::new(InvalidStateError)),
            _ => Err(Box::new(InvalidStateError)),
        }
    }
}
