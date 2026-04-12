use crate::dice::{Dice, DICE_PROBABILITIES};
use crate::state::{GameState, Move};

/// Returns the AI's chosen move for the given state and dice roll.
///
/// Uses expectiminimax search to `depth` plies. Higher depth means stronger play.
/// Recommended depth settings: 2 = casual, 4 = competent, 6 = strong.
///
/// # Panics
///
/// Panics if there are no legal moves for `roll`.
pub fn choose_move(state: &GameState, roll: Dice, depth: u32) -> Move {
    let moves = state.legal_moves(roll);
    assert!(
        !moves.is_empty(),
        "choose_move called with no legal moves for roll {:?}",
        roll
    );

    moves
        .into_iter()
        .max_by(|a, b| {
            move_score(state, a, depth)
                .partial_cmp(&move_score(state, b, depth))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap()
}

fn move_score(state: &GameState, mv: &Move, depth: u32) -> f64 {
    let result = state.apply_move(mv.clone());
    if result.game_over {
        return f64::INFINITY;
    }
    if depth == 0 {
        // Leaf: evaluate from the original player's perspective.
        // If the turn passed to the opponent, negate.
        return if result.landed_on_rosette {
            evaluate(&result.new_state)
        } else {
            -evaluate(&result.new_state)
        };
    }
    if result.landed_on_rosette {
        // Same player rolls again — same perspective, no negation
        chance_node(&result.new_state, depth)
    } else {
        // Opponent's turn — negate to flip perspective
        -chance_node(&result.new_state, depth)
    }
}

/// Evaluates a position from `state.current_player()`'s perspective.
///
/// Higher is better for the current player. Considers:
/// - Scored pieces (most valuable: +10 each)
/// - Piece advancement along the path
/// - Rosette occupancy bonus (+0.5 per rosette held)
/// - Shared-row vulnerability penalty (−0.2 per piece exposed)
fn evaluate(state: &GameState) -> f64 {
    let player = state.current_player;
    let opponent = player.opponent();
    let rules = &state.rules;
    let path_len = rules.path_for(player).len() as f64;
    let mut score = 0.0;

    // Scored pieces
    score += state.scored[player.index()] as f64 * 10.0;
    score -= state.scored[opponent.index()] as f64 * 10.0;

    // Pieces on the board
    for &sq in rules.board_shape.valid_squares() {
        let piece = match state.board.get(sq) {
            Some(p) => p,
            None => continue,
        };
        let path = rules.path_for(piece.player);
        let advancement = match path.index_of(sq) {
            Some(i) => i as f64 / path_len,
            None => continue,
        };
        let is_rosette = rules.board_shape.is_rosette(sq);
        let is_shared = sq.row == 1;

        if piece.player == player {
            score += advancement;
            if is_rosette {
                score += 0.5;
            }
            if is_shared {
                score -= 0.2; // exposed to capture
            }
        } else {
            score -= advancement;
            if is_rosette {
                score -= 0.5;
            }
        }
    }

    score
}

/// Computes the expected value of `state` from the current player's perspective,
/// averaging over all possible dice rolls weighted by their probability.
fn chance_node(state: &GameState, depth: u32) -> f64 {
    if depth == 0 {
        return evaluate(state);
    }
    (0u8..=4)
        .map(|v| {
            let roll = Dice(v);
            let prob = DICE_PROBABILITIES[v as usize];
            let moves = state.legal_moves(roll);
            let value = if moves.is_empty() {
                // No legal moves: forfeit, opponent takes over — negate
                let forfeited = state.pass_turn();
                -chance_node(&forfeited, depth - 1)
            } else {
                decision_node(state, roll, depth)
            };
            prob * value
        })
        .sum()
}

/// Computes the best value achievable by the current player for a given roll.
fn decision_node(state: &GameState, roll: Dice, depth: u32) -> f64 {
    state
        .legal_moves(roll)
        .into_iter()
        .map(|mv| move_score(state, &mv, depth - 1))
        .fold(f64::NEG_INFINITY, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Square;
    use crate::player::{Piece, Player};
    use crate::state::{GameRules, GameState, PieceLocation};

    #[test]
    fn test_ai_depth_1_returns_valid_move() {
        let rules = GameRules::finkel();
        let state = GameState::new(&rules);
        let roll = Dice(2);
        let moves = state.legal_moves(roll);
        assert!(!moves.is_empty());
        let chosen = choose_move(&state, roll, 1);
        assert!(
            moves.contains(&chosen),
            "choose_move returned a move not in legal_moves"
        );
    }

    #[test]
    fn test_ai_prefers_capture_over_neutral_move() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        // P1 at (1,0), P2 at (1,1) — roll 1 lets P1 capture P2
        // Also place P1 at (2,3) — roll 1 would move it to (2,2), a neutral move
        let capture_from = Square::new(1, 0);
        let capture_to = Square::new(1, 1);
        let neutral_from = Square::new(2, 3);
        s.board
            .set(capture_from, Some(Piece::new(Player::Player1, 0)));
        s.board
            .set(capture_to, Some(Piece::new(Player::Player2, 0)));
        s.board
            .set(neutral_from, Some(Piece::new(Player::Player1, 1)));
        s.unplayed = [5, 6];
        let roll = Dice(1);
        let chosen = choose_move(&s, roll, 1);
        assert_eq!(
            chosen.from,
            PieceLocation::OnBoard(capture_from),
            "AI should prefer the capturing move"
        );
    }

    #[test]
    fn test_ai_prefers_rosette_over_neutral_move() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        // P1 at path[2]=(2,1). Roll 1 → path[3]=(2,0) which is a rosette.
        // P1 also at (1,4). Roll 1 → (1,5), neutral.
        let rosette_from = rules.path_for(Player::Player1).get(2).unwrap(); // (2,1)
        let neutral_from = rules.path_for(Player::Player1).get(8).unwrap(); // (1,4)
        s.board
            .set(rosette_from, Some(Piece::new(Player::Player1, 0)));
        s.board
            .set(neutral_from, Some(Piece::new(Player::Player1, 1)));
        s.unplayed[Player::Player1.index()] = 5;
        let roll = Dice(1);
        let chosen = choose_move(&s, roll, 1);
        assert_eq!(
            chosen.from,
            PieceLocation::OnBoard(rosette_from),
            "AI should prefer the rosette-landing move"
        );
    }

    #[test]
    fn test_ai_does_not_panic_at_depth_0() {
        let rules = GameRules::finkel();
        let state = GameState::new(&rules);
        let roll = Dice(1);
        let moves = state.legal_moves(roll);
        assert!(!moves.is_empty());
        let chosen = choose_move(&state, roll, 0);
        assert!(moves.contains(&chosen));
    }
}
