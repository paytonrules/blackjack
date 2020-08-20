use blackjack::deck::{Card, Rank};
use blackjack::{
    game::{deal, hit, stand, GameState},
    hand::{DealerHand, Hand},
};
use gdnative::api::{Animation, AnimationPlayer, AtlasTexture, RichTextLabel, ToolButton};
use gdnative::prelude::*;

fn load_scene<F, T>(name: &str, mut f: F) -> Option<T>
where
    F: FnMut(TRef<PackedScene>) -> Option<T>,
{
    let scene = ResourceLoader::godot_singleton().load(name, "PackedScene", false)?;
    let scene = unsafe { scene.assume_safe() };
    let packed_scene = scene.cast::<PackedScene>()?;

    f(packed_scene)
}

fn create_node_from_scene<T>(name: &str) -> Option<Ref<T>>
where
    T: GodotObject<RefKind = ManuallyManaged> + SubClass<Node>,
{
    load_scene(name, |scene| {
        scene
            .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
            .map(|node| unsafe { node.assume_unique() })
            .and_then(|node| node.cast::<T>())
            .map(|node| node.into_shared())
    })
}

fn clear_all_children(node_name: &str, owner: &Node) {
    get_typed_node::<Node, _>(node_name, owner, |parent| {
        for var in parent.get_children().iter() {
            let child = var.try_to_object::<Node>();
            child.map(|child| {
                let child = unsafe { child.assume_safe() };
                parent.remove_child(child);
                child.queue_free()
            });
        }
    });
}

