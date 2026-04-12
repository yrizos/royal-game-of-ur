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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub piece: Piece,
    pub from: PieceLocation,
    pub to: PieceLocation,
}

/// Outcome of applying a move to a game state.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameRules {
    pub board_shape: BoardShape,
    pub path_player1: Path,
    pub path_player2: Path,
    pub piece_count: u8,
    pub rosettes_grant_extra_turn: bool,
    pub rosettes_are_safe: bool,
}

impl GameRules {
    /// Returns the default Finkel ruleset.
    pub fn finkel() -> Self {
        let board_shape = BoardShape::finkel();
        let path_player1 = Path::new(vec![
            Square::new(2, 3),
            Square::new(2, 2),
            Square::new(2, 1),
            Square::new(2, 0),
            Square::new(1, 0),
            Square::new(1, 1),
            Square::new(1, 2),
            Square::new(1, 3),
            Square::new(1, 4),
            Square::new(1, 5),
            Square::new(1, 6),
            Square::new(1, 7),
            Square::new(2, 7),
            Square::new(2, 6),
        ]);
        let path_player2 = Path::new(vec![
            Square::new(0, 3),
            Square::new(0, 2),
            Square::new(0, 1),
            Square::new(0, 0),
            Square::new(1, 0),
            Square::new(1, 1),
            Square::new(1, 2),
            Square::new(1, 3),
            Square::new(1, 4),
            Square::new(1, 5),
            Square::new(1, 6),
            Square::new(1, 7),
            Square::new(0, 7),
            Square::new(0, 6),
        ]);
        Self {
            board_shape,
            path_player1,
            path_player2,
            piece_count: 7,
            rosettes_grant_extra_turn: true,
            rosettes_are_safe: true,
        }
    }

    /// Returns the path for the given player.
    pub fn path_for(&self, player: Player) -> &Path {
        match player {
            Player::Player1 => &self.path_player1,
            Player::Player2 => &self.path_player2,
        }
    }
}

/// The board: tracks which piece occupies each square.
///
/// Internally uses a flat 24-element array (3 rows × 8 cols, indexed `row * 8 + col`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board {
    cells: [Option<Piece>; 24],
}

impl Board {
    pub fn new() -> Self {
        Self { cells: [None; 24] }
    }

    pub fn get(&self, sq: Square) -> Option<Piece> {
        self.cells[sq.row as usize * 8 + sq.col as usize]
    }

    pub(crate) fn set(&mut self, sq: Square, piece: Option<Piece>) {
        self.cells[sq.row as usize * 8 + sq.col as usize] = piece;
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

/// The complete, immutable snapshot of a game at a point in time.
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
        Self {
            rules: rules.clone(),
            board: Board::new(),
            unplayed: [rules.piece_count, rules.piece_count],
            scored: [0, 0],
            current_player: Player::Player1,
            phase: GamePhase::WaitingForRoll,
        }
    }

    /// Returns all legal moves for the current player given `roll`.
    ///
    /// Returns an empty `Vec` if no moves are possible (turn is forfeit).
    pub fn legal_moves(&self, roll: Dice) -> Vec<Move> {
        if roll.value() == 0 {
            return Vec::new();
        }

        let player = self.current_player;
        let path = self.rules.path_for(player);
        let path_len = path.len();
        let roll = roll.value() as usize;
        let mut moves = Vec::new();

        // ── Try entering a piece from the unplayed pool ──────────────────────────
        if self.unplayed[player.index()] > 0 {
            // An unplayed piece is at logical position -1 (before path[0]).
            // Moving `roll` squares lands it at path[roll - 1].
            let target_idx = roll - 1;
            if target_idx < path_len {
                let target_sq = path.get(target_idx).unwrap();
                if self.can_land_on(target_sq) {
                    moves.push(Move {
                        piece: self.next_entering_piece(),
                        from: PieceLocation::Unplayed,
                        to: PieceLocation::OnBoard(target_sq),
                    });
                }
            }
        }

        // ── Try advancing each piece already on the board ────────────────────────
        for &sq in self.rules.board_shape.valid_squares() {
            let piece = match self.board.get(sq) {
                Some(p) if p.player == player => p,
                _ => continue,
            };

            let path_idx = match path.index_of(sq) {
                Some(i) => i,
                None => continue, // square not on this player's path
            };

            let new_path_idx = path_idx + roll;

            if new_path_idx == path_len {
                // Exact bear-off
                moves.push(Move {
                    piece,
                    from: PieceLocation::OnBoard(sq),
                    to: PieceLocation::Scored,
                });
            } else if new_path_idx < path_len {
                let target_sq = path.get(new_path_idx).unwrap();
                if self.can_land_on(target_sq) {
                    moves.push(Move {
                        piece,
                        from: PieceLocation::OnBoard(sq),
                        to: PieceLocation::OnBoard(target_sq),
                    });
                }
            }
            // else: overshoot — not a legal move, skip
        }

        moves
    }

