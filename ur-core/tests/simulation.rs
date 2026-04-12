use rand::{rngs::StdRng, RngCore, SeedableRng};
use ur_core::{
    dice::Dice,
    player::Player,
    state::{GamePhase, GameRules, GameState},
};

/// Plays a single game to completion using random move selection.
///
/// Returns the winner. Panics if the game does not complete within 10_000 turns.
fn play_random_game(rng: &mut StdRng) -> Player {
    let rules = GameRules::finkel();
    let mut state = GameState::new(&rules);

    for _ in 0..10_000 {
        match state.phase.clone() {
            GamePhase::GameOver(winner) => return winner,
            GamePhase::WaitingForRoll => {
                let roll = Dice::roll(rng);
                let moves = state.legal_moves(roll);
                if moves.is_empty() {
                    state = state.pass_turn();
                } else {
                    let idx = (rng.next_u32() as usize) % moves.len();
                    let result = state.apply_move(moves[idx].clone());
                    state = result.new_state;
                }
            }
            GamePhase::WaitingForMove(_) => {
                panic!("simulation entered unexpected WaitingForMove phase")
            }
        }
    }
    panic!("game did not complete within 10_000 turns — possible infinite loop in game logic")
}

#[test]
fn test_1000_random_games_all_complete_with_valid_winner() {
    let mut rng = StdRng::seed_from_u64(12345);
    for game_num in 0..1000 {
        let winner = play_random_game(&mut rng);
        assert!(
            matches!(winner, Player::Player1 | Player::Player2),
            "game {} produced invalid winner {:?}",
            game_num,
            winner
        );
    }
}
