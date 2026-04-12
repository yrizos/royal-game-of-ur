# ur-core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fill in all `todo!()` stubs in `ur-core` to produce a fully working Royal Game of Ur engine with all 40 tests passing and a 1000-game simulation completing without errors.

**Architecture:** Work in strict dependency order — player/dice first (no deps), then board geometry, then game state machine (`legal_moves` and `apply_move` are the hardest parts), then the expectiminimax AI, then wire up the integration test. Each task fills in test bodies and implementations together; the test goes from a `todo!()` panic to a real assertion failure to passing.

**Tech Stack:** Rust stable, `rand 0.8`, `ur-core` workspace crate at `/Users/Rizos/fun/royal-game-of-ur`.

---

## File Map

| File | Changes |
|------|---------|
| `ur-core/src/player.rs` | Implement `opponent()`, `index()`, fill 5 tests |
| `ur-core/src/dice.rs` | Implement `Dice::roll()`, fill 4 tests |
| `ur-core/src/board.rs` | Implement `BoardShape::finkel/is_valid/is_rosette`, fill 8 tests |
| `ur-core/src/state.rs` | Implement everything + add `pass_turn()`, fill 19 tests |
| `ur-core/src/ai.rs` | Implement `evaluate`, `decision_node`, `chance_node`, `choose_move`, fill 4 tests |
| `ur-core/tests/simulation.rs` | Fix forfeit placeholder, fill simulation test |

---

### Task 1: player.rs — Player::opponent, Player::index

**Files:**
- Modify: `ur-core/src/player.rs`

- [ ] **Step 1: Fill in the test bodies and implementations together**

Replace the entire contents of `ur-core/src/player.rs` with:

```rust
/// One of the two players.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    /// Returns the opposing player.
    pub fn opponent(self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }

    /// Returns the zero-based index used for array lookup (Player1 = 0, Player2 = 1).
    pub fn index(self) -> usize {
        match self {
            Player::Player1 => 0,
            Player::Player2 => 1,
        }
    }
}

/// A single game piece, identified by its owner and a 0-based index (0–6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub player: Player,
    pub index: u8,
}

impl Piece {
    pub fn new(player: Player, index: u8) -> Self {
        Self { player, index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player1_opponent_is_player2() {
        assert_eq!(Player::Player1.opponent(), Player::Player2);
    }

    #[test]
    fn test_player2_opponent_is_player1() {
        assert_eq!(Player::Player2.opponent(), Player::Player1);
    }

    #[test]
    fn test_opponent_is_involution() {
        assert_eq!(Player::Player1.opponent().opponent(), Player::Player1);
        assert_eq!(Player::Player2.opponent().opponent(), Player::Player2);
    }

    #[test]
    fn test_player_index_player1_is_0() {
        assert_eq!(Player::Player1.index(), 0);
    }

    #[test]
    fn test_player_index_player2_is_1() {
        assert_eq!(Player::Player2.index(), 1);
    }
}
```

- [ ] **Step 2: Run the player tests**

```bash
cargo test -p ur-core player
```

Expected output:
```
test player::tests::test_opponent_is_involution ... ok
test player::tests::test_player1_opponent_is_player2 ... ok
test player::tests::test_player2_opponent_is_player1 ... ok
test player::tests::test_player_index_player1_is_0 ... ok
test player::tests::test_player_index_player2_is_1 ... ok

test result: ok. 5 passed; 0 failed
```

- [ ] **Step 3: Commit**

```bash
git add ur-core/src/player.rs
git commit -m "feat: implement Player::opponent and Player::index"
```

---

### Task 2: dice.rs — Dice::roll

**Files:**
- Modify: `ur-core/src/dice.rs`

- [ ] **Step 1: Fill in `Dice::roll` and all test bodies**

Replace the entire contents of `ur-core/src/dice.rs` with:

