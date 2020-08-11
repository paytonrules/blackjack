use gdnative::api::AtlasTexture;
use gdnative::prelude::*;
mod deck;
mod game;
mod hand;
use deck::{Card, Rank};
use game::{deal, GameState};

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
struct Gui {}

#[methods]
impl Gui {
    fn new(_owner: &Node2D) -> Self {
        Gui {}
    }

    #[export]
    fn _on_new_game_pressed(&self, owner: &Node2D) {
        let mut state = GameState::new();

        state = deal(&state).expect("Dealing has to work, basically");

        match state {
            GameState::WaitingForPlayer(context) => {
                godot_print!("I dealt some cards including {:?}", context.player_hand);
                for card in context.player_hand.cards() {
                    let first_hand = Sprite::new();

                    let sprite_name = format!(
                        "res://images/playingCards.{}.atlastex",
                        card_texture_from_card(&card),
                    );
                    let resource_loader = ResourceLoader::godot_singleton();
                    let texture = resource_loader
                        .load(sprite_name, "AtlasTexture", false)
                        .and_then(|res| res.cast::<AtlasTexture>())
                        .expect("Couldn't load atlasTexture texture");

                    first_hand.set_texture(texture);
                    first_hand.set_position(Vector2::new(71.0, 200.0));
                    owner.add_child(first_hand, false);
                }
            }
            _ => {
                godot_print!("New state I didn't expect");
            }
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<Gui>();
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
