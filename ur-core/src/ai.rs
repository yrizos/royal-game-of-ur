use crate::dice::{Dice, DICE_PROBABILITIES};
use crate::state::{GameState, Move};

/// Returns the AI's chosen move for the given state and dice roll.
///
/// Uses expectiminimax search to `depth` plies. Higher depth means stronger play.
/// Recommended depth settings: 2 = casual, 4 = competent, 6 = strong.
///
/// # Panics
///
/// Panics if there are no legal moves for the given `roll` (callers must
/// verify that legal moves exist before calling this function).
pub fn choose_move(state: &GameState, roll: Dice, depth: u32) -> Move {
    todo!()
}

/// Evaluates a game state from the perspective of `state.current_player()`.
///
/// Higher scores are better for the current player. Factors considered:
/// - Piece advancement along the path (further = better)
/// - Number of scored pieces
/// - Number of available captures
/// - Rosette occupancy
/// - Vulnerability of own pieces on the shared row
fn evaluate(state: &GameState) -> f64 {
    todo!()
}

/// Expectiminimax node for a decision (the current player picks the best move).
fn decision_node(state: &GameState, roll: Dice, depth: u32) -> f64 {
    todo!()
}

/// Expectiminimax node for chance (weighted average over all dice outcomes).
fn chance_node(state: &GameState, depth: u32) -> f64 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_depth_1_returns_valid_move() {
        // choose_move must return a move that appears in legal_moves()
        todo!()
    }

    #[test]
    fn test_ai_prefers_capture_over_neutral_move() {
        // Construct a state where one move captures an opponent piece and
        // another advances without capturing; AI at depth 1 must pick the capture
        todo!()
    }

    #[test]
    fn test_ai_prefers_rosette_over_neutral_move() {
        // Construct a state where one move lands on a rosette and another does not;
        // AI at depth 1 must pick the rosette move
        todo!()
    }

    #[test]
    fn test_ai_does_not_panic_at_depth_0() {
        // depth=0 falls through to evaluate(); must return some valid move
        todo!()
    }
}