```rust
use rand::Rng;

/// The result of rolling four binary tetrahedral dice, producing a value 0–4.
///
/// Each die contributes 1 if it lands marked-side up. The total is the sum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Dice(pub u8);

impl Dice {
    /// Generates a random dice roll using the caller-provided RNG.
    ///
    /// Simulates four independent fair binary dice (each 50/50), returning
    /// their sum — a binomial B(4, 0.5) distribution producing values 0–4.
    pub fn roll(rng: &mut impl Rng) -> Self {
        let count: u8 = (0..4).map(|_| rng.gen::<bool>() as u8).sum();
        Dice(count)
    }

    /// Returns the numeric value of this roll (0–4).
    pub fn value(self) -> u8 {
        self.0
    }
}

/// Probability of each dice outcome. Index is the roll value (0–4).
///
/// | Roll | Ways | Probability |
/// |------|------|-------------|
/// | 0    | 1    | 1/16        |
/// | 1    | 4    | 4/16        |
/// | 2    | 6    | 6/16        |
/// | 3    | 4    | 4/16        |
/// | 4    | 1    | 1/16        |
pub const DICE_PROBABILITIES: [f64; 5] =
    [1.0 / 16.0, 4.0 / 16.0, 6.0 / 16.0, 4.0 / 16.0, 1.0 / 16.0];

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_dice_value_always_0_to_4() {
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..10_000 {
            let roll = Dice::roll(&mut rng);
            assert!(roll.value() <= 4, "roll {} out of range", roll.value());
        }
    }

    #[test]
    fn test_dice_probabilities_sum_to_1() {
        let sum: f64 = DICE_PROBABILITIES.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "probabilities sum to {}, not 1.0", sum);
    }

    #[test]
    fn test_dice_probability_distribution_matches_binomial() {
        let mut rng = StdRng::seed_from_u64(42);
        let n = 100_000usize;
        let mut counts = [0usize; 5];
        for _ in 0..n {
            counts[Dice::roll(&mut rng).value() as usize] += 1;
        }
        for (val, &count) in counts.iter().enumerate() {
            let observed = count as f64 / n as f64;
            let expected = DICE_PROBABILITIES[val];
            let diff = (observed - expected).abs();
            assert!(
                diff < 0.02,
                "roll {} observed {:.4} expected {:.4} diff {:.4} > 2%",
                val, observed, expected, diff
            );
        }
    }

    #[test]
    fn test_dice_roll_is_deterministic_given_seed() {
        let mut rng_a = StdRng::seed_from_u64(999);
        let mut rng_b = StdRng::seed_from_u64(999);
        for _ in 0..1000 {
            assert_eq!(Dice::roll(&mut rng_a), Dice::roll(&mut rng_b));
        }
    }
}
```

- [ ] **Step 2: Run the dice tests**

```bash
cargo test -p ur-core dice
```

Expected:
```
test dice::tests::test_dice_probabilities_sum_to_1 ... ok
test dice::tests::test_dice_probability_distribution_matches_binomial ... ok
test dice::tests::test_dice_roll_is_deterministic_given_seed ... ok
test dice::tests::test_dice_value_always_0_to_4 ... ok

test result: ok. 4 passed; 0 failed
```

- [ ] **Step 3: Commit**

```bash
git add ur-core/src/dice.rs
git commit -m "feat: implement Dice::roll with binomial B(4,0.5) sampling"
```

---

### Task 3: board.rs — BoardShape and Path tests

**Files:**
- Modify: `ur-core/src/board.rs`

The `Path` type is already fully implemented (all its methods are trivial wrappers over `Vec`). This task implements the three `BoardShape` methods and fills in the eight test bodies.

- [ ] **Step 1: Replace the entire contents of `ur-core/src/board.rs`**

