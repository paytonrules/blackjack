use blackjack::game::{deal, hit, stand, GameState};
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome to Blackjack. You play me, the dummy dealer. I will deal.");

    let mut state = GameState::new();

    loop {
        match &state {
            GameState::Ready(_) => {
                state = deal(&state)?;
            },
            GameState::WaitingForPlayer(context) => {
                println!( "Dealer shows {:?}", context.dealer_hand.upcard().unwrap().rank);
                print!("You have ");
                for card in context.player_hand.cards() {
                    print!("{:?} ", card.rank)
                }
                println!("");
                println!("For a total of {:?}", context.player_hand.score().0);
                println!("Hit (H) or Stand (S)?");

                let mut command = String::new();
                io::stdin()
                    .read_line(&mut command)
                    .expect("Failed to read line");
                
                match command.trim() {
                    "H" | "h" => state = hit(&state)?,
                    "S" | "s" => state = stand(&state)?,
                    _ => { println!("Please try again"); state = state }
                };
            }
            GameState::DealerWins(_) => {
                println!("Dealer Wins!");
                break;
            }
            GameState::PlayerWins(_) => {
                println!("Blackjack!");
                break;
            }
            GameState::Draw(_) => {
                println!("It's a tie, womp womp, you loser");
                break;
            }
        };
    };

    Ok(())
}
