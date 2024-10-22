use blackjack::deck::{Card, Rank};
use blackjack::{
    game::{deal, hit, stand, Action, GameState},
    hand::DealerHand,
};
use gdnative::api::{AtlasTexture, RichTextLabel, ToolButton};
use gdnative::prelude::*;
use im::{vector, Vector};
use std::cmp::Ordering;
use thiserror::Error;

#[derive(Debug, Error)]
enum GodotError {
    #[error("{0}")]
    FindNodeFailed(String),
}

fn clear_all_children(node_name: &str, owner: TRef<Node2D>) -> Result<(), GodotError> {
    let parent = get_typed_node::<Node>(node_name, owner)?;
    for var in parent.get_children().iter() {
        let child = var.try_to_object::<Node>();
        child.map(|child| {
            let child = unsafe { child.assume_safe() };
            parent.remove_child(child);
            child.queue_free()
        });
    }
    Ok(())
}

fn get_typed_node<'a, O>(
    name: &str,
    owner: TRef<'a, Node2D, Shared>,
) -> Result<TRef<'a, O>, GodotError>
where
    O: GodotObject + SubClass<Node>,
{
    owner
        .get_node(name)
        .map(|node| unsafe { node.assume_safe() })
        .and_then(|node| node.cast::<O>())
        .ok_or(GodotError::FindNodeFailed(
            "Node either not found or could not be cast to type".to_string(),
        ))
}

fn rank_as_texture_abbreviation(rank: &Rank) -> String {
    match rank {
        Rank::Two
        | Rank::Three
        | Rank::Four
        | Rank::Five
        | Rank::Six
        | Rank::Seven
        | Rank::Eight
        | Rank::Nine
        | Rank::Ten => format!("{}", rank.to_value().0),
        Rank::Ace => "A".to_string(),
        Rank::Jack => "J".to_string(),
        Rank::Queen => "Q".to_string(),
        Rank::King => "K".to_string(),
    }
}

fn texture_path_from_card(card: &Card) -> String {
    format!(
        "res://images/playingCards.card{:?}s{}.atlastex",
        card.suit,
        rank_as_texture_abbreviation(&card.rank)
    )
}

fn show_result_text(owner: TRef<Node2D>, result: &str) {
    get_typed_node::<RichTextLabel>("./Result", owner).map(|node| {
        node.add_text(result);
    });
}

fn clear_result_text(owner: TRef<Node2D>) {
    get_typed_node::<RichTextLabel>("./Result", owner).map(|node| {
        node.clear();
    });
}

fn show_dealer_hole_card(owner: TRef<Node2D>, texture: &str) {
    get_typed_node::<Node2D>("./DealerHand", owner).map(|dealer_hand_node| {
        let resource_loader = ResourceLoader::godot_singleton();
        let sprite = Sprite::new();
        let texture = resource_loader
            .load(texture, "AtlasTexture", false)
            .and_then(|res| res.cast::<AtlasTexture>())
            .expect("Couldn't load atlasTexture texture");
        sprite.set_texture(texture);
        sprite.set_position(Vector2::new(0.0, 0.0));

        let hole_texture = dealer_hand_node.get_child(0).unwrap();
        let hole_texture = unsafe { hole_texture.assume_unique() };
        hole_texture.replace_by(sprite, false);
        hole_texture.queue_free();
    });
}

fn sort_new_card_actions(mut actions: Vector<Action>) -> Vector<Action> {
    actions.sort_by(|a, b| match (a, b) {
        (Action::NewDealerCards(_), Action::NewPlayerCard(_)) => Ordering::Greater,
        (Action::NewPlayerCard(_), Action::NewDealerCards(_)) => Ordering::Less,
        _ => Ordering::Equal,
    });
    actions
}

fn filter_new_card_actions(actions: Vector<Action>) -> Vector<Action> {
    actions
        .iter()
        .filter(|action| match action {
            Action::NewHand(_, _) | Action::NewDealerCards(_) | Action::NewPlayerCard(_) => true,
            _ => false,
        })
        .cloned()
        .collect::<Vector<Action>>()
}

