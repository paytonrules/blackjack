use blackjack::game::{deal, hit, stand, Action, GameState};
use im::Vector;
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome to Blackjack. You play me, the dummy dealer. I will deal.");

    let mut state_and_actions = (GameState::new(), Vector::<Action>::new());

    loop {
        match &state_and_actions {
            (GameState::Ready(_), _) => state_and_actions = deal(&state_and_actions.0)?,
            (GameState::WaitingForPlayer(context), _) => {
                println!(
                    "Dealer shows {:?}",
                    context.dealer_hand.upcard().unwrap().rank
                );
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
                    "H" | "h" => state_and_actions.0 = hit(&state_and_actions.0)?.0,
                    "S" | "s" => state_and_actions.0 = stand(&state_and_actions.0)?.0,
                    _ => {
                        println!("Please try again");
                    }
                };
            }
            (GameState::DealerWins(context), _)
            | (GameState::PlayerWins(context), _)
            | (GameState::Draw(context), _) => {
                print!("Dealer has ");
                for card in context.dealer_hand.cards() {
                    print!("{:?} ", card.rank);
                }
                println!("");
                println!("For a total of {:?}", context.dealer_hand.score().0);
                print!("You have ");
                for card in context.player_hand.cards() {
                    print!("{:?} ", card.rank)
                }
                println!("");
                println!("For a total of {:?}", context.player_hand.score().0);
                match state_and_actions.0 {
                    GameState::DealerWins(_) => println!("Dealer Wins!"),
                    GameState::PlayerWins(_) => println!("Player Wins!"),
                    GameState::Draw(_) => println!("Tie. Womp womp"),
                    _ => panic!("Impossible state reached"),
                }
                println!("Another hand?");
                let mut command = String::new();
                io::stdin()
                    .read_line(&mut command)
                    .expect("Failed to read line");

                match command.trim() {
                    "Y" | "y" => state_and_actions = deal(&state_and_actions.0)?,
                    _ => break,
                }
            }
        };
    }

    Ok(())
}
