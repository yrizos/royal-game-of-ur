---
name: ur-core scaffold
description: TDD skeleton for the ur-core library crate and project workspace
type: project
date: 2026-04-13
---

# Scaffold Design: Royal Game of Ur

## Scope

Bootstrap the Cargo workspace and `ur-core` library crate as a TDD skeleton. `ur-desktop` is intentionally excluded and will be added later. The goal is a compilable workspace with all public types declared, all functions stubbed with `todo!()`, and all spec-required test cases present (failing, empty bodies).

## Workspace Structure

```
royal-game-of-ur/
  Cargo.toml              # workspace, members = ["ur-core"]
  .gitignore              # Rust, macOS, VSCode
  .gitattributes          # *.rs text eol=lf, binary assets -text
  .editorconfig           # indent_style=space, indent_size=4, trim_trailing_whitespace
  rust-toolchain.toml     # channel = "stable", pinned
  rustfmt.toml            # max_width=100, grouped imports, edition=2021
  Justfile                # test, check, fmt recipes
  CLAUDE.md
  SPEC.md
  docs/
    superpowers/
      specs/              # this file
  ur-core/
    Cargo.toml            # [lib], deps: rand, serde (feature flag)
    src/
      lib.rs
      board.rs
      player.rs
      dice.rs
      state.rs
      ai.rs
    tests/
      simulation.rs
```

## Module Responsibilities

### `board.rs`
- `Square` — `(row: u8, col: u8)` newtype/struct
- `BoardShape` — defines which squares exist; default is the 20-square Finkel layout
- `Path` — ordered `Vec<Square>` for one player's route, includes logical entry/exit markers

### `player.rs`
- `Player` — enum `{ Player1, Player2 }`, with `opponent()` helper
- `Piece` — struct `{ player: Player, index: u8 }` (indices 0–6)

### `dice.rs`
- `Dice` — newtype over `u8` (values 0–4)
- `Dice::roll(rng)` — uses caller-provided `impl Rng`
- Probability constants for expectiminimax

### `state.rs`
- `GameRules` — bundles board shape, paths, dice type, piece count, rule flags
- `GamePhase` — enum `{ WaitingForRoll, WaitingForMove(Dice), GameOver(Player) }`
- `GameState` — complete immutable snapshot; derives `Clone, Debug, PartialEq, Eq, Hash`
- `Move` — piece + origin + destination
- `MoveResult` — new state + metadata (capture, rosette, scored, game over)
- Core functions: `GameState::new`, `legal_moves`, `apply_move`, `is_finished`, `winner`, `current_player`

### `ai.rs`
- `choose_move(state, roll, depth) -> Move`
- Expectiminimax search with chance nodes weighted by dice probability
- Board evaluation heuristic: advancement, scored pieces, captures, rosette occupancy, shared-row vulnerability

## Test Coverage (TDD Stubs)

### Inline tests per module

**`board.rs`**
- `test_path_player1_covers_correct_squares`
- `test_path_player2_covers_correct_squares`
- `test_shared_row_is_columns_0_through_7`
- `test_rosette_positions`
- `test_invalid_squares_row0_cols_4_5`
- `test_invalid_squares_row2_cols_4_5`

**`dice.rs`**
- `test_dice_range_0_to_4`
- `test_dice_probability_distribution`

**`state.rs` — legal move generation**
- `test_no_legal_moves_roll_0`
- `test_no_legal_moves_all_blocked`
- `test_can_enter_piece_from_pool`
- `test_cannot_enter_when_entry_occupied_by_friendly`
- `test_bearing_off_requires_exact_roll`
- `test_overshoot_not_allowed`
- `test_capture_on_shared_row`
- `test_capture_blocked_by_rosette`
- `test_friendly_square_blocked`
- `test_rosette_safe_from_capture`

**`state.rs` — move application**
- `test_piece_advances_correct_squares`
- `test_capture_returns_piece_to_pool`
- `test_rosette_grants_extra_turn`
- `test_scoring_a_piece`
- `test_win_detection_all_7_scored`
- `test_apply_illegal_move_panics`

**`ai.rs`**
- `test_ai_prefers_capture_over_neutral`
- `test_ai_prefers_rosette_over_neutral`
- `test_ai_depth_1_returns_valid_move`

### Integration tests (`tests/simulation.rs`)
- `test_1000_random_games_all_complete_with_valid_winner`

## Dependencies

```toml
# ur-core/Cargo.toml
[dependencies]
rand = "0.8"

[features]
default = []
serde = ["dep:serde"]

[dependencies.serde]
version = "1"
features = ["derive"]
optional = true
```

## Justfile Recipes

```just
test:
    cargo test --workspace

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features

fmt:
    cargo fmt

build:
    cargo build --workspace
```

## Design Decisions

- **Flat module layout** chosen over domain folders — appropriate for this crate size, easy to navigate, tests inline with source.
- **Caller-provided RNG** — keeps `ur-core` deterministic and testable; no hidden global state.
- **`todo!()` stubs** — all functions compile but panic at runtime; TDD workflow fills them in test-by-test.
- **`ur-desktop` excluded** — added later when frontend work begins; workspace `members` list updated at that point.