    /// Applies `mv` to this state and returns the result.
    ///
    /// # Panics
    ///
    /// Panics if `mv` is not a structurally valid move (piece not present at `from`).
    pub fn apply_move(&self, mv: Move) -> MoveResult {
        let player = self.current_player;
        let mut new_state = self.clone();
        let mut captured: Option<Piece> = None;
        let mut piece_scored = false;

        // ── Remove piece from source ─────────────────────────────────────────────
        match mv.from {
            PieceLocation::Unplayed => {
                assert!(
                    new_state.unplayed[player.index()] > 0,
                    "no unplayed pieces for {:?}",
                    player
                );
                new_state.unplayed[player.index()] -= 1;
            }
            PieceLocation::OnBoard(sq) => {
                assert_eq!(
                    new_state.board.get(sq),
                    Some(mv.piece),
                    "piece {:?} not found at {:?}",
                    mv.piece,
                    sq
                );
                new_state.board.set(sq, None);
            }
            PieceLocation::Scored => panic!("scored pieces cannot move"),
        }

        // ── Place piece at destination ───────────────────────────────────────────
        let landed_on_rosette = match mv.to {
            PieceLocation::OnBoard(sq) => {
                // Capture opponent piece if present
                if let Some(occupant) = new_state.board.get(sq) {
                    assert_eq!(
                        occupant.player,
                        player.opponent(),
                        "cannot capture friendly piece"
                    );
                    captured = Some(occupant);
                    new_state.unplayed[player.opponent().index()] += 1;
                }
                new_state.board.set(sq, Some(mv.piece));
                new_state.rules.board_shape.is_rosette(sq)
            }
            PieceLocation::Scored => {
                new_state.scored[player.index()] += 1;
                piece_scored = true;
                false
            }
            PieceLocation::Unplayed => panic!("pieces cannot move to unplayed"),
        };

        // ── Advance turn ─────────────────────────────────────────────────────────
        let game_over = new_state.scored[player.index()] == new_state.rules.piece_count;

        if game_over {
            new_state.phase = GamePhase::GameOver(player);
        } else if landed_on_rosette && new_state.rules.rosettes_grant_extra_turn {
            // Same player rolls again — phase resets, current_player unchanged
            new_state.phase = GamePhase::WaitingForRoll;
        } else {
            new_state.current_player = player.opponent();
            new_state.phase = GamePhase::WaitingForRoll;
        }

        MoveResult {
            new_state,
            captured,
            landed_on_rosette,
            piece_scored,
            game_over,
        }
    }

    /// Passes the turn to the opponent without a move.
    ///
    /// Call this when `legal_moves()` returns empty (turn forfeited).
    pub fn pass_turn(&self) -> GameState {
        let mut new_state = self.clone();
        new_state.current_player = self.current_player.opponent();
        new_state.phase = GamePhase::WaitingForRoll;
        new_state
    }

