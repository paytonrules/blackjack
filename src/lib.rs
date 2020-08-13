use gdnative::api::AtlasTexture;
use gdnative::prelude::*;
mod deck;
mod game;
mod hand;
use deck::{Card, Rank};
use game::{deal, GameState};
use std::error;

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

fn card_texture_from_card(card: &Card) -> String {
    format!(
        "card{:?}s{}",
        card.suit,
        rank_as_texture_abbreviation(&card.rank)
    )
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
                    let resource_loader = ResourceLoader::godot_singleton();
                    for card in context.player_hand.cards() {
                        let sprite = Sprite::new();
                        let sprite_name = format!(
                            "res://images/playingCards.{}.atlastex",
                            card_texture_from_card(&card),
                        );
                        let texture = resource_loader
                            .load(sprite_name, "AtlasTexture", false)
                            .and_then(|res| res.cast::<AtlasTexture>())
                            .expect("Couldn't load atlasTexture texture");

                        let child_count = player_hand.get_child_count() as f32;
                        sprite.set_texture(texture);
                        sprite.set_position(Vector2::new(child_count * 70.0, 0.0));
                        player_hand.add_child(sprite, false);
                    }
                });

                get_typed_node::<Node2D, _>("./DealerHand", owner, |dealer_hand| {
                    let resource_loader = ResourceLoader::godot_singleton();

                    // Show dealer hole card first
                    let sprite = Sprite::new();
                    let sprite_name = "res://images/playingCardBacks.cardBack_blue1.atlastex";

                    let texture = resource_loader
                        .load(sprite_name, "AtlasTexture", false)
                        .and_then(|res| res.cast::<AtlasTexture>())
                        .expect("Couldn't load atlasTexture texture");
                    sprite.set_texture(texture);
                    sprite.set_position(Vector2::new(0.0, 0.0));
                    dealer_hand.add_child(sprite, false);

                    let sprite = Sprite::new();
                    let sprite_name = format!(
                        "res://images/playingCards.{}.atlastex",
                        card_texture_from_card(&context.dealer_hand.upcard().unwrap()),
                    );
                    let texture = resource_loader
                        .load(sprite_name, "AtlasTexture", false)
                        .and_then(|res| res.cast::<AtlasTexture>())
                        .expect("Couldn't load atlasTexture texture");

                    sprite.set_texture(texture);
                    sprite.set_position(Vector2::new(70.0, 0.0));
                    dealer_hand.add_child(sprite, false);
                });
            }

            GameState::Ready(_) => {}
            GameState::DealerWins(_) => {}
            GameState::PlayerWins(_) => {}
            GameState::Draw(_) => {}
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
    use deck::{Card, Rank, Suit};

    #[test]
    fn two_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Two,
            suit: Suit::Diamond,
        };
        let resource = card_texture_from_card(&card);

        assert_eq!("cardDiamonds2", resource);
    }

    #[test]
    fn three_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Three,
            suit: Suit::Diamond,
        };
        let resource = card_texture_from_card(&card);

        assert_eq!("cardDiamonds3", resource);
    }

    #[test]
    fn ace_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Ace,
            suit: Suit::Diamond,
        };
        let resource = card_texture_from_card(&card);

        assert_eq!("cardDiamondsA", resource);
    }

    #[test]
    fn king_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::King,
            suit: Suit::Diamond,
        };
        let resource = card_texture_from_card(&card);

        assert_eq!("cardDiamondsK", resource);
    }

    #[test]
    fn queen_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Queen,
            suit: Suit::Diamond,
        };
        let resource = card_texture_from_card(&card);

        assert_eq!("cardDiamondsQ", resource);
    }

    #[test]
    fn jack_of_diamonds_resource_string_from_card() {
        let card = Card {
            rank: Rank::Jack,
            suit: Suit::Diamond,
        };
        let resource = card_texture_from_card(&card);

        assert_eq!("cardDiamondsJ", resource);
    }

    #[test]
    fn jack_of_clubs_resource_string_from_card() {
        let card = Card {
            rank: Rank::Jack,
            suit: Suit::Club,
        };
        let resource = card_texture_from_card(&card);

        assert_eq!("cardClubsJ", resource);
    }
}
