# The Royal Game of Ur: Core Library Specification

## Purpose

`ur-core` is a Rust library crate that implements the complete game logic for the Royal Game of Ur. It is designed to be consumed by any frontend (desktop, web, mobile, terminal) without containing any rendering, input handling, or I/O code. The library is a pure computation engine: it accepts data, processes game rules, and returns data.

## Ruleset: Finkel (Default)

The default ruleset follows Irving Finkel's reconstruction, which is the most widely played modern interpretation. The library should be designed so that alternative rulesets (Masters, Aseb) can be added later through a configuration mechanism, but only the Finkel rules need to be implemented initially.

### Board Geometry

The board is a grid of 3 rows and 8 columns with four squares removed. Rows are numbered 0 (top), 1 (middle), 2 (bottom). Columns are numbered 0 through 7 from left to right.

Valid squares:

- Row 0: columns 0, 1, 2, 3, 6, 7 (columns 4 and 5 do not exist)
- Row 1: columns 0, 1, 2, 3, 4, 5, 6, 7 (all eight)
- Row 2: columns 0, 1, 2, 3, 6, 7 (columns 4 and 5 do not exist)

Total: 20 squares.

### Rosette Positions

Five squares are rosettes: (0, 0), (0, 6), (1, 3), (2, 0), (2, 6).

Rosette effects under Finkel rules:

- Landing on a rosette grants the player an extra turn.
- Pieces on rosettes are safe from capture.

### Piece Paths

Each player has 7 pieces. Pieces travel a fixed path from entry to exit.

Player 1 (bottom row entry) path, expressed as (row, column) coordinates:

```
Entry -> (2,3) -> (2,2) -> (2,1) -> (2,0)
      -> (1,0) -> (1,1) -> (1,2) -> (1,3) -> (1,4) -> (1,5) -> (1,6) -> (1,7)
      -> (2,7) -> (2,6) -> Exit
```

Player 2 (top row entry) path:

```
Entry -> (0,3) -> (0,2) -> (0,1) -> (0,0)
      -> (1,0) -> (1,1) -> (1,2) -> (1,3) -> (1,4) -> (1,5) -> (1,6) -> (1,7)
      -> (0,7) -> (0,6) -> Exit
```

The middle row (row 1, columns 0 through 7) is the bridge — the contested track shared by both players. Both players' pieces occupy the same physical squares on this row and can interact (captures).

The private rows (row 0 for Player 2, row 2 for Player 1) and the exit squares (columns 6 and 7 on private rows) are exclusive to each player.

### Dice

Four tetrahedral dice, each binary (marked or unmarked). Rolling all four produces a value from 0 to 4 by counting how many land marked-side up.

Probability distribution:

| Roll | Ways | Probability |
|------|------|-------------|
| 0    | 1    | 1/16 (6.25%) |
| 1    | 4    | 4/16 (25%) |
| 2    | 6    | 6/16 (37.5%) |
| 3    | 4    | 4/16 (25%) |
| 4    | 1    | 1/16 (6.25%) |

### Move Rules

- A player must move exactly one piece by the number rolled.
- A piece may not land on a square occupied by a friendly piece.
- A piece may land on a square occupied by an opponent's piece if that square is not a rosette. The opponent's piece is captured and returned to their unplayed pool.
- A piece may not land on a rosette occupied by an opponent's piece (rosettes are safe).
- To bear off (exit the board), a piece must land exactly on the exit square. Overshooting is not allowed.
- If no legal move exists, the turn is forfeited.
- Rolling a 0 always forfeits the turn (no piece can move zero squares).

### Turn Flow

1. The current player rolls the dice.
2. The library computes all legal moves for that roll.
3. If no legal moves exist, the turn passes to the opponent.
4. If legal moves exist, the player selects one. The library applies the move and produces a new game state.
5. If the moved piece landed on a rosette, the current player takes another turn (go to step 1).
6. Otherwise, the turn passes to the opponent.

### Win Condition

The first player to bear off all 7 pieces wins.

## Public API Surface

The API design is informed by the RoyalUr Java/Python libraries but adapted to Rust idioms. All public types should derive `Clone`, `Debug`, `PartialEq`, and `Eq` where appropriate. Types representing game state should also derive `Hash` for use in transposition tables.

### Core Types

**`Player`**: An enum with two variants (Player1, Player2).

**`Square`**: A struct representing a board position as (row, column).