#[derive(Clone)]
struct CardAnimationProperties {
    destination_node: Ref<Node2D>,
    texture_name: String,
}

#[derive(NativeClass)]
#[inherit(Node2D)]
struct Blackjack {
    state: GameState,
    actions: Vector<Action>,
    animations: Vector<CardAnimationProperties>,
}

#[methods]
impl Blackjack {
    fn new(_owner: &Node2D) -> Self {
        Blackjack {
            state: GameState::new(),
            actions: vector![],
            animations: vector![],
        }
    }

    #[export]
    fn _on_new_game_pressed(&mut self, owner: TRef<Node2D>) {
        clear_all_children("./DealerHand", owner);
        clear_all_children("./PlayerHand", owner);
        clear_result_text(owner);

        let (state, actions) = deal(&self.state).expect("Dealing has to work, basically");
        self.state = state;
        self.actions = actions;
    }

    #[export]
    fn _on_stand_pressed(&mut self, _owner: TRef<Node2D>) {
        let (state, actions) = stand(&self.state).expect("You could stand at this point");
        self.state = state;
        self.actions = actions;
    }

    #[export]
    fn _on_hit_pressed(&mut self, _owner: TRef<Node2D>) {
        let (state, actions) = hit(&self.state).expect("You can hit at this point");
        self.state = state;
        self.actions = actions;
    }

    #[export]
    fn _process(&mut self, owner: TRef<Node2D>, _delta: f64) {
        self.process_animations(owner);
        if self.animations.len() <= 0 {
            self.actions.iter().for_each(|action| match action {
                Action::DealerBlackjack => {
                    show_result_text(owner, "Dealer blackjack!");
                }
                Action::DealerWins => {
                    show_result_text(owner, "Dealer..WINS!");
                }
                Action::Draw => {
                    show_result_text(owner, "Draws are like kissing your sister");
                }
                Action::PlayerWins => {
                    show_result_text(owner, "Player..WINS!");
                }
                Action::ShowDealerHoleCard(hole_card) => {
                    show_dealer_hole_card(owner, &texture_path_from_card(&hole_card));
                }
                Action::PlayerBusts => {
                    show_result_text(owner, "Player busts, Dealer WINS!");
                }
                Action::PlayerBlackjack => {
                    show_result_text(owner, "Player has blackjack!");
                }
                Action::DealerBusts => {
                    show_result_text(owner, "Dealer busts...Player WINS!");
                }
                _ => {}
            });
            self.actions.clear();
        }

        match &self.state {
            GameState::WaitingForPlayer(_) => {
                get_typed_node::<ToolButton>("./Hit", owner).map(|node| {
                    node.set_disabled(false);
                });
                get_typed_node::<ToolButton>("./Stand", owner).map(|node| {
                    node.set_disabled(false);
                });
                get_typed_node::<ToolButton>("./NewGame", owner).map(|node| {
                    node.set_disabled(true);
                });
            }
            _ => {
                get_typed_node::<ToolButton>("./Hit", owner).map(|node| {
                    node.set_disabled(true);
                });
                get_typed_node::<ToolButton>("./Stand", owner).map(|node| {
                    node.set_disabled(true);
                });
                get_typed_node::<ToolButton>("./NewGame", owner).map(|node| {
                    node.set_disabled(false);
                });
            }
        }
    }

    #[export]
    fn card_dealt(&mut self, owner: TRef<Node2D>) {
        if self.animations.len() > 0 {
            let next_animation = self.animations.remove(0);
            self.play_animation(owner, &next_animation);
        }
    }

