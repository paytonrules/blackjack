struct GameContext {

}

#[derive(Debug, PartialEq)]
enum Game {
    Ready,
    CheckingDealerBlackjack,
    WaitingForPlayer,
    CheckingPlayerHand,
    PlayerLoses,
    PlayingDealerHand,
}

fn deal(state: Game) -> Game {
    Game::Ready
}

#[cfg(test)]
mod tests {
    use super::*;

    fn experiment() {
        let game_state = Game::Ready;

        assert_eq!(Deal(game_state), Game::Ready);
    }
}