    /// Returns true if the game is over.
    pub fn is_finished(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver(_))
    }

    /// Returns the winning player, or `None` if the game is not over.
    pub fn winner(&self) -> Option<Player> {
        match self.phase {
            GamePhase::GameOver(player) => Some(player),
            _ => None,
        }
    }

    /// Returns whose turn it is.
    pub fn current_player(&self) -> Player {
        self.current_player
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Returns a piece from the current player's unplayed pool.
    ///
    /// Assigns the lowest piece index not currently on the board for this player.
    fn next_entering_piece(&self) -> Piece {
        let player = self.current_player;
        let mut on_board: Vec<u8> = self
            .rules
            .board_shape
            .valid_squares()
            .iter()
            .filter_map(|&sq| self.board.get(sq))
            .filter(|p| p.player == player)
            .map(|p| p.index)
            .collect();
        on_board.sort_unstable();
        let mut next = 0u8;
        for idx in on_board {
            if idx == next {
                next += 1;
            }
        }
        Piece::new(player, next)
    }

    /// Returns whether the current player can land on `sq`.
    ///
    /// Returns false if the square is occupied by a friendly piece,
    /// or if it is a rosette occupied by an opponent piece (safe from capture).
    fn can_land_on(&self, sq: Square) -> bool {
        match self.board.get(sq) {
            None => true,
            Some(piece) => {
                if piece.player == self.current_player {
                    false // friendly occupant
                } else {
                    // opponent occupant: capturable unless rosette is safe
                    !(self.rules.rosettes_are_safe && self.rules.board_shape.is_rosette(sq))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers for constructing test states ─────────────────────────────────

    /// Places `piece` at the given path index for the current player and returns
    /// a cloned state with that change applied. Does NOT adjust unplayed counts.
    #[allow(dead_code)]
    fn place_at_path_idx(
        state: &GameState,
        player: Player,
        piece_idx: u8,
        path_idx: usize,
    ) -> GameState {
        let sq = state.rules.path_for(player).get(path_idx).unwrap();
        let mut s = state.clone();
        s.board.set(sq, Some(Piece::new(player, piece_idx)));
        s
    }

    // ── Legal move generation ────────────────────────────────────────────────

    #[test]
    fn test_no_legal_moves_roll_0() {
        let rules = GameRules::finkel();
        let state = GameState::new(&rules);
        assert!(state.legal_moves(Dice(0)).is_empty());
    }

    #[test]
    fn test_no_legal_moves_all_blocked() {
        let rules = GameRules::finkel();
        // All of Player1's entry squares blocked by Player1's own pieces,
        // and no pieces on board yet — just zero unplayed pieces so nothing can enter.
        let mut state = GameState::new(&rules);
        state.unplayed[Player::Player1.index()] = 0;
        // No pieces on board, no unplayed → no moves for any roll
        assert!(state.legal_moves(Dice(2)).is_empty());
    }

    #[test]
    fn test_can_enter_piece_from_pool() {
        let rules = GameRules::finkel();
        let state = GameState::new(&rules); // all 7 pieces unplayed
        let moves = state.legal_moves(Dice(1));
        // With roll=1 and entry square (path[0] = (2,3)) empty, entering is legal
        assert!(!moves.is_empty());
        let entry_sq = rules.path_for(Player::Player1).get(0).unwrap();
        assert!(
            moves
                .iter()
                .any(|m| m.from == PieceLocation::Unplayed
                    && m.to == PieceLocation::OnBoard(entry_sq))
        );
    }

    #[test]
    fn test_cannot_enter_when_entry_occupied_by_friendly() {
        let rules = GameRules::finkel();
        // Place a Player1 piece at path index 0 (the roll-1 entry square for roll=1)
        let state = GameState::new(&rules);
        let mut s = state.clone();
        let entry_sq = rules.path_for(Player::Player1).get(0).unwrap();
        s.board.set(entry_sq, Some(Piece::new(Player::Player1, 0)));
        // Now roll=1 would land there — must be blocked
        let moves = s.legal_moves(Dice(1));
        assert!(
            moves
                .iter()
                .all(|m| m.to != PieceLocation::OnBoard(entry_sq)
                    || m.from != PieceLocation::Unplayed),
            "should not be able to enter when entry square is friendly-occupied"
        );
    }

    #[test]
    fn test_bearing_off_requires_exact_roll() {
        let rules = GameRules::finkel();
        // Place a piece at the last path square (index 13)
        let state = GameState::new(&rules);
        let path = rules.path_for(Player::Player1);
        let last_sq = path.get(13).unwrap();
        let mut s = state.clone();
        s.board.set(last_sq, Some(Piece::new(Player::Player1, 0)));
        s.unplayed[Player::Player1.index()] = 6; // 1 on board, 6 unplayed
                                                 // Roll=1 from index 13 → index 14 = path.len() → exact bear-off
        let moves = s.legal_moves(Dice(1));
        assert!(moves.iter().any(|m| m.from == PieceLocation::OnBoard(last_sq)
            && m.to == PieceLocation::Scored),
            "roll=1 from last square should produce a bear-off move");
    }

    #[test]
    fn test_overshoot_not_allowed() {
        let rules = GameRules::finkel();
        let path = rules.path_for(Player::Player1);
        let last_sq = path.get(13).unwrap();
        let mut s = GameState::new(&rules);
        s.board.set(last_sq, Some(Piece::new(Player::Player1, 0)));
        s.unplayed[Player::Player1.index()] = 6;
        // Roll=2 from index 13 → index 15 > 14 → overshoot, illegal
        let moves = s.legal_moves(Dice(2));
        assert!(
            moves
                .iter()
                .all(|m| m.from != PieceLocation::OnBoard(last_sq)),
            "roll=2 from last square should not produce any move for that piece"
        );
    }

    #[test]
    fn test_capture_on_shared_row_is_legal() {
        let rules = GameRules::finkel();
        // Player1 at path index 4 (=(1,0)), Player2 at path index 5 (=(1,1))
        // Player1 rolls 1 → lands on (1,1) where Player2 sits → legal capture
        let mut s = GameState::new(&rules);
        let p1_sq = rules.path_for(Player::Player1).get(4).unwrap(); // (1,0)
        let p2_sq = rules.path_for(Player::Player2).get(5).unwrap(); // (1,1)
        assert_eq!(p1_sq, Square::new(1, 0));
        assert_eq!(p2_sq, Square::new(1, 1));
        s.board.set(p1_sq, Some(Piece::new(Player::Player1, 0)));
        s.board.set(p2_sq, Some(Piece::new(Player::Player2, 0)));
        s.unplayed = [6, 6];
        let moves = s.legal_moves(Dice(1));
        assert!(
            moves.iter().any(|m| m.from == PieceLocation::OnBoard(p1_sq)
                && m.to == PieceLocation::OnBoard(p2_sq)),
            "landing on opponent's non-rosette square should be legal"
        );
    }

    #[test]
    fn test_capture_blocked_by_rosette() {
        let rules = GameRules::finkel();
        // (1,3) is a rosette. Place Player2 on it. Player1 tries to land there.
        // Player1 at path index 6 (=(1,2)). Roll 1 → (1,3). Blocked.
        let mut s = GameState::new(&rules);
        let p1_sq = Square::new(1, 2); // path index 6 for P1
        let rosette = Square::new(1, 3);
        assert!(rules.board_shape.is_rosette(rosette));
        s.board.set(p1_sq, Some(Piece::new(Player::Player1, 0)));
        s.board.set(rosette, Some(Piece::new(Player::Player2, 0)));
        s.unplayed = [6, 6];
        let moves = s.legal_moves(Dice(1));
        assert!(
            moves
                .iter()
                .all(|m| m.to != PieceLocation::OnBoard(rosette)),
            "should not be able to capture opponent on rosette"
        );
    }

    #[test]
    fn test_friendly_square_blocked() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        // Two Player1 pieces: piece 0 at path[4]=(1,0), piece 1 at path[5]=(1,1)
        let sq0 = Square::new(1, 0);
        let sq1 = Square::new(1, 1);
        s.board.set(sq0, Some(Piece::new(Player::Player1, 0)));
        s.board.set(sq1, Some(Piece::new(Player::Player1, 1)));
        s.unplayed[Player::Player1.index()] = 5;
        let moves = s.legal_moves(Dice(1));
        // piece 0 at sq0 cannot move to sq1 (friendly)
        assert!(
            moves
                .iter()
                .all(|m| !(m.from == PieceLocation::OnBoard(sq0)
                    && m.to == PieceLocation::OnBoard(sq1))),
            "should not move to friendly-occupied square"
        );
    }

    #[test]
    fn test_rosette_safe_from_capture() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        let rosette = Square::new(2, 0); // P1 path index 3, also a rosette
        assert!(rules.board_shape.is_rosette(rosette));
        // Place Player1 piece on the rosette
        s.board.set(rosette, Some(Piece::new(Player::Player1, 0)));
        s.unplayed[Player::Player1.index()] = 6;
        // Switch to Player2's turn
        s.current_player = Player::Player2;
        // Player2 at path index 2 (=(0,1)), roll 1 → path[3]=(0,0)
        // That's NOT the rosette above (which is (2,0)) — let's find P2's path
        // P2's path index 3 = (0,0). That's not a rosette.
        // Let's place P1 on (1,3) which is shared AND a rosette, and P2 tries to reach it.
        // P2 at path index 6 (=(1,2)), roll 1 → path[7]=(1,3) — the shared rosette.
        let p1_rosette = Square::new(1, 3);
        assert!(rules.board_shape.is_rosette(p1_rosette));
        let p2_start = rules.path_for(Player::Player2).get(6).unwrap(); // (1,2)
                                                                        // Reset board
        let mut s2 = GameState::new(&rules);
        s2.board.set(p2_start, Some(Piece::new(Player::Player2, 0)));
        s2.board
            .set(p1_rosette, Some(Piece::new(Player::Player1, 0)));
        s2.unplayed = [6, 6];
        s2.current_player = Player::Player2;
        let moves = s2.legal_moves(Dice(1));
        assert!(
            moves
                .iter()
                .all(|m| m.to != PieceLocation::OnBoard(p1_rosette)),
            "Player2 should not be able to capture Player1 on the shared rosette"
        );
    }

    // ── Move application ─────────────────────────────────────────────────────

    #[test]
    fn test_piece_advances_correct_number_of_squares() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        // Player1 piece at path[4]=(1,0). Roll 3 → should land at path[7]=(1,3).
        let start_sq = rules.path_for(Player::Player1).get(4).unwrap();
        let expected_sq = rules.path_for(Player::Player1).get(7).unwrap();
        s.board.set(start_sq, Some(Piece::new(Player::Player1, 0)));
        s.unplayed[Player::Player1.index()] = 6;
        let moves = s.legal_moves(Dice(3));
        let mv = moves
            .iter()
            .find(|m| m.from == PieceLocation::OnBoard(start_sq))
            .unwrap()
            .clone();
        assert_eq!(mv.to, PieceLocation::OnBoard(expected_sq));
        let result = s.apply_move(mv);
        assert_eq!(
            result.new_state.board.get(expected_sq),
            Some(Piece::new(Player::Player1, 0))
        );
        assert_eq!(result.new_state.board.get(start_sq), None);
    }

    #[test]
    fn test_capture_returns_opponent_piece_to_pool() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        let p1_sq = Square::new(1, 0); // P1 at path[4]
        let p2_sq = Square::new(1, 1); // P2 at path[5]
        s.board.set(p1_sq, Some(Piece::new(Player::Player1, 0)));
        s.board.set(p2_sq, Some(Piece::new(Player::Player2, 0)));
        s.unplayed = [6, 6];
        let moves = s.legal_moves(Dice(1));
        let capture_mv = moves
            .iter()
            .find(|m| {
                m.from == PieceLocation::OnBoard(p1_sq) && m.to == PieceLocation::OnBoard(p2_sq)
            })
            .unwrap()
            .clone();
        let result = s.apply_move(capture_mv);
        // Opponent's unplayed count increases by 1
        assert_eq!(result.new_state.unplayed[Player::Player2.index()], 7);
        // Capturing piece is now on the square
        assert_eq!(
            result.new_state.board.get(p2_sq),
            Some(Piece::new(Player::Player1, 0))
        );
        // Captured piece is gone from the board
        assert_eq!(result.new_state.board.get(p1_sq), None);
    }

    #[test]
    fn test_capture_metadata_is_set() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        let p1_sq = Square::new(1, 0);
        let p2_sq = Square::new(1, 1);
        let p2_piece = Piece::new(Player::Player2, 0);
        s.board.set(p1_sq, Some(Piece::new(Player::Player1, 0)));
        s.board.set(p2_sq, Some(p2_piece));
        s.unplayed = [6, 6];
        let moves = s.legal_moves(Dice(1));
        let capture_mv = moves
            .iter()
            .find(|m| m.to == PieceLocation::OnBoard(p2_sq))
            .unwrap()
            .clone();
        let result = s.apply_move(capture_mv);
        assert_eq!(result.captured, Some(p2_piece));
    }

    #[test]
    fn test_rosette_grants_extra_turn() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        // P1 at path[2]=(2,1). Roll 1 → path[3]=(2,0), which is a rosette.
        let start_sq = rules.path_for(Player::Player1).get(2).unwrap();
        let rosette_sq = rules.path_for(Player::Player1).get(3).unwrap();
        assert!(rules.board_shape.is_rosette(rosette_sq));
        s.board.set(start_sq, Some(Piece::new(Player::Player1, 0)));
        s.unplayed[Player::Player1.index()] = 6;
        let moves = s.legal_moves(Dice(1));
        let mv = moves
            .iter()
            .find(|m| m.to == PieceLocation::OnBoard(rosette_sq))
            .unwrap()
            .clone();
        let result = s.apply_move(mv);
        // Same player still has the turn
        assert_eq!(result.new_state.current_player, Player::Player1);
        assert_eq!(result.new_state.phase, GamePhase::WaitingForRoll);
        assert!(result.landed_on_rosette);
    }

    #[test]
    fn test_non_rosette_passes_turn() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        // P1 enters at path[0]=(2,3) with roll=1.
        s.unplayed[Player::Player1.index()] = 7;
        let moves = s.legal_moves(Dice(1));
        let enter_mv = moves
            .iter()
            .find(|m| m.from == PieceLocation::Unplayed)
            .unwrap()
            .clone();
        let target = rules.path_for(Player::Player1).get(0).unwrap(); // (2,3) — not a rosette
        assert!(!rules.board_shape.is_rosette(target));
        assert_eq!(enter_mv.to, PieceLocation::OnBoard(target));
        let result = s.apply_move(enter_mv);
        assert_eq!(result.new_state.current_player, Player::Player2);
        assert!(!result.landed_on_rosette);
    }

    #[test]
    fn test_scoring_a_piece() {
        let rules = GameRules::finkel();
        let mut s = GameState::new(&rules);
        let last_sq = rules.path_for(Player::Player1).get(13).unwrap();
        s.board.set(last_sq, Some(Piece::new(Player::Player1, 0)));
        s.unplayed[Player::Player1.index()] = 6;
        let moves = s.legal_moves(Dice(1));
        let bear_off = moves
            .iter()
            .find(|m| m.to == PieceLocation::Scored)
            .unwrap()
            .clone();
        let result = s.apply_move(bear_off);
        assert!(result.piece_scored);
        assert_eq!(result.new_state.scored[Player::Player1.index()], 1);
    }

    #[test]
    fn test_win_detection_all_7_scored() {
        let rules = GameRules::finkel();
        // Build a state where Player1 has 6 scored and 1 piece at the last square
        let mut s = GameState::new(&rules);
        s.scored[Player::Player1.index()] = 6;
        s.unplayed[Player::Player1.index()] = 0;
        let last_sq = rules.path_for(Player::Player1).get(13).unwrap();
        s.board.set(last_sq, Some(Piece::new(Player::Player1, 0)));
        let moves = s.legal_moves(Dice(1));
        let bear_off = moves
            .iter()
            .find(|m| m.to == PieceLocation::Scored)
            .unwrap()
            .clone();
        let result = s.apply_move(bear_off);
        assert!(result.game_over);
        assert!(result.new_state.is_finished());
        assert_eq!(result.new_state.winner(), Some(Player::Player1));
    }

    #[test]
    #[should_panic]
    fn test_apply_illegal_move_panics() {
        let rules = GameRules::finkel();
        let state = GameState::new(&rules);
        // Try to move a piece that doesn't exist on the board
        let illegal = Move {
            piece: Piece::new(Player::Player1, 0),
            from: PieceLocation::OnBoard(Square::new(1, 0)), // nothing here
            to: PieceLocation::OnBoard(Square::new(1, 1)),
        };
        state.apply_move(illegal);
    }

    // ── Turn forfeiture ──────────────────────────────────────────────────────

    #[test]
    fn test_forfeit_turn_when_no_legal_moves() {
        let rules = GameRules::finkel();
        let s = GameState::new(&rules);
        // Roll 0 always produces no legal moves
        assert!(s.legal_moves(Dice(0)).is_empty());
        // pass_turn hands the turn to the opponent
        let after_forfeit = s.pass_turn();
        assert_eq!(after_forfeit.current_player, Player::Player2);
        assert_eq!(after_forfeit.phase, GamePhase::WaitingForRoll);
        // Original state unchanged (immutable)
        assert_eq!(s.current_player, Player::Player1);
    }
}
