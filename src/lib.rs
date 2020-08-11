use gdnative::api::AtlasTexture;
use gdnative::prelude::*;
mod deck;
mod game;
mod hand;
use game::{deal, GameState};

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
                let first_hand = Sprite::new();

                let sprite_name = "res://images/playingCards.cardClubsJ.atlastex";
                let resource_loader = ResourceLoader::godot_singleton();
                let texture = resource_loader
                    .load(sprite_name, "AtlasTexture", false)
                    .and_then(|res| res.cast::<AtlasTexture>())
                    .expect("Couldn't load atlasTexture texture");

                first_hand.set_texture(texture);
                first_hand.set_position(Vector2::new(71.0, 200.0));
                owner.add_child(first_hand, false);
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