```rust
/// A position on the board identified by row and column.
///
/// Valid squares follow the Finkel 3×8 layout with four removed squares:
/// row 0 and row 2 have no columns 4 or 5.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Square {
    pub row: u8,
    pub col: u8,
}

impl Square {
    pub fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }
}

/// Defines which squares exist on the board and which are rosettes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoardShape {
    valid_squares: Vec<Square>,
    rosettes: Vec<Square>,
}

impl BoardShape {
    /// Returns the standard 20-square Finkel board shape.
    ///
    /// Valid squares: row 0 cols 0-3 and 6-7; row 1 cols 0-7; row 2 cols 0-3 and 6-7.
    /// Rosettes: (0,0), (0,6), (1,3), (2,0), (2,6).
    pub fn finkel() -> Self {
        let mut valid_squares = Vec::with_capacity(20);
        for row in 0u8..3 {
            for col in 0u8..8 {
                if row != 1 && (col == 4 || col == 5) {
                    continue; // these squares don't exist
                }
                valid_squares.push(Square::new(row, col));
            }
        }
        let rosettes = vec![
            Square::new(0, 0),
            Square::new(0, 6),
            Square::new(1, 3),
            Square::new(2, 0),
            Square::new(2, 6),
        ];
        Self { valid_squares, rosettes }
    }

    /// Returns true if the given square exists on this board.
    pub fn is_valid(&self, sq: Square) -> bool {
        self.valid_squares.contains(&sq)
    }

    /// Returns true if the given square is a rosette.
    pub fn is_rosette(&self, sq: Square) -> bool {
        self.rosettes.contains(&sq)
    }

    pub fn valid_squares(&self) -> &[Square] {
        &self.valid_squares
    }

    pub fn rosettes(&self) -> &[Square] {
        &self.rosettes
    }
}

/// An ordered sequence of squares defining a player's route from entry to exit.
///
/// Does not include the logical off-board entry or exit positions; those are
/// handled by `PieceLocation` in the state module.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Path {
    squares: Vec<Square>,
}

impl Path {
    pub fn new(squares: Vec<Square>) -> Self {
        Self { squares }
    }

    pub fn squares(&self) -> &[Square] {
        &self.squares
    }

    pub fn len(&self) -> usize {
        self.squares.len()
    }

    pub fn is_empty(&self) -> bool {
        self.squares.is_empty()
    }

    /// Returns the square at position `index` along the path, if it exists.
    pub fn get(&self, index: usize) -> Option<Square> {
        self.squares.get(index).copied()
    }

    /// Returns the index of `sq` in this path, if present.
    pub fn index_of(&self, sq: Square) -> Option<usize> {
        self.squares.iter().position(|&s| s == sq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player1_path() -> Path {
        Path::new(vec![
            Square::new(2, 3), Square::new(2, 2), Square::new(2, 1), Square::new(2, 0),
            Square::new(1, 0), Square::new(1, 1), Square::new(1, 2), Square::new(1, 3),
            Square::new(1, 4), Square::new(1, 5), Square::new(1, 6), Square::new(1, 7),
            Square::new(2, 7), Square::new(2, 6),
        ])
    }

    fn player2_path() -> Path {
        Path::new(vec![
            Square::new(0, 3), Square::new(0, 2), Square::new(0, 1), Square::new(0, 0),
            Square::new(1, 0), Square::new(1, 1), Square::new(1, 2), Square::new(1, 3),
            Square::new(1, 4), Square::new(1, 5), Square::new(1, 6), Square::new(1, 7),
            Square::new(0, 7), Square::new(0, 6),
        ])
    }

    #[test]
    fn test_path_player1_covers_correct_squares() {
        let path = player1_path();
        assert_eq!(path.len(), 14);
        // Private leg: row 2 descending to col 0
        assert_eq!(path.get(0), Some(Square::new(2, 3)));
        assert_eq!(path.get(1), Some(Square::new(2, 2)));
        assert_eq!(path.get(2), Some(Square::new(2, 1)));
        assert_eq!(path.get(3), Some(Square::new(2, 0)));
        // Shared middle row: col 0 through 7
        for col in 0u8..8 {
            assert_eq!(path.get(4 + col as usize), Some(Square::new(1, col)));
        }
        // Exit leg: row 2 cols 7 then 6
        assert_eq!(path.get(12), Some(Square::new(2, 7)));
        assert_eq!(path.get(13), Some(Square::new(2, 6)));
    }

    #[test]
    fn test_path_player2_covers_correct_squares() {
        let path = player2_path();
        assert_eq!(path.len(), 14);
        // Private leg: row 0 descending to col 0
        assert_eq!(path.get(0), Some(Square::new(0, 3)));
        assert_eq!(path.get(1), Some(Square::new(0, 2)));
        assert_eq!(path.get(2), Some(Square::new(0, 1)));
        assert_eq!(path.get(3), Some(Square::new(0, 0)));
        // Shared middle row: col 0 through 7
        for col in 0u8..8 {
            assert_eq!(path.get(4 + col as usize), Some(Square::new(1, col)));
        }
        // Exit leg: row 0 cols 7 then 6
        assert_eq!(path.get(12), Some(Square::new(0, 7)));
        assert_eq!(path.get(13), Some(Square::new(0, 6)));
    }

    #[test]
    fn test_shared_row_is_columns_0_through_7() {
        let p1 = player1_path();
        let p2 = player2_path();
        for col in 0u8..8 {
            let sq = Square::new(1, col);
            assert!(
                p1.index_of(sq).is_some(),
                "Player1 path missing shared square (1,{})", col
            );
            assert!(
                p2.index_of(sq).is_some(),
                "Player2 path missing shared square (1,{})", col
            );
        }
    }

    #[test]
    fn test_rosette_positions() {
        let shape = BoardShape::finkel();
        let expected_rosettes = vec![
            Square::new(0, 0),
            Square::new(0, 6),
            Square::new(1, 3),
            Square::new(2, 0),
            Square::new(2, 6),
        ];
        for sq in &expected_rosettes {
            assert!(shape.is_rosette(*sq), "expected rosette at ({},{})", sq.row, sq.col);
        }
        // Verify no extra rosettes
        assert_eq!(shape.rosettes().len(), 5);
    }

    #[test]
    fn test_invalid_squares_row0_cols_4_5() {
        let shape = BoardShape::finkel();
        assert!(!shape.is_valid(Square::new(0, 4)), "(0,4) should be invalid");
        assert!(!shape.is_valid(Square::new(0, 5)), "(0,5) should be invalid");
    }

    #[test]
    fn test_invalid_squares_row2_cols_4_5() {
        let shape = BoardShape::finkel();
        assert!(!shape.is_valid(Square::new(2, 4)), "(2,4) should be invalid");
        assert!(!shape.is_valid(Square::new(2, 5)), "(2,5) should be invalid");
    }

    #[test]
    fn test_valid_square_count_is_20() {
        let shape = BoardShape::finkel();
        assert_eq!(shape.valid_squares().len(), 20);
    }

    #[test]
    fn test_path_index_of_roundtrip() {
        let path = player1_path();
        for (i, &sq) in path.squares().iter().enumerate() {
            assert_eq!(path.index_of(sq), Some(i));
            assert_eq!(path.get(path.index_of(sq).unwrap()), Some(sq));
        }
    }
}
```

