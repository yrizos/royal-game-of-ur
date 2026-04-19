# Royal Game of Ur

A Rust workspace containing a reusable game logic library and a terminal application for the Royal Game of Ur, one of the oldest known board games (circa 2600 BCE, Mesopotamia).

## Project Structure

```
royal-game-of-ur/
  ur-core/    # Library crate: pure game logic, no I/O
  ur-cli/     # Binary crate: ratatui terminal frontend, depends on ur-core
```

## Architecture

`ur-core` is the single source of truth for all game rules. It contains board geometry, piece paths, legal move generation, move application, win detection, and the AI opponent. It has no dependencies on rendering, input, audio, or any platform API. It must compile to any Rust target including WebAssembly.

`ur-cli` is a thin terminal frontend built with ratatui and crossterm. It renders the board, handles keyboard input, runs animations (dice rolls, piece movement, captures), and calls `ur-core` for every game logic decision. It never computes whether a move is legal, never checks for captures, and never decides who wins. It asks `ur-core`.

## Ruleset

The default ruleset is the Finkel reconstruction. See SPEC.md for the complete rules including board geometry, piece paths, rosette behavior, dice mechanics, and capture rules.

Key facts for quick reference:

- Board: 20 squares on a 3x8 grid with 4 squares removed (row 0 and row 2 each lack columns 4 and 5)
- Pieces: 7 per player
- Dice: 4 binary dice, result 0 to 4
- Rosettes at: (0,0), (0,6), (1,3), (2,0), (2,6)
- Rosettes grant extra turn and are safe from capture
- Middle row (row 1) is shared between players
- Rolling 0 forfeits the turn
- Must land exactly on exit to bear off
- First player to bear off all 7 pieces wins

## Development Conventions

### Code Style

- Run `cargo fmt` before every commit.
- Run `cargo clippy --all-targets --all-features` and resolve all warnings.
- Write doc comments (`///`) on all public types and functions in `ur-core`.
- Keep functions short. If a function exceeds 40 lines, consider splitting it.

### Testing

- Write tests before implementation (TDD).
- Every rule edge case in SPEC.md must have a corresponding test.
- Test names should describe the scenario: `test_capture_blocked_by_rosette`, not `test_move_3`.
- Run `cargo test --workspace` to verify everything passes.
- Include a randomized full-game simulation test that plays 1000 games to completion.

### Commits

- Each commit should represent one logical change.
- Commit messages follow conventional commits: `feat:`, `fix:`, `test:`, `refactor:`, `docs:`.
- Do not commit code that fails `cargo test` or `cargo clippy`.

### Dependencies

- `ur-core` should have minimal dependencies. `rand` is acceptable for dice rolling. `serde` is acceptable behind a feature flag for serialization.
- `ur-cli` depends on `ratatui`, `crossterm`, and `rand` for terminal rendering, input, and dice rolling.
- Do not add dependencies without justification.

## Build and Run

```bash
# Run tests
cargo test --workspace

# Run the terminal app
cargo run -p ur-cli

# Check formatting and lints
cargo fmt --check
cargo clippy --all-targets --all-features
```

## AI Opponent

The AI uses expectiminimax search. This is minimax adapted for games with chance nodes (dice rolls). At each decision node, the AI evaluates all legal moves. At each chance node, it weights outcomes by dice probability.

The search depth acts as a difficulty setting. Depth 2 is casual, depth 4 is competent, depth 6 is strong. The board evaluation heuristic should consider: piece advancement along the path, number of scored pieces, number of captures available, rosette occupancy, and vulnerability to capture on the shared row.

## Future Plans (Do Not Implement Yet)

- Wasm wrapper for browser-based frontends
- C FFI wrapper for non-Rust consumers
- Additional rulesets (Masters, Aseb)
- Online multiplayer
- Mobile frontends

These are mentioned only to explain why `ur-core` is designed with strict separation from I/O. The current scope is `ur-core` and `ur-cli` only.

## graphify — USE THIS WHEN BUILDING CODE

A knowledge graph of this entire codebase lives in `graphify-out/`. It maps every module, type, function, spec, and design decision — and the relationships between them.

### How to use it when writing code

**Before implementing anything, query the graph to understand what you're touching:**

1. **Adding a function or type?** Run `/graphify query "where does X fit"` to find which community it belongs to, what it should depend on, and what already exists nearby. Don't duplicate something that's already there.
2. **Changing a module?** Run `/graphify explain "ModuleName"` to see everything connected to it — callers, callees, types it shares, tests that cover it. Change with full awareness of impact.
3. **Deciding where code goes?** The graph communities are the real module boundaries. `App State Machine` (community 0), `Game State & Rules` (community 1), `Animation System` (community 8), etc. Put new code where the graph says it connects, not where it seems like it "should" go.
4. **Fixing a bug?** Run `/graphify query "how does X connect to Y"` to trace the path between the symptom and potential causes across module boundaries.

**Read `graphify-out/GRAPH_REPORT.md` before:**
- Planning a refactor or deciding where new code should go
- Investigating a bug that might span modules
- Answering "where is X used" or "what depends on Y"

### Keeping the graph current

- Code-only changes are picked up automatically by the git post-commit hook.
- Run `/graphify . --update` after adding or changing non-code files (docs, specs, plans).
- If `graphify-out/.needs_update` exists, run the update before proceeding.

### Install the git hook (if missing)

```bash
graphify hook install
```