fn get_typed_node<O, F>(name: &str, owner: &Node, mut f: F)
where
    F: FnMut(TRef<O>),
    O: GodotObject + SubClass<Node>,
{
    let node = match owner
        .get_node(name)
        .map(|node| unsafe { node.assume_safe() })
        .and_then(|node| node.cast::<O>())
    {
        Some(it) => it,
        _ => {
            godot_print!("Couldn't find node {:?}", name);
            return;
        }
    };
    f(node)
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

fn add_card_to_hand(owner: &Node2D, texture: &str, hand: &Node2D) {
    play_deal_animation(owner, &hand, texture);
}

fn show_dealer_hole_card(texture: &str, hand: &Node2D) {
    let resource_loader = ResourceLoader::godot_singleton();
    let sprite = Sprite::new();
    let texture = resource_loader
        .load(texture, "AtlasTexture", false)
        .and_then(|res| res.cast::<AtlasTexture>())
        .expect("Couldn't load atlasTexture texture");
    sprite.set_texture(texture);
    sprite.set_position(Vector2::new(0.0, 0.0));

    let hole_texture = hand.get_child(0).unwrap();
    let hole_texture = unsafe { hole_texture.assume_unique() };
    hole_texture.replace_by(sprite, false);
    hole_texture.queue_free();
}

fn show_player_hand(owner: &Node2D, player_hand: &Hand) {
    get_typed_node::<Node2D, _>("./PlayerHand", owner, |node| {
        for card in player_hand.cards() {
            add_card_to_hand(owner, &texture_path_from_card(&card), &node);
        }
    });
}

fn show_initial_dealer_hand(owner: &Node2D, dealer_hand: &DealerHand) {
    get_typed_node::<Node2D, _>("./DealerHand", owner, |node| {
        add_card_to_hand(
            owner,
            "res://images/playingCardBacks.cardBack_blue1.atlastex",
            &node,
        );

        add_card_to_hand(
            owner,
            &texture_path_from_card(&dealer_hand.upcard().unwrap()),
            &node,
        );
    });
}

fn show_remaining_dealer_hand(owner: &Node2D, dealer_hand: &DealerHand) {
    get_typed_node::<Node2D, _>("./DealerHand", owner, |node| {
        show_dealer_hole_card(
            &texture_path_from_card(&dealer_hand.hole_card().unwrap()),
            &node,
        );

        for card in dealer_hand.cards().skip(2) {
            add_card_to_hand(owner, &texture_path_from_card(&card), &node);
        }
    });
}

fn show_entire_dealer_hand(owner: &Node2D, dealer_hand: &DealerHand) {
    get_typed_node::<Node2D, _>("./DealerHand", owner, |node| {
        for card in dealer_hand.cards() {
            add_card_to_hand(owner, &texture_path_from_card(&card), &node);
        }
    });
}

fn show_latest_player_card(owner: &Node2D, player_hand: &Hand) {
    get_typed_node::<Node2D, _>("./PlayerHand", owner, |node| {
        let player_cards = player_hand.cards();
        let new_card = player_cards.last().unwrap();
        add_card_to_hand(owner, &texture_path_from_card(&new_card), &node);
    })
}

fn show_result_text(owner: &Node2D, result: &str) {
    get_typed_node::<RichTextLabel, _>("./Result", owner, |node| {
        node.add_text(result);
    });
}

fn clear_result_text(owner: &Node2D) {
    get_typed_node::<RichTextLabel, _>("./Result", owner, |node| {
        node.clear();
    });
}

fn play_deal_animation(
    owner: &Node2D,
    hand: &Node2D,
    texture: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let fly_in_node = create_node_from_scene::<Node2D>("res://CardFlyIn.tscn")
        .ok_or("Could Not Create Node From Scene res://CardFlyIn.tscn")?;
    let fly_in_node = unsafe { fly_in_node.assume_safe() };

    get_typed_node::<AnimationPlayer, _>("./CardAnimation", &fly_in_node, |player| {
        let animation = player
            .get_animation("Card Fly In")
            .expect("Could Not find 'Card Fly In' animation");
        let animation = unsafe { animation.assume_safe() };

        animation.track_set_key_value(0, 1, hand.position());

        owner.add_child(fly_in_node, false);

        let binds = VariantArray::new();
        binds.push(texture);
        let hand = unsafe { hand.assume_shared() };
        binds.push(hand);
        let fly_in_node = unsafe { fly_in_node.assume_shared() };
        binds.push(fly_in_node);
        let owner = unsafe { owner.assume_shared() };
        player
            .connect(
                "animation_finished",
                owner,
                "card_dealt",
                binds.into_shared(),
                0,
            )
            .expect("This is some next level nonsense");

        player.play("Card Fly In", -1.0, 1.0, false);
    });
    Ok(())
}

#[derive(NativeClass)]
#[inherit(Node2D)]
struct Blackjack {
    state: GameState,
}

#[methods]
impl Blackjack {
    fn new(_owner: &Node2D) -> Self {
        Blackjack {
            state: GameState::new(),
        }
    }

    #[export]
    fn _on_new_game_pressed(&mut self, owner: &Node2D) {
        clear_all_children("./DealerHand", owner);
        clear_all_children("./PlayerHand", owner);
        clear_result_text(owner);

        self.state = deal(&self.state).expect("Dealing has to work, basically");

        match &self.state {
            GameState::WaitingForPlayer(context) => {
                show_player_hand(owner, &context.player_hand);
                show_initial_dealer_hand(owner, &context.dealer_hand);
            }
            GameState::DealerWins(context) => {
                show_player_hand(owner, &context.player_hand);
                show_entire_dealer_hand(owner, &context.dealer_hand);
                show_result_text(owner, "Dealer BLACKJACK!");
            }
            GameState::PlayerWins(context) => {
                show_player_hand(owner, &context.player_hand);
                show_entire_dealer_hand(owner, &context.dealer_hand);
                show_result_text(owner, "PLAYER BLACKJACK!");
            }
            GameState::Draw(context) => {
                show_player_hand(owner, &context.player_hand);
                show_entire_dealer_hand(owner, &context.dealer_hand);
                show_result_text(owner, "Everybody has BLACKJACK!");
            }
            GameState::Ready(_) => godot_error!("GameState::Ready Should be impossible!"),
        }
    }

    #[export]
    fn _on_stand_pressed(&mut self, owner: &Node2D) {
        self.state = stand(&self.state).expect("You could stand at this point");

        match &self.state {
            GameState::WaitingForPlayer(_) => {
                godot_error!("GameState::WaitingForPlayer Should be impossible!")
            }
            GameState::DealerWins(context) => {
                show_result_text(owner, "Dealer..WINS!");
                show_remaining_dealer_hand(owner, &context.dealer_hand);
            }
            GameState::PlayerWins(context) => {
                show_result_text(owner, "Player..WINS!");
                show_remaining_dealer_hand(owner, &context.dealer_hand);
            }
            GameState::Draw(context) => {
                show_result_text(owner, "Draws are like kissing your sister");
                show_remaining_dealer_hand(owner, &context.dealer_hand);
            }
            GameState::Ready(_) => godot_error!("GameState::Ready Should be impossible!"),
        }
    }

    #[export]
    fn _on_hit_pressed(&mut self, owner: &Node2D) {
        self.state = hit(&self.state).expect("You can hit at this point");

        match &self.state {
            GameState::WaitingForPlayer(context) => {
                show_latest_player_card(owner, &context.player_hand);
            }
            GameState::DealerWins(context) => {
                show_latest_player_card(owner, &context.player_hand);
                show_remaining_dealer_hand(owner, &context.dealer_hand);
                show_result_text(owner, "Dealer..WINS!");
            }
            GameState::PlayerWins(context) => {
                show_latest_player_card(owner, &context.player_hand);
                show_remaining_dealer_hand(owner, &context.dealer_hand);
                show_result_text(owner, "Player..WINS!");
            }
            GameState::Draw(context) => {
                show_latest_player_card(owner, &context.player_hand);
                show_remaining_dealer_hand(owner, &context.dealer_hand);
                show_result_text(owner, "Draws are like kissing your sister");
            }
            GameState::Ready(_) => godot_error!("GameState::Ready Should be impossible!"),
        }
    }

    #[export]
    fn _process(&self, owner: &Node2D, _delta: f64) {
        match &self.state {
            GameState::WaitingForPlayer(_) => {
                get_typed_node::<ToolButton, _>("./Hit", owner, |node| {
                    node.set_disabled(false);
                });
                get_typed_node::<ToolButton, _>("./Stand", owner, |node| {
                    node.set_disabled(false);
                });
                get_typed_node::<ToolButton, _>("./NewGame", owner, |node| {
                    node.set_disabled(true);
                });
            }
            _ => {
                get_typed_node::<ToolButton, _>("./Hit", owner, |node| {
                    node.set_disabled(true);
                });
                get_typed_node::<ToolButton, _>("./Stand", owner, |node| {
                    node.set_disabled(true);
                });
                get_typed_node::<ToolButton, _>("./NewGame", owner, |node| {
                    node.set_disabled(false);
                });
            }
        }
    }

    #[export]
    fn card_dealt(
        &self,
        owner: &Node2D,
        _anim_name: String,
        texture: String,
        hand: Ref<Node2D>,
        fly_in_node: Ref<Node2D>,
    ) {
        owner.remove_child(fly_in_node);
        let fly_in_node = unsafe { fly_in_node.assume_unique() };
        fly_in_node.queue_free();

        let hand = unsafe { hand.assume_safe() };
        let resource_loader = ResourceLoader::godot_singleton();
        let sprite = Sprite::new();
        let texture = resource_loader
            .load(texture, "AtlasTexture", false)
            .and_then(|res| res.cast::<AtlasTexture>())
            .expect("Couldn't load atlasTexture texture");

        let child_count = hand.get_child_count() as f32;
        sprite.set_texture(texture);
        sprite.set_position(Vector2::new(child_count * 70.0, 0.0));
        hand.add_child(sprite, false);
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
}
