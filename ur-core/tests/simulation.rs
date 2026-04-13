use rand::{rngs::StdRng, RngCore, SeedableRng};
use ur_core::{
    dice::Dice,
    player::Player,
    state::{GamePhase, GameRules, GameState},
};

/// Plays a single game to completion using random move selection.
///
/// Returns the winner and the final game state.
/// Panics if the game does not complete within 10_000 turns.
fn play_random_game(rng: &mut StdRng) -> (Player, GameState) {
    let rules = GameRules::finkel();
    let mut state = GameState::new(&rules);

    for _ in 0..10_000 {
        match state.phase.clone() {
            GamePhase::GameOver(winner) => return (winner, state),
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
        }
    }
    panic!("game did not complete within 10_000 turns — possible infinite loop in game logic")
}

#[test]
fn test_1000_random_games_all_complete_with_valid_winner() {
    let mut rng = StdRng::seed_from_u64(12345);
    let rules = GameRules::finkel();
    for game_num in 0..1000 {
        let (winner, final_state) = play_random_game(&mut rng);
        assert_eq!(
            final_state.scored[winner.index()],
            rules.piece_count,
            "game {game_num}: winner {winner:?} has wrong scored count"
        );
        let leftover = rules
            .board_shape
            .valid_squares()
            .iter()
            .any(|&sq| matches!(final_state.board.get(sq), Some(p) if p.player == winner));
        assert!(
            !leftover,
            "game {game_num}: winner {winner:?} still has pieces on board"
        );
    }
}