- [ ] **Step 2: Run board tests**

```bash
cargo test -p ur-core board
```

Expected:
```
test board::tests::test_invalid_squares_row0_cols_4_5 ... ok
test board::tests::test_invalid_squares_row2_cols_4_5 ... ok
test board::tests::test_path_index_of_roundtrip ... ok
test board::tests::test_path_player1_covers_correct_squares ... ok
test board::tests::test_path_player2_covers_correct_squares ... ok
test board::tests::test_rosette_positions ... ok
test board::tests::test_shared_row_is_columns_0_through_7 ... ok
test board::tests::test_valid_square_count_is_20 ... ok

test result: ok. 8 passed; 0 failed
```

- [ ] **Step 3: Commit**

```bash
git add ur-core/src/board.rs
git commit -m "feat: implement BoardShape::finkel, is_valid, is_rosette and board tests"
```

---

### Task 4: state.rs — Board, GameRules, GameState basics

**Files:**
- Modify: `ur-core/src/state.rs`

This task implements:
- `Board::new`, `Board::get`, `Board::set`
- `GameRules::finkel`, `GameRules::path_for`
- `GameState::new`, `GameState::is_finished`, `GameState::winner`, `GameState::current_player`
- New public method: `GameState::pass_turn` (needed for forfeit and simulation)

The `legal_moves` and `apply_move` remain `todo!()` — they are Tasks 5 and 6.

- [ ] **Step 1: Replace the entire contents of `ur-core/src/state.rs`**

