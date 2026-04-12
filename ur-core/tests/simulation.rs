use rand::{rngs::StdRng, RngCore, SeedableRng};
use ur_core::{
    dice::Dice,
    player::Player,
    state::{GamePhase, GameRules, GameState},
};

/// Plays a single game to completion using random move selection.
///
/// Returns the winner. Panics if the game does not complete within 10_000 turns
/// (which would indicate an infinite loop bug in the game logic).
fn play_random_game(rng: &mut StdRng) -> Player {
    let rules = GameRules::finkel();
    let mut state = GameState::new(&rules);

    for _ in 0..10_000 {
        match &state.phase.clone() {
            GamePhase::GameOver(winner) => return *winner,
            GamePhase::WaitingForRoll => {
                let roll = Dice::roll(rng);
                let moves = state.legal_moves(roll);
                if moves.is_empty() {
                    // No legal moves: forfeit turn. The implementation is
                    // responsible for advancing to the next player when
                    // legal_moves() returns empty — this is a placeholder
                    // that will be filled in during state implementation.
                    todo!("forfeit: advance to next player's turn")
                } else {
                    let idx = (rng.next_u32() as usize) % moves.len();
                    let result = state.apply_move(moves[idx].clone());
                    state = result.new_state;
                }
            }
            GamePhase::WaitingForMove(_) => {
                panic!("simulation drove state into WaitingForMove without rolling")
            }
        }
    }
    panic!("game did not complete within 10_000 turns")
}

#[test]
fn test_1000_random_games_all_complete_with_valid_winner() {
    todo!()
}
