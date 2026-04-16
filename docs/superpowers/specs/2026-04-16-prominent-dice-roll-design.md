# Prominent Dice Roll — Design Spec

**Date:** 2026-04-16  
**Status:** Approved

## Problem

The dice roll is the central event of every turn, but it is currently rendered as a small grey text fragment in the single-line status bar (`Dice: ●●●○ = 3  Moves: 12  Time: 01:23 …`). It competes visually with move count, timer, AI spinner, log snippets, and key hints. Players frequently miss the result.

## Goal

Make the dice roll impossible to miss. Both players' rolls must be shown prominently in their respective side panels. Rolling must be fully automatic — the player never presses a key to roll.

---

## Design Decisions (from brainstorm)

| Decision | Choice |
|---|---|
| Where the dice live | Inside the active player's side panel |
| Dice visual style | Four tetrahedral dice shown as ▲ (scored) / △ (not scored) |
| AI panel | AI's roll also shown in the AI panel for consistency |
| Roll = 0 or no legal moves | Red alarming style + auto-advance after 1 s |
| User-initiated roll | Removed entirely — rolling is always automatic |

---

## Panel States

The player panel renders one of five states based on `App` fields. All five apply equally to both the human (Player 1) and AI (Player 2) panels, just in their respective colours.

### State 1 — Pending (auto-roll imminent)

Shown for ~0.3 s after the opponent's turn ends, before the dice animation fires.

```
│ ▶ YOUR TURN          │
│                      │
│   rolling...         │
```

### State 2 — Dice animating

The existing `Animation::DiceRoll` is playing. Dice symbols cycle rapidly; the sum shows `= ?`.

```
│   ▲  △  ▲  △        │
│   = ?                │
```

### State 3 — Landed, moves available

Dice freeze on the final value. Prompt appears in green.

```
│   ▲  ▲  ▲  △        │   (roll = 3, Player 1 colour)
│   = 3                │
│   pick a move        │   (green)
```

When it is the opponent's turn, the panel shows the last roll dimmed with a `(last roll)` label — so the human can see what the AI rolled. This requires a separate `last_opponent_roll: Option<Dice>` field (see Data Model), because `dice_roll` is cleared by `apply_move` before the human's turn begins.

```
│   ▲  ▲  △  △        │   (roll = 2, Player 2 colour, dimmed)
│   = 2 (last roll)    │
```

### State 4 — No legal moves (forfeit)

Triggered when the roll is 0 **or** when the roll is non-zero but no legal moves exist. Everything is rendered in red. A `passing turn...` label appears. After 1 second the turn is automatically forfeited and the next player's auto-roll fires.

```
│   △  △  △  △        │   (red, roll = 0)
│   = 0  no moves      │   (red)
│   passing turn...    │   (dim grey)
```

```
│   ▲  ▲  △  △        │   (red, roll = 2 but blocked)
│   = 2  no moves      │   (red)
│   passing turn...    │   (dim grey)
```

### State 5 — Rosette extra turn

After landing on a rosette the same player rolls again immediately (no 0.3 s delay — the re-roll fires as soon as the piece-move animation completes).

```
│   ✦ rosette bonus!   │   (gold)
│   rolling again...   │   (grey)
```

---

## Behaviour Changes

### Auto-roll (replaces Space = Roll)

- When it becomes a player's turn (after `apply_move`, `forfeit_turn`, or game start), the app sets a flag `pending_roll: bool = true`.
- On the next `tick()` call, if `pending_roll` is true and no animation is running, the dice animation is started automatically. A 0.3 s delay is added via a `roll_after: Option<Instant>` field so the UI does not feel instantaneous.
- Exception: after a rosette the delay is skipped (re-roll fires immediately after the piece-move animation finishes).
- The `Space` key binding (`Action::RollDice`) is removed from `input.rs` and `handle_roll_dice` becomes an internal method not reachable by the user.

### Forfeit delay

- When `on_animation_done()` finds no legal moves, instead of forfeiting immediately it:
  1. Sets a new `forfeit_after: Option<Instant>` field to `Instant::now() + 1 s`.
  2. Leaves `dice_roll` and the panel in the alarming (red) state.
- On each `tick()`, if `forfeit_after` is set and the deadline has passed, `forfeit_turn()` is called and `pending_roll` is set for the next player.

### Status bar

- The dice fragment (`Dice: ●●●○ = 3`) is removed from `render_status_bar`.
- The `Space=Roll` key hint is removed from the status bar.
- Move count, timer, and remaining key hints (`↑↓=Select  Enter=Move  Esc=Pause  L=log`) are unchanged.

### AI panel during AI turn

- `start_ai_turn()` continues to roll immediately (as it does today — no 0.3 s delay for the AI's own roll, since the AI never "waits").
- The AI's roll is stored in `App::dice_roll` as today. The AI panel reads `dice_roll` and renders it in the AI's colour (red) while the AI is thinking or after it has played.

---

## Data Model Changes (`App`)

| Field | Change |
|---|---|
| `pending_roll: bool` | **New.** Set true when a player's turn begins. Cleared when `Animation::DiceRoll` starts. |
| `roll_after: Option<Instant>` | **New.** When set, the auto-roll fires at this time. Cleared on fire. The delay is `AUTO_ROLL_DELAY_MS = 300`. |
| `forfeit_after: Option<Instant>` | **New.** When set, the forfeit fires at this time. Cleared on fire. The delay is `FORFEIT_DISPLAY_MS = 1000`. |
| `rosette_reroll: bool` | **New.** Set by `apply_move` when the result is a rosette extra turn; causes the next auto-roll to skip the 0.3 s delay. |
| `last_opponent_roll: Option<Dice>` | **New.** Stores the opponent's last roll so the panel can show it dimmed after `dice_roll` has been cleared by `apply_move`. Set in `apply_move` before clearing `dice_roll`. |
| `dice_roll` | Unchanged. Now also read by the panel renderer. |

---

## Files Changed

| File | Change |
|---|---|
| `ur-cli/src/app.rs` | Add 4 new fields; add `tick_auto_roll()` and `tick_forfeit_delay()` helpers called from `tick()`; remove user-facing `handle_roll_dice` trigger; set `pending_roll` in `apply_move`, `begin_game`, `on_animation_done` (forfeit branch). |
| `ur-cli/src/animation.rs` | `tick()` calls the two new helpers after advancing animations. |
| `ur-cli/src/input.rs` | Remove `RollDice` action and the `Space` binding in `Screen::Game`. |
| `ur-cli/src/ui/game.rs` | `render_player_panel` gains a `panel_dice: PanelDice` parameter (a small enum: `Hidden`, `Pending`, `Animating(Dice)`, `Result(Dice)`, `NoMoves(Dice)`, `RosettePending`) and renders the dice widget accordingly. `render_status_bar` drops the dice fragment and `Space=Roll` hint. The `PanelDice` value is computed in `render_game` from `App` fields before calling `render_player_panel`. |

---

## Out of Scope

- Sound / audio cues on roll
- Configurable auto-roll delay
- Roll history panel