```rust
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
            Square::new(2, 3), Square::new(2, 2), Square::new(2, 1), Square::new(2, 0),
            Square::new(1, 0), Square::new(1, 1), Square::new(1, 2), Square::new(1, 3),
            Square::new(1, 4), Square::new(1, 5), Square::new(1, 6), Square::new(1, 7),
            Square::new(2, 7), Square::new(2, 6),
        ]);
        let path_player2 = Path::new(vec![
            Square::new(0, 3), Square::new(0, 2), Square::new(0, 1), Square::new(0, 0),
            Square::new(1, 0), Square::new(1, 1), Square::new(1, 2), Square::new(1, 3),
            Square::new(1, 4), Square::new(1, 5), Square::new(1, 6), Square::new(1, 7),
            Square::new(0, 7), Square::new(0, 6),
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
        todo!()
    }

    /// Applies `mv` to this state and returns the result.
    ///
    /// # Panics
    ///
    /// Panics if `mv` is not a structurally valid move (piece not present at `from`).
    pub fn apply_move(&self, mv: Move) -> MoveResult {
        todo!()
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
    fn place_at_path_idx(state: &GameState, player: Player, piece_idx: u8, path_idx: usize) -> GameState {
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
        assert!(moves.iter().any(|m| m.from == PieceLocation::Unplayed
            && m.to == PieceLocation::OnBoard(entry_sq)));
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
        assert!(moves.iter().all(|m| m.to != PieceLocation::OnBoard(entry_sq)
            || m.from != PieceLocation::Unplayed),
            "should not be able to enter when entry square is friendly-occupied");
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
        assert!(moves.iter().all(|m| m.from != PieceLocation::OnBoard(last_sq)),
            "roll=2 from last square should not produce any move for that piece");
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
        assert!(moves.iter().any(|m| m.from == PieceLocation::OnBoard(p1_sq)
            && m.to == PieceLocation::OnBoard(p2_sq)),
            "landing on opponent's non-rosette square should be legal");
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
        assert!(moves.iter().all(|m| m.to != PieceLocation::OnBoard(rosette)),
            "should not be able to capture opponent on rosette");
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
        assert!(moves.iter().all(|m| !(m.from == PieceLocation::OnBoard(sq0)
            && m.to == PieceLocation::OnBoard(sq1))),
            "should not move to friendly-occupied square");
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
        s2.board.set(p1_rosette, Some(Piece::new(Player::Player1, 0)));
        s2.unplayed = [6, 6];
        s2.current_player = Player::Player2;
        let moves = s2.legal_moves(Dice(1));
        assert!(moves.iter().all(|m| m.to != PieceLocation::OnBoard(p1_rosette)),
            "Player2 should not be able to capture Player1 on the shared rosette");
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
        let mv = moves.iter().find(|m| m.from == PieceLocation::OnBoard(start_sq)).unwrap().clone();
        assert_eq!(mv.to, PieceLocation::OnBoard(expected_sq));
        let result = s.apply_move(mv);
        assert_eq!(result.new_state.board.get(expected_sq), Some(Piece::new(Player::Player1, 0)));
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
        let capture_mv = moves.iter()
            .find(|m| m.from == PieceLocation::OnBoard(p1_sq)
                && m.to == PieceLocation::OnBoard(p2_sq))
            .unwrap().clone();
        let result = s.apply_move(capture_mv);
        // Opponent's unplayed count increases by 1
        assert_eq!(result.new_state.unplayed[Player::Player2.index()], 7);
        // Capturing piece is now on the square
        assert_eq!(result.new_state.board.get(p2_sq), Some(Piece::new(Player::Player1, 0)));
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
        let capture_mv = moves.iter()
            .find(|m| m.to == PieceLocation::OnBoard(p2_sq)).unwrap().clone();
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
        let mv = moves.iter().find(|m| m.to == PieceLocation::OnBoard(rosette_sq)).unwrap().clone();
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
        let enter_mv = moves.iter().find(|m| m.from == PieceLocation::Unplayed).unwrap().clone();
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
        let bear_off = moves.iter().find(|m| m.to == PieceLocation::Scored).unwrap().clone();
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
        let bear_off = moves.iter().find(|m| m.to == PieceLocation::Scored).unwrap().clone();
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
        let mut s = GameState::new(&rules);
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
```

- [ ] **Step 2: Run all state tests that don't depend on legal_moves or apply_move**

```bash
cargo test -p ur-core state::tests::test_no_legal_moves_roll_0
cargo test -p ur-core state::tests::test_forfeit_turn_when_no_legal_moves
```

Both should FAIL because `legal_moves` is still `todo!()`. That's expected — they'll pass after Task 5.

- [ ] **Step 3: Run full test suite to confirm previous tasks still pass**

```bash
cargo test -p ur-core player dice board
```

Expected: 17 passed, 0 failed.

- [ ] **Step 4: Commit**

```bash
git add ur-core/src/state.rs
git commit -m "feat: implement Board, GameRules::finkel, GameState basics, and pass_turn"
```

---

### Task 5: state.rs — legal_moves

**Files:**
- Modify: `ur-core/src/state.rs`

- [ ] **Step 1: Replace the `legal_moves` stub with the implementation**

Find the `legal_moves` method and replace its body:

```rust
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
```

- [ ] **Step 2: Run the legal-move tests**

```bash
cargo test -p ur-core state::tests::test_no_legal_moves_roll_0 \
  state::tests::test_no_legal_moves_all_blocked \
  state::tests::test_can_enter_piece_from_pool \
  state::tests::test_cannot_enter_when_entry_occupied_by_friendly \
  state::tests::test_bearing_off_requires_exact_roll \
  state::tests::test_overshoot_not_allowed \
  state::tests::test_capture_on_shared_row_is_legal \
  state::tests::test_capture_blocked_by_rosette \
  state::tests::test_friendly_square_blocked \
  state::tests::test_rosette_safe_from_capture \
  state::tests::test_forfeit_turn_when_no_legal_moves
```

Expected: all 11 pass.

