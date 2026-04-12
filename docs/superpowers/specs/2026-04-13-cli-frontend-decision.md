---
name: CLI frontend decision
description: Decision to use ratatui terminal frontend (ur-cli) instead of macroquad desktop app (ur-desktop)
type: project
date: 2026-04-13
---

# Frontend Decision: CLI with Ratatui

The first frontend is a terminal application (`ur-cli`) built with ratatui. A desktop frontend (`ur-desktop` with macroquad) may or may not be added later.

**Why:** The CLI frontend validates the `ur-core` API with minimal overhead. Ratatui provides rich terminal rendering with full Unicode support, colored cells, box-drawing grids, and layout management — sufficient to build a polished, playable game without leaving the terminal.

## Project Structure

```
royal-game-of-ur/
  ur-core/    # Library crate: pure game logic, no I/O
  ur-cli/     # Binary crate: ratatui terminal frontend
```

## Dependencies for ur-cli

- `ur-core` (workspace dependency)
- `ratatui` (terminal UI framework)
- `crossterm` (terminal backend for ratatui)
- `rand` (dice rolling)

## Game Flow

Title Screen → Main Menu (New Game / Quit) → Difficulty Selection (Easy / Medium / Hard) → First Player Roll (dramatic dice-off) → Gameplay → Game Over Summary

## Gameplay Loop

1. Turn indicator shows whose turn it is
2. Current player rolls dice (Space for human; auto for bot with spinner)
3. Legal moves highlighted; human navigates with arrow keys, confirms with Enter
4. Move animates square-by-square; captures flash before disappearing
5. Rosette landing announced in game log; same player rolls again
6. Bot turn shows AI thinking indicator then animates the same way

## Key Bindings

- **Space:** Roll dice
- **Arrow keys:** Navigate legal pieces
- **Enter:** Confirm piece selection
- **Escape:** Quit confirmation (gameplay) / back (menus)
- **L:** Toggle full game log
- **N:** New game (from game over screen)
- **Q:** Quit

## Board Rendering

Multi-cell tiles. Pieces shown as `●` centered in tile. Player 1 and Player 2 distinguished by foreground color. Rosettes distinguished by background color. Exact colors, tile dimensions, and border styles determined during development.

## Layout (always visible during gameplay)

Board, unplayed/scored pieces per player, current dice roll, turn indicator, capture counter, move counter, game timer, last game log message, contextual keyboard shortcuts.

## Features

Single-player vs AI, difficulty selection (Easy/Medium/Hard), animated dice rolls, piece movement animation (square-by-square), capture flash, game log with full history (L to expand), title screen (ASCII art, Mesopotamian/cuneiform style), first player dramatic dice-off, game over summary (winner, moves, time, captures per player), retro aesthetic (no sound).

## AI Difficulty Mapping

- Easy → search depth 2
- Medium → search depth 4
- Hard → search depth 6
