# UI Revamp Design — Royal Game of Ur CLI

Date: 2026-04-17

## Scope

Revamp the terminal UI layout and component consistency. No changes to game logic, AI, rules, or any `ur-core` code. No new screens. No new game features. Existing behaviour is preserved exactly; only rendering and input handling change.

## Problems Being Fixed

1. **Layout shifts** — player panel content (dice, event messages) appears/disappears, pushing other elements around. Nothing has a fixed address.
2. **Dice disappear** — `PanelDice::Hidden` renders nothing, collapsing the dice area when inactive.
3. **Inconsistent boxes** — three different box-drawing mechanisms across the codebase: log modal (no `inner()`, no padding), help screen (`block.inner()` + hardcoded `"  "` string prefix), player panels (ad-hoc).
4. **Log labels** — log entries use `P1`/`P2` instead of player-colored `You`/`AI`. Not all events are captured.
5. **Turn summary buried** — the `event_msg` (capture/rosette/score) is a transient dimmed string at the bottom of the panel. It is not prominent and disappears.
6. **Space/Enter** — Space rolls dice but cannot confirm a move. Both Space and Enter should confirm a move when one is selected.

## Layout

Portrait board (3 columns × 8 rows, H-shaped). Three-column layout: YOU panel | Board | AI panel. Status bar at bottom.

```
┌── YOU ──────────────────────┐  YOU  ◆  AI  ┌── AI ───────────────────────┐
│                             │ ┌───┬───┬───┐ │                             │
│  ▶ YOUR TURN                │ │ ✦ │   │ ✦ │ │                             │
│                             │ ├───┼───┼───┤ │  ┌───┐ ┌───┐ ┌───┐ ┌───┐  │
│  ┌───┐ ┌───┐ ┌───┐ ┌───┐  │ │ ● │   │   │ │  │ ● │ │   │ │   │ │   │  │
│  │ ● │ │ ● │ │ ● │ │   │  │ ├───┼───┼───┤ │  └───┘ └───┘ └───┘ └───┘  │
│  └───┘ └───┘ └───┘ └───┘  │ │   │   │   │ │  Last: 1                    │
│  Roll: 3                   │ ├───┼───┼───┤ │                             │
│  ◆ captured AI at step 10  │ │   │ ✦ │   │ │  moved to step 7            │
│                            │ └───┼───┼───┘ │                             │
│  ·  ·  ·  ·  ·  ·  ·  ·   │     │   │     │  ·  ·  ·  ·  ·  ·  ·  ·   │
│                            │     ├───┤     │                             │
│  Scored  ● ● ●             │     │   │     │  Scored  ● ●               │
│  Pool    ● ● ● ●           │ ┌───┼───┼───┐ │  Pool    ● ● ● ● ●         │
│  Captures: 2               │ │ ✦ │   │ ✦ │ │  Captures: 1               │
│                            │ ├───┼───┼───┤ │                             │
└────────────────────────────┘ │   │   │   │ └─────────────────────────────┘
                               └───┴───┴───┘
[ Moves: 24  Time: 03:45   ↑↓ select   Space/Enter move   Esc   [L] log ]
```

### Player panel section order (top to bottom, all sections fixed-height)

1. **Turn indicator** — `▶ YOUR TURN` (active, player color, bold) or empty line (inactive). Always 1 line reserved.
2. **Dice** — 4 binary dice boxes, always rendered. Active: full color. Inactive: dimmed with "Last: N". Never hidden, never collapse. Always 4 lines reserved (3 box rows + 1 label row).
3. **Turn summary** — immediately below dice. What the roll caused: capture, rosette, score, no moves, or blank. Always exactly 2 lines reserved.
4. **Divider** — `·  ·  ·  ·  ·  ·  ·` separator line.
5. **Stats** — Scored (colored dots), Pool (colored dots), Captures. Always 3 lines reserved.

All sections are always reserved at fixed height. Content changes; size never does.

### Board area

- Board widget unchanged (portrait, H-shape, existing `BoardWidget`).
- Column headers (YOU / ◆ / AI) rendered above board as today.
- Board vertically centered in available height.

### Status bar

Single line at bottom:
```
Moves: N  Time: MM:SS   ↑↓ select   Space/Enter move   Esc   [L] log
```
AI thinking spinner stays in the status bar when AI is computing.

## Unified Box Component

A single `StyledBox` struct replaces all three current box-drawing approaches.

```rust
pub struct StyledBox<'a> {
    pub title: &'a str,
    pub border_color: Color,
    pub scrollable: bool,
}
```

Behaviour:
- Always calls `block.inner(area)` to get the content rect.
- Always applies 1-char inner padding on all sides (via `Rect` shrink, not string prefixes).
- When `scrollable: true` and content overflows, renders a scroll indicator in `title_bottom`.
- Returns the padded inner `Rect` to the caller for content rendering.

Used by:
| Consumer | border_color | scrollable |
|---|---|---|
| YOU player panel | `COLOR_P1` when active, `Color::DarkGray` when inactive | false |
| AI player panel | `COLOR_P2` when active, `Color::DarkGray` when inactive | false |
| Log modal (L) | `Color::Yellow` | true |
| Help modal | `Color::Yellow` | true |

No other widget draws its own box.

## Log Modal Fixes

- Labels: `You` (COLOR_P1) and `AI` (COLOR_P2) instead of `P1`/`P2`.
- All events captured: every move (including ordinary moves with no special outcome), every AI roll, no-moves forfeits.
- Rendered via `StyledBox` with `scrollable: true`. Identical mechanism to help modal.

## Input Change

Space confirms a move when a legal move is selected at the cursor (same as Enter). Space rolls dice only when `dice_roll.is_none()` and `!pending_roll` (i.e. no roll is in progress or pending). This matches the existing Enter behaviour exactly — just extends it to Space.

## What Does NOT Change

- `ur-core` — zero changes.
- `BoardWidget` rendering logic — zero changes.
- All screen transitions (`Title`, `DifficultySelect`, `DiceOff`, `PauseMenu`, `Help`, `GameOver`).
- `PanelDice` state machine logic.
- AI difficulty, search, or timing.
- Animation system.
- All existing key bindings except Space gains dual role.