    fn process_animations(&mut self, owner: TRef<Node2D>) {
        let deal_actions = filter_new_card_actions(self.actions.clone());
        let deal_actions = sort_new_card_actions(deal_actions);

        let mut animations = deal_actions
            .iter()
            .filter_map(|action| match action {
                Action::NewHand(player_hand, dealer_hand) => {
                    let mut player_animations = self
                        .get_animations_for_player_cards(owner, &player_hand.cards())
                        .expect("Error getting animations");
                    let dealer_animations = self
                        .get_animations_for_initial_dealer_hand(owner, &dealer_hand)
                        .expect("Error getting animations");
                    player_animations.extend(dealer_animations);
                    Some(player_animations)
                }
                Action::NewDealerCards(cards) => {
                    self.get_animations_for_dealer_cards(owner, cards).ok()
                }
                Action::NewPlayerCard(player_card) => self
                    .get_animation_for_player_card(owner, *player_card)
                    .map(|card| vector![card])
                    .ok(),
                _ => None,
            })
            .flatten()
            .collect::<Vector<CardAnimationProperties>>();

        if self.animations.is_empty() && !animations.is_empty() {
            let first_animation = animations.pop_front().unwrap();
            self.animations = animations;
            self.play_animation(owner, &first_animation);
        } else {
            self.animations.extend(animations);
        }

        self.actions = self
            .actions
            .iter()
            .filter(|action| match action {
                Action::NewDealerCards(_) | Action::NewPlayerCard(_) | Action::NewHand(_, _) => {
                    false
                }
                _ => true,
            })
            .cloned()
            .collect::<Vector<Action>>();
    }

    fn get_animation_for_player_card(
        &self,
        owner: TRef<Node2D>,
        player_card: Card,
    ) -> Result<CardAnimationProperties, GodotError> {
        self.get_animations_for_player_cards(owner, &vector![player_card])
            .and_then(|mut animations| {
                let animation = animations.pop_front();
                animation.ok_or(GodotError::FindNodeFailed("No animations".to_string()))
            })
    }

    fn get_animations_for_player_cards(
        &self,
        owner: TRef<Node2D>,
        player_cards: &Vector<Card>,
    ) -> Result<Vector<CardAnimationProperties>, GodotError> {
        get_typed_node::<Node2D>("./PlayerHand", owner).map(|player_hand| {
            player_cards
                .iter()
                .map(|card| CardAnimationProperties {
                    destination_node: unsafe { player_hand.assume_shared() },
                    texture_name: texture_path_from_card(card),
                })
                .collect()
        })
    }

    fn get_animations_for_dealer_cards(
        &self,
        owner: TRef<Node2D>,
        dealer_cards: &Vector<Card>,
    ) -> Result<Vector<CardAnimationProperties>, GodotError> {
        get_typed_node::<Node2D>("./DealerHand", owner).map(|dealer_node| {
            dealer_cards
                .iter()
                .map(|card| CardAnimationProperties {
                    destination_node: unsafe { dealer_node.assume_shared() },
                    texture_name: texture_path_from_card(card),
                })
                .collect()
        })
    }

    fn get_animations_for_initial_dealer_hand(
        &self,
        owner: TRef<Node2D>,
        dealer_hand: &DealerHand,
    ) -> Result<Vector<CardAnimationProperties>, GodotError> {
        get_typed_node::<Node2D>("./DealerHand", owner).map(|dealer_node| {
            let dealer_node = unsafe { dealer_node.assume_shared() };
            vector![
                CardAnimationProperties {
                    destination_node: dealer_node,
                    texture_name: String::from(
                        "res://images/playingCardBacks.cardBack_blue1.atlastex"
                    )
                },
                CardAnimationProperties {
                    destination_node: dealer_node,
                    texture_name: texture_path_from_card(&dealer_hand.upcard().unwrap())
                }
            ]
        })
    }