**`BoardShape`**: Defines which squares exist on the board. The standard shape is the 20-square layout described above. This type exists to support future variant boards.

**`Path`**: An ordered sequence of squares defining a player's route from entry to exit. Includes the entry point (off-board) and exit point (off-board) as logical positions.

**`Dice`**: Represents a dice roll result (a value from 0 to 4).

**`Piece`**: Represents a single game piece, identified by player and an index (0 through 6).

**`GameState`**: The complete, immutable snapshot of a game at a point in time. Contains:

- The board: which pieces occupy which squares.
- Each player's unplayed piece count (pieces not yet entered).
- Each player's scored piece count (pieces that have exited).
- Whose turn it is.
- The current phase: waiting for roll, waiting for move, or game over.
- The current dice roll (if in the waiting-for-move phase).

**`Move`**: Represents a legal move. Contains the piece being moved, its origin (a square or "off-board"), and its destination (a square or "off-board/scored").

**`MoveResult`**: The outcome of applying a move. Contains the new game state and metadata about what happened: whether a capture occurred, whether a rosette was landed on (granting an extra turn), whether a piece was scored, and whether the game is now over.

**`GameRules`**: A configuration struct that bundles the board shape, paths, dice type, piece count, and rule flags (rosettes grant extra turn, rosettes are safe). The Finkel rules are the default. This is the extension point for future rulesets.

### Core Functions

**`GameState::new(rules: &GameRules) -> GameState`**: Creates a new game in its initial state with all pieces unplayed.

**`GameState::legal_moves(&self, roll: Dice) -> Vec<Move>`**: Given a dice roll, returns all legal moves for the current player. Returns an empty vec if no moves are possible.

**`GameState::apply_move(&self, mv: Move) -> MoveResult`**: Applies a move and returns the resulting state along with metadata. This function should validate that the move is legal and return an error (or panic, depending on design choice) if it is not.

**`GameState::is_finished(&self) -> bool`**: Returns true if a player has won.

**`GameState::winner(&self) -> Option<Player>`**: Returns the winning player, if the game is over.

**`GameState::current_player(&self) -> Player`**: Returns whose turn it is.

**`Dice::roll(rng: &mut impl Rng) -> Dice`**: Generates a random dice roll using the provided random number generator. The library does not own the RNG; the caller provides it. This keeps the core deterministic and testable.

### AI Module

**`ai::choose_move(state: &GameState, roll: Dice, depth: u32) -> Move`**: Given a game state and dice roll, returns the AI's chosen move. Uses expectiminimax search to the specified depth.

The AI module should include:

- A board evaluation heuristic that considers piece advancement, captures, rosette control, and piece safety.
- Expectiminimax search that handles chance nodes (dice rolls) interleaved with decision nodes (move selection).
- The ability to configure search depth, which acts as a difficulty setting.

The AI is part of the core library because it is pure computation over game states. A consumer who wants a simple AI opponent should not need to implement their own.

## Design Constraints

- No I/O of any kind. No file access, no network, no stdout.
- No platform-specific code. The crate must compile on any Rust target.
- No global mutable state. All state is passed explicitly through function arguments.
- The `rand` crate is the only external dependency.

## Testing Strategy

Game logic is ideal for comprehensive unit testing because every rule is deterministic given a fixed state and dice roll.

Required test coverage:

- Legal move generation for every edge case: no legal moves, bearing off with exact roll, overshooting blocked, rosette safety, capture on the bridge, capture blocked by rosette, friendly square blocked.
- Move application: piece advancement, capture and return to pool, rosette extra turn, scoring a piece, win detection.
- Path correctness: verify each player's path covers the right squares in the right order.
- AI: verify the AI selects reasonable moves at depth 1 (should prefer captures and rosettes over neutral moves).
- Full game simulation: play 1000 random games to completion and verify all end in a valid win state with no panics.

## Reference Material

- RoyalUr-Java (github.com/RoyalUr/RoyalUr-Java): API design reference, especially the Game class, Move class, and ruleset configuration. MIT license.
- RoyalUr-Python (github.com/RoyalUr/RoyalUr-Python): Same API in Python, useful for cross-referencing behavior.
- RoyalUr.net rules page (royalur.net/rules): Canonical description of the Finkel ruleset with diagrams.
- Irving Finkel's reconstruction: Based on a cuneiform tablet (Rm-III.6.b) from 177 BCE, translated by Finkel at the British Museum.
