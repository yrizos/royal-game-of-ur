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
        match &state.phase {
            GamePhase::GameOver(winner) => return (*winner, state),
            GamePhase::WaitingForRoll => {
                let roll = Dice::roll(rng);
                let moves = state.legal_moves(roll);
                if moves.is_empty() {
                    state = state.forfeit_turn().unwrap();
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
    // Two seeds to cover different random sequences.
    for seed in [12345u64, 98765u64] {
        let mut rng = StdRng::seed_from_u64(seed);
        let rules = GameRules::finkel();
        for game_num in 0..1000 {
            let (winner, final_state) = play_random_game(&mut rng);
            let loser = winner.opponent();

            // Winner must have all pieces scored.
            assert_eq!(
                final_state.scored[winner.index()],
                rules.piece_count,
                "seed {seed} game {game_num}: winner {winner:?} has wrong scored count"
            );

            // Winner must have no pieces still on the board.
            let winner_leftover = rules
                .board_shape
                .valid_squares()
                .iter()
                .any(|&sq| matches!(final_state.board.get(sq), Some(p) if p.player == winner));
            assert!(
                !winner_leftover,
                "seed {seed} game {game_num}: winner {winner:?} still has pieces on board"
            );

            // Loser must not have scored all pieces (game would have ended earlier).
            assert!(
                final_state.scored[loser.index()] < rules.piece_count,
                "seed {seed} game {game_num}: loser {loser:?} also has all pieces scored"
            );

            // Total pieces accounted for: on-board + unplayed + scored == piece_count per player.
            for player in [winner, loser] {
                let on_board = rules
                    .board_shape
                    .valid_squares()
                    .iter()
                    .filter(
                        |&&sq| matches!(final_state.board.get(sq), Some(p) if p.player == player),
                    )
                    .count() as u8;
                let total = on_board
                    + final_state.unplayed[player.index()]
                    + final_state.scored[player.index()];
                assert_eq!(
                    total,
                    rules.piece_count,
                    "seed {seed} game {game_num}: {player:?} piece count mismatch \
                     (on_board={on_board} unplayed={} scored={})",
                    final_state.unplayed[player.index()],
                    final_state.scored[player.index()]
                );
            }
        }
    }
}