    fn play_animation(&self, owner: TRef<Node2D>, props: &CardAnimationProperties) {
        let hand = unsafe { props.destination_node.assume_safe() };
        let resource_loader = ResourceLoader::godot_singleton();
        let sprite = Sprite::new();
        let texture = resource_loader
            .load(&props.texture_name, "AtlasTexture", false)
            .and_then(|res| res.cast::<AtlasTexture>())
            .expect("Couldn't load atlasTexture texture");

        sprite.set_texture(texture);
        sprite.set_position(Vector2::new(-hand.position().x, -hand.position().y));

        let sprite = unsafe { sprite.assume_shared() };

        let child_count = hand.get_child_count() as f32;
        hand.add_child(sprite, false);

        let tween = Tween::new();
        tween.interpolate_property(
            sprite,
            "position",
            Vector2::new(-hand.position().x, -hand.position().y),
            Vector2::new(child_count * 35.0, 0.0),
            0.25,
            Tween::TRANS_LINEAR,
            Tween::EASE_IN,
            0.0,
        );

        tween.interpolate_property(
            sprite,
            "rotation_degrees",
            0.0,
            360.0,
            0.25,
            Tween::TRANS_LINEAR,
            Tween::EASE_IN,
            0.0,
        );

        let tween = unsafe { tween.assume_shared() };
        hand.add_child(tween, false);
        let tween = unsafe { tween.assume_safe() };

        tween.start();
        tween.connect(
            "tween_all_completed",
            owner,
            "card_dealt",
            VariantArray::new_shared(),
            0,
        );
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<Blackjack>();
}

godot_init!(init);

#[cfg(test)]
mod godot_lib {
    use super::*;
    use blackjack::deck::Suit;
    use blackjack::hand::{DealerHand, Hand};

    #[test]
    fn two_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Two,
            suit: Suit::Diamond,
        };
        let resource = texture_path_from_card(&card);
        assert_eq!("res://images/playingCards.cardDiamonds2.atlastex", resource);
    }

    #[test]
    fn three_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Three,
            suit: Suit::Diamond,
        };
        let resource = texture_path_from_card(&card);

        assert_eq!("res://images/playingCards.cardDiamonds3.atlastex", resource);
    }

    #[test]
    fn ace_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Ace,
            suit: Suit::Diamond,
        };
        let resource = texture_path_from_card(&card);

        assert_eq!("res://images/playingCards.cardDiamondsA.atlastex", resource);
    }

    #[test]
    fn king_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::King,
            suit: Suit::Diamond,
        };
        let resource = texture_path_from_card(&card);

        assert_eq!("res://images/playingCards.cardDiamondsK.atlastex", resource);
    }

    #[test]
    fn queen_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Queen,
            suit: Suit::Diamond,
        };
        let resource = texture_path_from_card(&card);

        assert_eq!("res://images/playingCards.cardDiamondsQ.atlastex", resource);
    }

    #[test]
    fn jack_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Jack,
            suit: Suit::Diamond,
        };
        let resource = texture_path_from_card(&card);

        assert_eq!("res://images/playingCards.cardDiamondsJ.atlastex", resource);
    }

    #[test]
    fn jack_of_clubs_resource_string_from_card() {
        let card = Card {
            rank: Rank::Jack,
            suit: Suit::Club,
        };
        let resource = texture_path_from_card(&card);

        assert_eq!("res://images/playingCards.cardClubsJ.atlastex", resource);
    }

    #[test]
    fn sorting_empty_action_leaves_it_empty() {
        let animations = sort_new_card_actions(vector![]);

        assert_eq!(animations, vector![]);
    }

    #[test]
    fn sorting_actions_puts_player_hands_before_dealer_hands() {
        let irrelevant_card = Card {
            rank: Rank::Jack,
            suit: Suit::Club,
        };
        let actions = vector![
            Action::NewDealerCards(vector![]),
            Action::NewPlayerCard(irrelevant_card)
        ];

        let new_actions = sort_new_card_actions(actions);

        assert_eq!(
            new_actions,
            vector![
                Action::NewPlayerCard(irrelevant_card),
                Action::NewDealerCards(vector![])
            ]
        )
    }

    #[test]
    fn filter_new_card_actions_keeps_new_card_actions() {
        let irrelevant_card = Card {
            rank: Rank::Jack,
            suit: Suit::Club,
        };
        let hand = Hand::new();
        let dealer_hand = DealerHand::new();
        let actions = vector![
            Action::NewHand(hand, dealer_hand),
            Action::NewDealerCards(vector![]),
            Action::NewPlayerCard(irrelevant_card)
        ];

        assert_eq!(actions, filter_new_card_actions(actions.clone()))
    }

    #[test]
    fn filter_new_card_actions_removes_anything_else() {
        let actions = vector![Action::DealerWins, Action::Draw, Action::PlayerWins];

        assert_eq!(vector![], filter_new_card_actions(actions));
    }
}
