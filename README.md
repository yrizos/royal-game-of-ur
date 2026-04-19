# The Royal Game of Ur

One of the oldest known board games, dating to about 2600 BCE in ancient Mesopotamia. Sets were excavated from the Royal Tombs of Ur (modern-day Iraq) by Sir Leonard Woolley in the 1920s.

These rules follow Irving Finkel's reconstruction at the British Museum, based on a cuneiform tablet from around 177 BCE.

Play against an AI opponent in your terminal.

```sh
just run
```

Requires Rust 1.70+ and a terminal at least 80x24.

## How to Play

Race all 7 pieces through a 14-step path and off the board before the AI does the same. SPACE roll, ↑↓←→ select piece, ENTER move.

```
Space        Roll the dice
↑↓ / ←→      Select which piece to move
Enter        Confirm the move
H            Help
L            Toggle game log
Esc          Pause menu
```

Steps 1-4 are your private lane (left column). Steps 5-12 are the bridge (both players!). Steps 13-14 are your exit lane (left column). The AI's path mirrors yours on the right column.

✦ = Rosette: extra turn + safe from capture. Must roll exactly to exit.

```
       YOU      ◆      AI
    ┌───────┬───────┬───────┐
    │  13   │  12   │       │
    ├───────┼───────┼───────┤
    │  14✦  │  11   │       │
    └───────┼───────┼───────┘
            │  10   │
            ├───────┤
            │   9   │
    ┌───────┼───────┼───────┐
    │   1   │   8✦  │       │
    ├───────┼───────┼───────┤
    │   2   │   7   │       │
    ├───────┼───────┼───────┤
    │   3   │   6   │       │
    ├───────┼───────┼───────┤
    │   4✦  │   5   │       │
    └───────┴───────┴───────┘
```

4 binary dice, total 0 to 4. Roll 0 = no move.

| Roll | 0   | 1   | 2   | 3   | 4   |
| ---- | --- | --- | --- | --- | --- |
| Prob | 6%  | 25% | 38% | 25% | 6%  |

**Capturing.** Land on an opponent's piece on the bridge ◆ to send it back to their pool. Pieces on a rosette ✦ are safe.

**Bearing off.** Roll exactly to exit. First to score all 7 wins.

**AI opponent.** Expectiminimax search, weighted by dice probability.

| Difficulty | Search depth |
| ---------- | ------------ |
| Easy       | 2            |
| Medium     | 4            |
| Hard       | 6            |

## Project Layout

```
.
├── ur-cli/                    # ratatui terminal frontend
│   └── src/
│       ├── ui/
│       │   ├── game.rs        # board widget, player panels, dice, status bar
│       │   ├── gameover.rs    # game over screen
│       │   ├── menu.rs        # difficulty select
│       │   ├── mod.rs         # top-level render dispatch
│       │   ├── pause.rs       # pause menu + scrollable help/rules overlay
│       │   ├── styled_box.rs  # reusable bordered box widget
│       │   └── title.rs       # title screen
│       ├── animation.rs       # dice roll, piece movement, capture flash
│       ├── app.rs             # app state, screens, game flow, 2D cursor
│       ├── input.rs           # key mapping per screen
│       └── main.rs            # entry point, terminal setup, event loop
├── ur-core/                   # pure game logic library, no I/O
│   ├── src/
│   │   ├── ai.rs              # expectiminimax search + board evaluation
│   │   ├── board.rs           # board geometry, paths, rosette positions
│   │   ├── dice.rs            # dice type and probability distribution
│   │   ├── lib.rs             # crate root, re-exports
│   │   ├── player.rs          # player enum
│   │   └── state.rs           # game state, legal moves, move application
│   └── tests/
│       └── simulation.rs      # 1000-game randomized simulation
├── docs/superpowers/
│   ├── specs/                 # design specs (write before implementing)
│   └── plans/                 # implementation plans (write before coding)
├── Cargo.toml                 # workspace root
├── Justfile                   # task runner
└── SPEC.md                    # full Finkel ruleset specification
```

All game rules live in `ur-core`, a pure logic library with no I/O. `ur-cli` is a thin terminal frontend that delegates every rule decision to `ur-core`.

## Automation

```sh
just run       # play the game
just test      # run all tests
just check     # lint and format check
just fmt       # autoformat
just build     # build all crates
just clean     # remove build artifacts
```
