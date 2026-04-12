use crate::board::{BoardShape, Path, Square};
use crate::dice::Dice;
use crate::player::{Piece, Player};

/// Where a piece is relative to the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceLocation {
    /// Not yet entered; waiting in the player's pool.
    Unplayed,
    /// On the board at the given square.
    OnBoard(Square),
    /// Exited the board; scored.
    Scored,
}

/// A legal move: which piece moves, from where, to where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    pub piece: Piece,
    pub from: PieceLocation,
    pub to: PieceLocation,
}

/// Outcome of applying a move to a game state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveResult {
    /// The new game state after the move.
    pub new_state: GameState,
    /// The opponent's piece that was captured, if any.
    pub captured: Option<Piece>,
    /// True if the piece landed on a rosette (grants an extra turn).
    pub landed_on_rosette: bool,
    /// True if the piece exited the board and was scored.
    pub piece_scored: bool,
    /// True if this move ended the game.
    pub game_over: bool,
}

/// Which phase the game is currently in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GamePhase {
    /// The current player must roll the dice.
    WaitingForRoll,
    /// The current player must choose a move given this roll.
    WaitingForMove(Dice),
    /// The game is over; this player won.
    GameOver(Player),
}

/// Full ruleset configuration.
///
/// Bundles geometry, paths, and rule flags. `GameRules::finkel()` returns the
/// default Finkel reconstruction rules. This is the extension point for
/// future rulesets (Masters, Aseb).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameRules {
    pub board_shape: BoardShape,
    pub path_player1: Path,
    pub path_player2: Path,
    /// Number of pieces each player starts with (7 in Finkel rules).
    pub piece_count: u8,
    /// If true, landing on a rosette grants the current player another turn.
    pub rosettes_grant_extra_turn: bool,
    /// If true, a piece on a rosette cannot be captured.
    pub rosettes_are_safe: bool,
}

impl GameRules {
    /// Returns the default Finkel ruleset.
    pub fn finkel() -> Self {
        todo!()
    }

    /// Returns the path for the given player.
    pub fn path_for(&self, player: Player) -> &Path {
        todo!()
    }
}

/// The board: tracks which piece occupies each square.
///
/// Internally uses a flat 24-element array (3 rows × 8 cols, indexed `row * 8 + col`).
/// Squares that do not exist on the board are always `None`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board {
    cells: [Option<Piece>; 24],
}

impl Board {
    /// Creates an empty board.
    pub fn new() -> Self {
        todo!()
    }

    /// Returns the piece on `sq`, or `None` if the square is empty.
    pub fn get(&self, sq: Square) -> Option<Piece> {
        todo!()
    }

    /// Places or removes a piece on `sq`.
    pub(crate) fn set(&mut self, sq: Square, piece: Option<Piece>) {
        todo!()
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

/// The complete, immutable snapshot of a game at a point in time.
///
/// Derives `Hash` to support use in transposition tables. Note: `rules` is
/// included in the hash even though it is constant for a given game; a
/// custom `Hash` impl that skips `rules` would be more efficient for
/// large search trees.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameState {
    pub rules: GameRules,
    pub board: Board,
    /// Number of unplayed pieces for each player. Index via `Player::index()`.
    pub unplayed: [u8; 2],
    /// Number of scored pieces for each player. Index via `Player::index()`.
    pub scored: [u8; 2],
    pub current_player: Player,
    pub phase: GamePhase,
}

impl GameState {
    /// Creates a new game in the initial state with all pieces unplayed.
    pub fn new(rules: &GameRules) -> Self {
        todo!()
    }

    /// Returns all legal moves for the current player given `roll`.
    ///
    /// Returns an empty `Vec` if no moves are possible (turn is forfeit).
    pub fn legal_moves(&self, roll: Dice) -> Vec<Move> {
        todo!()
    }

