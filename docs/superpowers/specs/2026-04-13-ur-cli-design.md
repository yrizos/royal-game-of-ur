# ur-cli Design Spec

**Date:** 2026-04-13
**Status:** Approved

## Overview

`ur-cli` is a terminal frontend for the Royal Game of Ur built with ratatui. It is a peer crate to `ur-core` in the workspace — independently publishable. It renders the board, handles input, runs animations, and delegates all game logic to `ur-core`. It never computes legal moves, captures, or win conditions itself.

## Crate Structure

```
royal-game-of-ur/
  ur-core/   # library crate (existing)
  ur-cli/    # binary crate (new)
    Cargo.toml
    src/
      main.rs         # entry point, terminal setup/teardown
      app.rs          # top-level App state and event loop
      ui/
        mod.rs
        title.rs      # title screen
        menu.rs       # main menu, difficulty selection
        game.rs       # gameplay screen
        gameover.rs   # game over summary
      animation.rs    # dice roll, piece movement, capture flash
      input.rs        # key event mapping
```

## Dependencies

```toml
[dependencies]
ur-core = { path = "../ur-core" }
ratatui = "0.26"
crossterm = "0.27"
rand = "0.8"
```

## Game Flow

```
Title Screen
  └─ New Game → Difficulty Selection → First Player Roll (dice-off) → Gameplay → Game Over
  └─ Quit
```

Each screen is a distinct UI state. `App` holds a `Screen` enum and routes input and rendering accordingly.

## Gameplay Layout

Players flank the board on either side (Layout C). The board is centered.

```
┌──────────────┬────────────────────────────────┬──────────────┐
│  Player 1    │   ┌────┬────┬────┬────┐         │  Player 2    │
│  (You)       │   │ ✦  │    │    │    │  ┌────┬──│  (Hard AI)  │
│              │ ┌─┼────┼────┼────┼────┼──┼────┼─┐│              │
│  ○○○○○○○     │ │ │    │    │    │ ✦  │  │    │ ││  ○○○●●○○    │
│  Scored: ●●  │ └─┼────┼────┼────┼────┼──┼────┼─┘│  Scored: ●● │
│              │   │ ✦  │    │    │    │  └────┴──│              │
│  ← YOUR TURN │   └────┴────┴────┴────┘         │              │
├──────────────┴────────────────────────────────┴──────────────┤
│  Dice: ●●○○ = 2   Moves: 14   Time: 3:22   Space to roll      │
│  Last: Player 1 captured Player 2 on (1,4)       [L] log      │
└───────────────────────────────────────────────────────────────┘
```

**Always-visible elements during gameplay:** board, unplayed/scored pieces per player, turn indicator, dice result, move counter, game timer, last log message, key hint.

## Board Rendering

- **Tile size:** 4 chars wide × 1 row tall
- **Board shape:** 3×8 grid, cols 4–5 absent in rows 0 and 2 (correct H-shape)
- **Piece symbol:** `●` for both players
- **Player 1 (human):** bright blue foreground
- **Player 2 (AI):** bright red foreground
- **Empty rosette:** gold `✦` centered in tile
- **Occupied rosette:** piece symbol in player color on warm amber background — rosette status always visible regardless of occupancy
- **Selected piece:** black on gold background
- **Legal move targets:** dimmed highlight to indicate reachable squares

## Title Screen

Three-part vertical layout, centered:
1. Small text: `The Royal Game of`
2. Block-letter `UR` in gold (figlet-style, exact font chosen during implementation)
3. Actual cuneiform glyphs (𒆳𒆳 𒀭𒂗𒍪 𒆳𒆳) as historical decoration
4. Subtitle: `circa 2600 BCE · Mesopotamia`
5. Menu: `[ New Game ]` / `[ Quit ]`

Cuneiform border (𒀭) top and bottom.

## Menus

**Main Menu:** New Game, Quit. Navigated with arrow keys, confirmed with Enter.

**Difficulty Selection:** Easy (depth 2), Medium (depth 4), Hard (depth 6). Brief description of each. Arrow keys + Enter.

**First Player Determination:** Dramatic dice-off. Both players roll simultaneously (animated), higher roll goes first. Tie = re-roll. Result announced before gameplay begins.

## Key Bindings

| Key | Context | Action |
|-----|---------|--------|
| Space | Gameplay | Roll dice |
| ↑ ↓ ← → | Gameplay | Navigate between selectable pieces |
| Enter | Gameplay / Menus | Confirm selection |
| Escape | Gameplay | Quit confirmation prompt |
| Escape | Menus | Back |
| L | Gameplay | Toggle full game log panel |
| N | Game Over | New game |
| Q | Game Over | Quit |

## Animations

All animations are non-blocking — they run frame-by-frame inside the event loop.

- **Dice roll:** rapid symbol cycling (`○●○●...`) settling on the final value. Duration ~600ms.
- **Piece movement:** piece moves square-by-square along its path. Duration ~80ms per square.
- **Capture flash:** captured piece flashes before disappearing. Duration ~300ms.
- **AI thinking:** spinner shown while AI computes. Computation runs on a separate thread to keep the UI responsive.

During animations, input is suppressed except Escape (which cancels and applies the move immediately).

## Game Log

One-line summary always visible in the status bar. Full log accessible by pressing L — expands as a scrollable overlay panel. Log entries: moves, captures, rosette landings, dice rolls, turn changes.

## Game Over Screen

Displayed when a player bears off all 7 pieces. Shows:
- Winner announcement
- Final move count
- Game duration
- Captures per player
- Prompt: `[N] New Game  [Q] Quit`

## Error Handling

Terminal resize below minimum dimensions (80×24) shows a "please resize your terminal" message and suspends rendering until the terminal is large enough. No other runtime errors are expected — all game logic is validated by `ur-core`.

## Out of Scope

- Multiplayer (human vs human)
- Replay / undo
- Sound
- Configuration file
- Any game logic — `ur-core` owns all of that