- [ ] **Step 3: Commit**

```bash
git add ur-core/src/state.rs
git commit -m "feat: implement GameState::legal_moves"
```

---

### Task 6: state.rs — apply_move

**Files:**
- Modify: `ur-core/src/state.rs`

- [ ] **Step 1: Replace the `apply_move` stub with the implementation**

```rust
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
```

- [ ] **Step 2: Run the move-application tests**

```bash
cargo test -p ur-core state
```

Expected: all 19 state tests pass.

- [ ] **Step 3: Commit**

```bash
git add ur-core/src/state.rs
git commit -m "feat: implement GameState::apply_move"
```

---

### Task 7: ai.rs — evaluate, expectiminimax, choose_move

**Files:**
- Modify: `ur-core/src/ai.rs`

- [ ] **Step 1: Replace the entire contents of `ur-core/src/ai.rs`**

```rust
use crate::dice::{Dice, DICE_PROBABILITIES};
use crate::state::{GamePhase, GameState, Move};

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
    assert!(!moves.is_empty(), "choose_move called with no legal moves for roll {:?}", roll);

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
        s.board.set(capture_from, Some(Piece::new(Player::Player1, 0)));
        s.board.set(capture_to, Some(Piece::new(Player::Player2, 0)));
        s.board.set(neutral_from, Some(Piece::new(Player::Player1, 1)));
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
        s.board.set(rosette_from, Some(Piece::new(Player::Player1, 0)));
        s.board.set(neutral_from, Some(Piece::new(Player::Player1, 1)));
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
```

- [ ] **Step 2: Run the AI tests**

```bash
cargo test -p ur-core ai
```

Expected: all 4 pass.

- [ ] **Step 3: Commit**

```bash
git add ur-core/src/ai.rs
git commit -m "feat: implement expectiminimax AI with board evaluation heuristic"
```

---

### Task 8: simulation.rs — wire up 1000-game test

**Files:**
- Modify: `ur-core/tests/simulation.rs`

- [ ] **Step 1: Replace the entire contents of `ur-core/tests/simulation.rs`**

```rust
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
```

- [ ] **Step 2: Run the simulation**

```bash
cargo test -p ur-core --test simulation -- --nocapture
```

Expected:
```
test test_1000_random_games_all_complete_with_valid_winner ... ok

test result: ok. 1 passed; 0 failed
```

This may take a few seconds.

- [ ] **Step 3: Run the complete test suite**

```bash
cargo test --workspace
```

Expected:
```
test result: ok. 40 passed; 0 failed; 0 ignored
```

(The simulation integration test runs separately, so the total may show as 40 + 1 = 41 tests.)

- [ ] **Step 4: Formatting and lints**

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

Fix any clippy warnings before committing.

- [ ] **Step 5: Commit**

```bash
git add ur-core/tests/simulation.rs
git commit -m "feat: wire up 1000-game simulation test — ur-core implementation complete"
```

---

## Self-Review

**Spec coverage:**
- Board geometry (20 squares, missing cols) → Task 3 ✓
- Rosette positions and behavior → Tasks 3, 5, 6 ✓
- Both player paths (14 squares each, shared middle row) → Tasks 3, 4 ✓
- Dice B(4, 0.5) distribution → Task 2 ✓
- Move rules (entry, advance, bear-off, capture, rosette safety, friendly block, overshoot) → Tasks 5, 6 ✓
- Turn flow (roll, legal moves, apply, extra turn on rosette, forfeit on empty) → Tasks 5, 6 ✓
- Win condition (all 7 pieces scored) → Task 6 ✓
- AI (expectiminimax, evaluate, depth-configurable) → Task 7 ✓
- `GameState::new`, `is_finished`, `winner`, `current_player` → Task 4 ✓
- `Dice::roll` → Task 2 ✓
- `pass_turn` (forfeit mechanism) → Task 4 ✓
- 1000-game simulation → Task 8 ✓

**Placeholder scan:** None found.

**Type consistency check:**
- `Dice(v)` used in ai.rs matches the `Dice` newtype from dice.rs ✓
- `PieceLocation::OnBoard(sq)` used consistently in all test assertions ✓
- `Player::index()` used for array indexing throughout state.rs ✓
- `pass_turn()` defined in Task 4, used in Tasks 7 and 8 ✓
- `next_entering_piece()` and `can_land_on()` defined as private helpers in Task 4, used in Task 5 ✓
- `move_score` helper defined in ai.rs, used by `choose_move`, `chance_node`, `decision_node` ✓