    /// Applies `mv` to this state and returns the result.
    ///
    /// # Panics
    ///
    /// Panics if `mv` is not a legal move in this state.
    pub fn apply_move(&self, mv: Move) -> MoveResult {
        todo!()
    }

    /// Returns true if the game is over.
    pub fn is_finished(&self) -> bool {
        todo!()
    }

    /// Returns the winning player, or `None` if the game is not over.
    pub fn winner(&self) -> Option<Player> {
        todo!()
    }

    /// Returns whose turn it is.
    pub fn current_player(&self) -> Player {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Legal move generation ────────────────────────────────────────────────

    #[test]
    fn test_no_legal_moves_roll_0() {
        // Rolling 0 always produces an empty move list regardless of board state
        todo!()
    }

    #[test]
    fn test_no_legal_moves_all_blocked() {
        // Construct a state where every reachable destination is occupied by
        // a friendly piece; legal_moves must return empty vec
        todo!()
    }

    #[test]
    fn test_can_enter_piece_from_pool() {
        // When a player has unplayed pieces and the entry square is empty,
        // entering is a legal move
        todo!()
    }

    #[test]
    fn test_cannot_enter_when_entry_occupied_by_friendly() {
        // Entry square occupied by the current player's own piece blocks entry
        todo!()
    }

    #[test]
    fn test_bearing_off_requires_exact_roll() {
        // A piece at the last path square can only bear off with the exact roll
        // that lands it on exit; no overshoot
        todo!()
    }

    #[test]
    fn test_overshoot_not_allowed() {
        // A piece that would overshoot the exit is not a legal move
        todo!()
    }

    #[test]
    fn test_capture_on_shared_row_is_legal() {
        // Landing on a shared-row square occupied by an opponent piece is legal
        todo!()
    }

    #[test]
    fn test_capture_blocked_by_rosette() {
        // Landing on a rosette occupied by an opponent piece is NOT legal
        todo!()
    }

    #[test]
    fn test_friendly_square_blocked() {
        // Cannot move to a square occupied by a friendly piece
        todo!()
    }

    #[test]
    fn test_rosette_safe_from_capture() {
        // Piece on a rosette is immune to capture; opponent cannot move there
        todo!()
    }

    // ── Move application ─────────────────────────────────────────────────────

    #[test]
    fn test_piece_advances_correct_number_of_squares() {
        // After apply_move, the piece is at path[start_index + roll]
        todo!()
    }

    #[test]
    fn test_capture_returns_opponent_piece_to_pool() {
        // After a capture, the opponent's unplayed count increases by 1
        // and the square holds the capturing piece
        todo!()
    }

    #[test]
    fn test_capture_metadata_is_set() {
        // MoveResult::captured is Some(piece) when a capture occurs
        todo!()
    }

    #[test]
    fn test_rosette_grants_extra_turn() {
        // After landing on a rosette, the current player in new_state is
        // the same player (not the opponent) and phase is WaitingForRoll
        todo!()
    }

    #[test]
    fn test_non_rosette_passes_turn() {
        // After landing on a non-rosette, current_player switches to the opponent
        todo!()
    }

    #[test]
    fn test_scoring_a_piece() {
        // Bearing off increments scored[player.index()] and
        // MoveResult::piece_scored is true
        todo!()
    }

    #[test]
    fn test_win_detection_all_7_scored() {
        // When scored[player.index()] reaches piece_count, is_finished() is true
        // and winner() returns Some(player)
        todo!()
    }

    #[test]
    fn test_apply_illegal_move_panics() {
        // Passing a Move not in legal_moves() must panic
        todo!()
    }

    // ── Turn forfeiture ──────────────────────────────────────────────────────

    #[test]
    fn test_forfeit_turn_when_no_legal_moves() {
        // When legal_moves() is empty, it is the caller's responsibility to
        // pass the turn; verify that GameState::new starts with correct player
        // and that legal_moves returns empty for a constructed no-move state
        todo!()
    }
}
