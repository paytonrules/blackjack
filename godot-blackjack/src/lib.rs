use blackjack::deck::{Card, Rank};
use blackjack::game::{deal, stand, GameState};
use gdnative::api::AtlasTexture;
use gdnative::prelude::*;

pub fn get_typed_node<O, F>(name: &str, owner: &Node, mut f: F)
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

fn add_card_to_hand(texture: &str, hand: &Node2D) {
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

fn show_dealer_hole_card(texture: &str, hand: &Node2D) {
    let resource_loader = ResourceLoader::godot_singleton();
    let sprite = Sprite::new();
    let texture = resource_loader
        .load(texture, "AtlasTexture", false)
        .and_then(|res| res.cast::<AtlasTexture>())
        .expect("Couldn't load atlasTexture texture");

    sprite.set_texture(texture);
    sprite.set_position(Vector2::new(0.0, 0.0));
    hand.add_child(sprite, false);
    let hole_texture = hand.get_child(0).unwrap();
    hand.remove_child(hole_texture);
    unsafe { hole_texture.assume_unique() }.queue_free();
}

#[derive(NativeClass)]
#[inherit(Node2D)]
struct Hand {}

#[methods]
impl Hand {
    fn new(_owner: &Node2D) -> Self {
        Hand {}
    }
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
        self.state = deal(&self.state).expect("Dealing has to work, basically");

        match &self.state {
            GameState::WaitingForPlayer(context) => {
                get_typed_node::<Node2D, _>("./PlayerHand", owner, |player_hand| {
                    for card in context.player_hand.cards() {
                        add_card_to_hand(&texture_path_from_card(&card), &player_hand);
                    }
                });

                get_typed_node::<Node2D, _>("./DealerHand", owner, |dealer_hand| {
                    add_card_to_hand(
                        "res://images/playingCardBacks.cardBack_blue1.atlastex",
                        &dealer_hand,
                    );

                    add_card_to_hand(
                        &texture_path_from_card(&context.dealer_hand.upcard().unwrap()),
                        &dealer_hand,
                    );
                });
            }

            GameState::Ready(_) => {}
            GameState::DealerWins(_) => {}
            GameState::PlayerWins(_) => {}
            GameState::Draw(_) => {}
        }
    }

    #[export]
    fn _on_stand_pressed(&mut self, owner: &Node2D) {
        self.state = stand(&self.state).expect("You could stand at this point");

        match &self.state {
            GameState::WaitingForPlayer(_) => {}
            GameState::Ready(_) => {}
            GameState::DealerWins(context)
            | GameState::PlayerWins(context)
            | GameState::Draw(context) => {
                get_typed_node::<Node2D, _>("./DealerHand", owner, |dealer_hand| {
                    show_dealer_hole_card(
                        &texture_path_from_card(&context.dealer_hand.hole_card().unwrap()),
                        &dealer_hand,
                    );

                    for card in context.dealer_hand.cards().skip(2) {
                        add_card_to_hand(&texture_path_from_card(&card), &dealer_hand);
                    }
                });
            }
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<Blackjack>();
    handle.add_class::<Hand>();
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
