# Prominent Dice Roll — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the dice roll display from the hidden status bar into both player panels, and make rolling fully automatic (no Space key required).

**Architecture:** Nine focused tasks proceeding from input → data model → rendering → behaviour. Each task compiles and passes tests independently. New `PanelDice` enum drives the panel renderer; two new `App` helpers (`tick_auto_roll`, `tick_forfeit_delay`) called from the animation tick loop handle all timing.

**Tech Stack:** Rust, ratatui, crossterm, `std::time::Instant`.

---

## File Map

| File | What changes |
|---|---|
| `ur-cli/src/input.rs` | Remove `RollDice` action + `Space` binding |
| `ur-cli/src/app.rs` | 5 new fields, 2 constants, 2 new helpers, updated `begin_game` / `apply_move` / `on_animation_done` / `start_ai_turn` |
| `ur-cli/src/animation.rs` | `tick()` calls the two new helpers |
| `ur-cli/src/ui/game.rs` | `PanelDice` enum, `dice_pips_line` helper, updated `render_player_panel`, `render_status_bar`, and `render_game` |

---

### Task 1: Remove `RollDice` action and `Space=Roll` binding

**Files:**
- Modify: `ur-cli/src/input.rs`

- [ ] **Step 1: Write the failing test**

Add to `ur-cli/src/input.rs` inside the existing `#[cfg(test)]` block:

```rust
#[test]
fn test_space_in_game_returns_none_after_roll_removed() {
    let action = map_key(key(KeyCode::Char(' ')), &crate::app::Screen::Game);
    assert_eq!(action, None);
}
```

- [ ] **Step 2: Run the test — expect FAIL**

```bash
cargo test -p ur-cli input::tests::test_space_in_game_returns_none_after_roll_removed
```

Expected: FAIL — `assert_eq!(Some(Action::RollDice), None)`.

- [ ] **Step 3: Remove `RollDice` from the `Action` enum**

In `ur-cli/src/input.rs`, remove the variant:
```rust
// DELETE this line:
RollDice,
```

- [ ] **Step 4: Remove the `Space` binding from `Screen::Game`**

Replace the `Screen::Game` arm:

```rust
Screen::Game => match key.code {
    KeyCode::Up | KeyCode::Char('k') | KeyCode::Left | KeyCode::Char('h') => {
        Some(Action::SelectPrev)
    }
    KeyCode::Down | KeyCode::Char('j') | KeyCode::Right => Some(Action::SelectNext),
    KeyCode::Enter => Some(Action::ConfirmMove),
    KeyCode::Char('l') => Some(Action::ToggleLog),
    KeyCode::Esc => Some(Action::QuitPrompt),
    _ => None,
},
```

- [ ] **Step 5: Delete the old `test_space_maps_to_roll_dice_in_game` test**

Remove:
```rust
#[test]
fn test_space_maps_to_roll_dice_in_game() {
    let action = map_key(key(KeyCode::Char(' ')), &crate::app::Screen::Game);
    assert_eq!(action, Some(Action::RollDice));
}
```

- [ ] **Step 6: Fix the main event loop**

In `ur-cli/src/main.rs`, remove line 95 from `handle_action`:

```rust
// DELETE this arm:
Action::RollDice => app.handle_roll_dice(),
```

`handle_roll_dice` remains in `app.rs` (its unit tests still reference it). It is now an internal method only — no longer reachable from the event loop. If `cargo clippy` warns about it, add `#[allow(dead_code)]` above the method signature in `app.rs`.

- [ ] **Step 7: Run all tests — expect PASS**

```bash
cargo test -p ur-cli
```

- [ ] **Step 8: Commit**

```bash
git add ur-cli/src/input.rs ur-cli/src/main.rs
git commit -m "feat: remove user-initiated dice roll — rolling is now automatic"
```

---

### Task 2: Add new `App` fields and constants

**Files:**
- Modify: `ur-cli/src/app.rs`

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)]` block in `ur-cli/src/app.rs`:

```rust
#[test]
fn test_new_app_auto_roll_fields_initial_state() {
    let app = App::new();
    assert!(!app.pending_roll);
    assert!(app.roll_after.is_none());
    assert!(app.forfeit_after.is_none());
    assert!(!app.rosette_reroll);
    assert!(app.last_opponent_roll.is_none());
}
```

- [ ] **Step 2: Run — expect FAIL** (fields don't exist yet)

```bash
cargo test -p ur-cli app::tests::test_new_app_auto_roll_fields_initial_state
```

- [ ] **Step 3: Add constants and fields**

After the existing constants near the top of `app.rs`:

```rust
/// Delay in ms before the dice roll animation fires automatically.
const AUTO_ROLL_DELAY_MS: u64 = 300;
/// How long (ms) to display a no-moves result before forfeiting.
const FORFEIT_DISPLAY_MS: u64 = 1000;
```

Add to the `App` struct (after `ai_spinner_frame`):

```rust
/// True when a dice roll should fire automatically as soon as conditions allow.
pub pending_roll: bool,
/// Earliest time at which the auto-roll may fire (None = fire immediately).
pub roll_after: Option<std::time::Instant>,
/// When set, the no-moves forfeit fires at this time.
pub forfeit_after: Option<std::time::Instant>,
/// True when the auto-roll is a rosette re-roll (skips the normal delay).
pub rosette_reroll: bool,
/// The most recent roll made by the AI, kept for display after apply_move clears dice_roll.
pub last_opponent_roll: Option<Dice>,
```

Add to `App::new()`:

```rust
pending_roll: false,
roll_after: None,
forfeit_after: None,
rosette_reroll: false,
last_opponent_roll: None,
```

- [ ] **Step 4: Run — expect PASS**

```bash
cargo test -p ur-cli app::tests::test_new_app_auto_roll_fields_initial_state
```

- [ ] **Step 5: Commit**

```bash
git add ur-cli/src/app.rs
git commit -m "feat: add pending_roll, roll_after, forfeit_after, rosette_reroll, last_opponent_roll fields"
```

---

### Task 3: Add `PanelDice` enum and `dice_pips_line` helper

**Files:**
- Modify: `ur-cli/src/ui/game.rs`

This task adds the data type and rendering helper. The next task wires them into the panel.

- [ ] **Step 1: Add `PanelDice` enum**

After the color constants at the top of `ur-cli/src/ui/game.rs`:

```rust
/// Describes what the dice widget inside a player panel should show.
#[derive(Debug, Clone, Copy)]
pub enum PanelDice {
    /// Nothing to show (panel is inactive and no prior roll to display).
    Hidden,
    /// Auto-roll is queued; waiting for the delay to elapse.
    Pending,
    /// Dice roll animation is in progress; carries the current cycling display value.
    Animating(Dice),
    /// Roll landed with legal moves available; carries the final value.
    Result(Dice),
    /// Roll landed with no legal moves; displayed in red before auto-forfeit.
    NoMoves(Dice),
    /// Opponent's last roll, shown dimmed in the inactive panel.
    LastRoll(Dice),
    /// Rosette extra turn granted; immediate re-roll incoming.
    RosettePending,
}
```

Also add `Dice` to the `use ur_core` import at the top so it is accessible in game.rs:

```rust
use ur_core::{
    board::Square,
    dice::Dice,
    player::Player,
    state::{Board, GameRules, PieceLocation},
};
```

- [ ] **Step 2: Add `dice_pips_line` helper**

Add this function after the color constants (before the `BK` enum):

```rust
/// Builds a `Line` showing four tetrahedral dice: ▲ for scored-side-up, △ for blank.
/// `value` filled dice are drawn in `color`; the rest in `DarkGray`.
fn dice_pips_line(value: u8, color: Color) -> Line<'static> {
    const FILLED: &str = "\u{25b2}"; // ▲
    const EMPTY:  &str = "\u{25b3}"; // △
    let mut spans = vec![Span::raw("  ")];
    for i in 0..4u8 {
        let (sym, c) = if i < value {
            (FILLED, color)
        } else {
            (EMPTY, Color::DarkGray)
        };
        spans.push(Span::styled(sym.to_string(), Style::default().fg(c)));
        if i < 3 {
            spans.push(Span::raw("  "));
        }
    }
    Line::from(spans)
}
```

- [ ] **Step 3: Compile check**

```bash
cargo build -p ur-cli 2>&1 | grep error
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add ur-cli/src/ui/game.rs
git commit -m "feat: add PanelDice enum and dice_pips_line helper to game UI"
```

---

### Task 4: Update `render_player_panel` to render the dice widget

**Files:**
- Modify: `ur-cli/src/ui/game.rs`

- [ ] **Step 1: Update `render_player_panel` signature**

Change the function signature to accept `panel_dice`:

```rust
pub fn render_player_panel(
    f: &mut Frame,
    area: Rect,
    player: Player,
    is_human: bool,
    is_current: bool,
    captures: u32,
    panel_dice: PanelDice,
) {
```

- [ ] **Step 2: Replace the function body's text construction**

Replace everything from `let text = vec![` through the closing `];` with:

```rust
    let mut text = vec![
        Line::from(Span::styled(
            format!("Captures: {}", captures),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            turn_indicator,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
    ];

    // Dice widget
    match panel_dice {
        PanelDice::Hidden => {}
        PanelDice::Pending => {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                "  rolling...",
                Style::default().fg(Color::DarkGray),
            )));
        }
        PanelDice::Animating(display) => {
            text.push(Line::from(""));
            text.push(dice_pips_line(display.value(), color));
            text.push(Line::from(Span::styled(
                "  = ?",
                Style::default().fg(Color::DarkGray),
            )));
        }
        PanelDice::Result(roll) => {
            text.push(Line::from(""));
            text.push(dice_pips_line(roll.value(), color));
            text.push(Line::from(Span::styled(
                format!("  = {}", roll.value()),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )));
            text.push(Line::from(Span::styled(
                "  pick a move",
                Style::default().fg(Color::Green),
            )));
        }
        PanelDice::NoMoves(roll) => {
            text.push(Line::from(""));
            text.push(dice_pips_line(roll.value(), Color::Red));
            text.push(Line::from(Span::styled(
                format!("  = {}  no moves", roll.value()),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            text.push(Line::from(Span::styled(
                "  passing turn...",
                Style::default().fg(Color::DarkGray),
            )));
        }
        PanelDice::LastRoll(roll) => {
            text.push(Line::from(""));
            text.push(dice_pips_line(roll.value(), Color::DarkGray));
            text.push(Line::from(Span::styled(
                format!("  = {} (last roll)", roll.value()),
                Style::default().fg(Color::DarkGray),
            )));
        }
        PanelDice::RosettePending => {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                "  \u{2736} rosette bonus!",
                Style::default().fg(Color::Yellow),
            )));
            text.push(Line::from(Span::styled(
                "  rolling again...",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
```

- [ ] **Step 3: Temporarily pass `PanelDice::Hidden` at both call sites in `render_game`**

Find the two `render_player_panel(` calls in `render_game` and add `PanelDice::Hidden` as the final argument:

```rust
render_player_panel(
    f,
    cols[0],
    Player::Player1,
    true,
    game_state.current_player == Player::Player1,
    app.stats.captures[0],
    PanelDice::Hidden,  // <-- add
);
render_player_panel(
    f,
    cols[2],
    Player::Player2,
    false,
    game_state.current_player == Player::Player2,
    app.stats.captures[1],
    PanelDice::Hidden,  // <-- add
);
```

- [ ] **Step 4: Compile check**

```bash
cargo build -p ur-cli 2>&1 | grep error
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add ur-cli/src/ui/game.rs
git commit -m "feat: render dice widget inside player panel (PanelDice::Hidden placeholder)"
```

---

### Task 5: Wire `PanelDice` computation and clean up `render_status_bar`

**Files:**
- Modify: `ur-cli/src/ui/game.rs`

- [ ] **Step 1: Add `compute_panel_dice` function**

Add this function in `ur-cli/src/ui/game.rs` just before `render_game`:

```rust
/// Computes what the dice widget should display for `player`'s panel.
fn compute_panel_dice(app: &crate::app::App, player: Player) -> PanelDice {
    let gs = match &app.game_state {
        Some(gs) => gs,
        None => return PanelDice::Hidden,
    };

    if gs.current_player == player {
        // Active player — show current roll state.
        if app.rosette_reroll && app.pending_roll {
            return PanelDice::RosettePending;
        }
        if app.pending_roll {
            return PanelDice::Pending;
        }
        if let Some(roll) = app.dice_roll {
            if app.forfeit_after.is_some() {
                return PanelDice::NoMoves(roll);
            }
            if let Some(crate::animation::Animation::DiceRoll { display, .. }) = &app.animation {
                return PanelDice::Animating(*display);
            }
            return PanelDice::Result(roll);
        }
        PanelDice::Hidden
    } else {
        // Inactive panel — show AI's last roll dimmed (only in Player2's panel).
        if player == Player::Player2 {
            if let Some(roll) = app.last_opponent_roll {
                return PanelDice::LastRoll(roll);
            }
        }
        PanelDice::Hidden
    }
}
```

- [ ] **Step 2: Replace `PanelDice::Hidden` placeholders in `render_game`**

```rust
render_player_panel(
    f,
    cols[0],
    Player::Player1,
    true,
    game_state.current_player == Player::Player1,
    app.stats.captures[0],
    compute_panel_dice(app, Player::Player1),
);
render_player_panel(
    f,
    cols[2],
    Player::Player2,
    false,
    game_state.current_player == Player::Player2,
    app.stats.captures[1],
    compute_panel_dice(app, Player::Player2),
);
```

- [ ] **Step 3: Remove `dice_roll` parameter from `render_status_bar`**

Update the signature (remove `dice_roll: Option<ur_core::dice::Dice>`):

```rust
pub fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    moves: u32,
    elapsed: std::time::Duration,
    last_log: Option<&str>,
    log_visible: bool,
    ai_thinking: bool,
    ai_spinner_frame: u32,
) {
```

Remove the `dice_str` variable and its usage. Replace the `left` construction:

```rust
    let left = format!(
        "Moves: {}  Time: {}  {}  {}",
        moves, time_str, ai_str, log_entry
    );
```

Remove `Space=Roll` from the `right` string:

```rust
    let right = format!(
        "  \u{2191}\u{2193}=Select  Enter=Move  Esc=Pause  {}",
        log_hint
    );
```

- [ ] **Step 4: Update the call site in `render_game`**

Remove the `app.dice_roll` argument from the `render_status_bar` call:

```rust
render_status_bar(
    f,
    status_area,
    app.stats.moves,
    elapsed,
    app.log.last().map(|s| s.as_str()),
    app.log_visible,
    app.ai_thinking,
    app.ai_spinner_frame,
);
```

- [ ] **Step 5: Compile and run tests**

```bash
cargo test -p ur-cli
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add ur-cli/src/ui/game.rs
git commit -m "feat: wire PanelDice into both panels; remove dice from status bar"
```

---

### Task 6: Implement `tick_auto_roll` and wire into `begin_game`

**Files:**
- Modify: `ur-cli/src/app.rs`

- [ ] **Step 1: Write failing tests**

Add to the `#[cfg(test)]` block in `ur-cli/src/app.rs`:

```rust
#[test]
fn test_tick_auto_roll_fires_when_deadline_past() {
    let mut app = App::new();
    app.screen = Screen::Game;
    app.game_state = Some(ur_core::state::GameState::new(
        &ur_core::state::GameRules::finkel(),
    ));
    app.pending_roll = true;
    // Deadline already in the past
    app.roll_after = Some(std::time::Instant::now()
        - std::time::Duration::from_millis(1));
    app.tick_auto_roll();
    assert!(!app.pending_roll, "pending_roll should be cleared after firing");
    assert!(app.dice_roll.is_some(), "dice_roll should be set");
    assert!(
        matches!(app.animation, Some(crate::animation::Animation::DiceRoll { .. })),
        "DiceRoll animation should start"
    );
}

#[test]
fn test_tick_auto_roll_waits_until_deadline() {
    let mut app = App::new();
    app.screen = Screen::Game;
    app.game_state = Some(ur_core::state::GameState::new(
        &ur_core::state::GameRules::finkel(),
    ));
    app.pending_roll = true;
    app.roll_after = Some(std::time::Instant::now()
        + std::time::Duration::from_secs(10));
    app.tick_auto_roll();
    assert!(app.pending_roll, "pending_roll should still be set before deadline");
    assert!(app.animation.is_none());
}

#[test]
fn test_tick_auto_roll_blocked_by_active_animation() {
    let mut app = App::new();
    app.screen = Screen::Game;
    app.game_state = Some(ur_core::state::GameState::new(
        &ur_core::state::GameRules::finkel(),
    ));
    app.pending_roll = true;
    app.roll_after = Some(std::time::Instant::now()
        - std::time::Duration::from_millis(1));
    app.animation = Some(crate::animation::Animation::PieceMove {
        remaining: vec![],
        frames_per_step: 3,
        frames_this_step: 3,
        is_player1: true,
    });
    app.tick_auto_roll();
    assert!(app.pending_roll, "pending_roll should survive while animation runs");
}

#[test]
fn test_tick_auto_roll_skipped_on_ai_turn() {
    let mut app = App::new();
    app.screen = Screen::Game;
    let mut gs = ur_core::state::GameState::new(&ur_core::state::GameRules::finkel());
    gs.current_player = ur_core::player::Player::Player2;
    app.game_state = Some(gs);
    app.pending_roll = true;
    app.roll_after = Some(std::time::Instant::now()
        - std::time::Duration::from_millis(1));
    app.tick_auto_roll();
    assert!(app.pending_roll, "auto-roll should not fire on AI's turn");
}

#[test]
fn test_begin_game_sets_pending_roll_for_player1() {
    let mut app = App::new();
    app.screen = Screen::Game;
    app.begin_game(ur_core::player::Player::Player1);
    assert!(app.pending_roll, "pending_roll must be set when Player1 goes first");
    assert!(app.roll_after.is_some(), "roll_after must be set for the 300ms delay");
}
```

- [ ] **Step 2: Run — expect FAIL**

```bash
cargo test -p ur-cli app::tests::test_tick_auto_roll_fires_when_deadline_past 2>&1 | tail -5
```

- [ ] **Step 3: Add `tick_auto_roll` to `App`**

Add this method to `impl App` in `ur-cli/src/app.rs`:

```rust
/// Called every tick. If `pending_roll` is set, `roll_after` has elapsed,
/// no animation is running, and it is the human player's turn, fires the
/// dice-roll animation automatically.
pub fn tick_auto_roll(&mut self) {
    if !self.pending_roll {
        return;
    }
    let is_human_turn = self
        .game_state
        .as_ref()
        .map(|gs| gs.current_player == ur_core::player::Player::Player1)
        .unwrap_or(false);
    if !is_human_turn {
        return;
    }
    if self.animation.is_some() {
        return;
    }
    let ready = self
        .roll_after
        .map(|t| std::time::Instant::now() >= t)
        .unwrap_or(true);
    if !ready {
        return;
    }
    self.pending_roll = false;
    self.rosette_reroll = false;
    self.roll_after = None;
    let final_value = Dice::roll(&mut self.rng);
    self.animation = Some(Animation::DiceRoll {
        frames_remaining: DICE_ROLL_ANIMATION_FRAMES,
        final_value,
        display: Dice(0),
    });
    self.dice_roll = Some(final_value);
}
```

- [ ] **Step 4: Update `begin_game` to set `pending_roll` for Player1**

Replace the last block of `begin_game`:

```rust
        // OLD:
        // if first_player == ur_core::player::Player::Player2 {
        //     self.start_ai_turn();
        // }

        // NEW:
        if first_player == ur_core::player::Player::Player2 {
            self.start_ai_turn();
        } else {
            self.pending_roll = true;
            self.roll_after = Some(
                std::time::Instant::now()
                    + std::time::Duration::from_millis(AUTO_ROLL_DELAY_MS),
            );
        }
```

- [ ] **Step 5: Run — expect PASS**

```bash
cargo test -p ur-cli app::tests::test_tick_auto_roll_fires_when_deadline_past
cargo test -p ur-cli app::tests::test_tick_auto_roll_waits_until_deadline
cargo test -p ur-cli app::tests::test_tick_auto_roll_blocked_by_active_animation
cargo test -p ur-cli app::tests::test_tick_auto_roll_skipped_on_ai_turn
cargo test -p ur-cli app::tests::test_begin_game_sets_pending_roll_for_player1
```

- [ ] **Step 6: Commit**

```bash
git add ur-cli/src/app.rs
git commit -m "feat: implement tick_auto_roll and wire into begin_game"
```

---

### Task 7: Wire `pending_roll` in `apply_move` (`last_opponent_roll`, `rosette_reroll`)

**Files:**
- Modify: `ur-cli/src/app.rs`

This task makes every completed move (human or AI) schedule the correct next step.

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn test_apply_move_sets_last_opponent_roll_from_ai_move() {
    use ur_core::{dice::Dice, player::Player, state::{GameRules, GameState, Move, Piece, PieceLocation}};
    let rules = GameRules::finkel();
    let mut app = App::new();
    app.screen = Screen::Game;
    // Put a Player2 piece on board so we have a legal move.
    let mut gs = GameState::new(&rules);
    gs.current_player = Player::Player2;
    let path = rules.path_for(Player::Player2);
    let from_sq = path.get(0).unwrap();
    let to_sq   = path.get(1).unwrap();
    gs.board.set(from_sq, Some(ur_core::state::Piece::new(Player::Player2, 0)));
    gs.unplayed[Player::Player2.index()] = 6;
    app.game_state = Some(gs);
    app.dice_roll = Some(Dice(1)); // pretend AI rolled 1
    let mv = Move {
        piece: ur_core::state::Piece::new(Player::Player2, 0),
        from: PieceLocation::OnBoard(from_sq),
        to: PieceLocation::OnBoard(to_sq),
    };
    app.apply_move(mv);
    assert_eq!(app.last_opponent_roll, Some(Dice(1)),
        "last_opponent_roll must be saved before dice_roll is cleared");
}

#[test]
fn test_apply_move_sets_pending_roll_after_human_move() {
    use ur_core::{dice::Dice, player::Player, state::{GameRules, GameState, Move, Piece, PieceLocation}};
    let rules = GameRules::finkel();
    let mut app = App::new();
    app.screen = Screen::Game;
    let mut gs = GameState::new(&rules);
    // Player1 piece at path[0]; roll=1 lands at path[1] (non-rosette).
    let path = rules.path_for(Player::Player1);
    let from_sq = path.get(0).unwrap();
    let to_sq   = path.get(1).unwrap();
    gs.board.set(from_sq, Some(ur_core::state::Piece::new(Player::Player1, 0)));
    gs.unplayed[Player::Player1.index()] = 6;
    app.game_state = Some(gs);
    app.dice_roll = Some(Dice(1));
    let mv = Move {
        piece: ur_core::state::Piece::new(Player::Player1, 0),
        from: PieceLocation::OnBoard(from_sq),
        to: PieceLocation::OnBoard(to_sq),
    };
    // After this move Player2 (AI) should get the turn → start_ai_turn fires.
    // Verify that pending_roll is NOT set (AI uses start_ai_turn, not pending_roll).
    // This indirectly tests that apply_move doesn't incorrectly set pending_roll for AI turn.
    app.apply_move(mv);
    // AI turn: ai_thinking should be true (start_ai_turn ran), pending_roll should be false.
    assert!(!app.pending_roll, "pending_roll must not be set when it's the AI's turn");
}
```

- [ ] **Step 2: Run — some will pass, some fail; note which**

```bash
cargo test -p ur-cli app::tests::test_apply_move_sets_last_opponent_roll_from_ai_move
cargo test -p ur-cli app::tests::test_apply_move_sets_pending_roll_after_human_move
```

- [ ] **Step 3: Update `apply_move` — save `last_opponent_roll`, set `pending_roll`**

Find the section in `apply_move` that starts with:
```rust
        self.game_state = Some(result.new_state.clone());
        self.dice_roll = None;
```

Replace with:

```rust
        self.game_state = Some(result.new_state.clone());
        // Save AI's roll before clearing, so the AI panel can show it dimmed.
        if mv.piece.player == ur_core::player::Player::Player2 {
            self.last_opponent_roll = self.dice_roll;
        }
        self.dice_roll = None;
```

Find the section at the bottom of `apply_move`:
```rust
        if result.new_state.current_player == ur_core::player::Player::Player2 {
            self.start_ai_turn();
        }
```

Replace with:

```rust
        if result.new_state.current_player == ur_core::player::Player::Player2 {
            self.start_ai_turn();
        } else {
            // Human's turn — schedule auto-roll.
            self.pending_roll = true;
            self.rosette_reroll = result.landed_on_rosette;
            // Rosette re-rolls skip the delay; normal transitions get 300 ms.
            self.roll_after = if result.landed_on_rosette {
                None
            } else {
                Some(
                    std::time::Instant::now()
                        + std::time::Duration::from_millis(AUTO_ROLL_DELAY_MS),
                )
            };
        }
```

- [ ] **Step 4: Run — expect PASS**

```bash
cargo test -p ur-cli app::tests::test_apply_move_sets_last_opponent_roll_from_ai_move
cargo test -p ur-cli app::tests::test_apply_move_sets_pending_roll_after_human_move
```

- [ ] **Step 5: Run full suite**

```bash
cargo test -p ur-cli
```

- [ ] **Step 6: Commit**

```bash
git add ur-cli/src/app.rs
git commit -m "feat: save last_opponent_roll and schedule pending_roll in apply_move"
```

---

### Task 8: Implement `tick_forfeit_delay` and update `on_animation_done` + `start_ai_turn`

**Files:**
- Modify: `ur-cli/src/app.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn test_on_animation_done_sets_forfeit_after_on_no_moves() {
    let rules = ur_core::state::GameRules::finkel();
    let mut app = App::new();
    app.screen = Screen::Game;
    app.game_state = Some(ur_core::state::GameState::new(&rules));
    // Roll 0 — guaranteed no moves.
    app.dice_roll = Some(ur_core::dice::Dice(0));
    app.on_animation_done();
    assert!(app.forfeit_after.is_some(),
        "forfeit_after must be set when there are no legal moves");
    assert!(app.dice_roll.is_some(),
        "dice_roll must NOT be cleared yet — panel shows red state until forfeit fires");
    assert!(app.legal_moves.is_empty(),
        "no legal moves should be populated");
}

#[test]
fn test_tick_forfeit_delay_advances_to_ai_when_deadline_past() {
    let rules = ur_core::state::GameRules::finkel();
    let mut app = App::new();
    app.screen = Screen::Game;
    app.game_state = Some(ur_core::state::GameState::new(&rules));
    app.dice_roll = Some(ur_core::dice::Dice(0));
    // Deadline already past
    app.forfeit_after = Some(std::time::Instant::now()
        - std::time::Duration::from_millis(1));
    app.tick_forfeit_delay();
    assert!(app.forfeit_after.is_none(), "forfeit_after must be cleared after firing");
    assert!(app.dice_roll.is_none(), "dice_roll must be cleared after forfeit");
    // Player1 forfeited → Player2 (AI) should now be active → ai_thinking true
    assert!(app.ai_thinking, "AI turn should have started after forfeit");
}

#[test]
fn test_tick_forfeit_delay_waits_until_deadline() {
    let rules = ur_core::state::GameRules::finkel();
    let mut app = App::new();
    app.screen = Screen::Game;
    app.game_state = Some(ur_core::state::GameState::new(&rules));
    app.dice_roll = Some(ur_core::dice::Dice(0));
    app.forfeit_after = Some(std::time::Instant::now()
        + std::time::Duration::from_secs(10));
    app.tick_forfeit_delay();
    assert!(app.forfeit_after.is_some(), "forfeit_after must not fire before deadline");
}
```

- [ ] **Step 2: Run — expect FAIL**

```bash
cargo test -p ur-cli app::tests::test_on_animation_done_sets_forfeit_after_on_no_moves
```

- [ ] **Step 3: Update `on_animation_done` — replace immediate forfeit with `forfeit_after`**

Find the `if moves.is_empty()` block inside `on_animation_done`:

```rust
                if moves.is_empty() {
                    self.log
                        .push(format!("Roll {} — no moves, turn forfeited", roll.value()));
                    if let Some(new_gs) = gs.clone().forfeit_turn() {
                        let next_player = new_gs.current_player;
                        self.game_state = Some(new_gs);
                        if next_player == ur_core::player::Player::Player2 {
                            self.start_ai_turn();
                        }
                    }
                    self.dice_roll = None;
                } else {
```

Replace with:

```rust
                if moves.is_empty() {
                    self.log
                        .push(format!("Roll {} — no moves, passing turn", roll.value()));
                    // Show the no-moves (red) state for FORFEIT_DISPLAY_MS before
                    // auto-advancing. dice_roll is kept so the panel renders red.
                    self.forfeit_after = Some(
                        std::time::Instant::now()
                            + std::time::Duration::from_millis(FORFEIT_DISPLAY_MS),
                    );
                } else {
```

- [ ] **Step 4: Add `tick_forfeit_delay` method to `impl App`**

```rust
/// Called every tick. If `forfeit_after` has elapsed, forfeits the current
/// player's turn and starts the next player's turn.
pub fn tick_forfeit_delay(&mut self) {
    let deadline = match self.forfeit_after {
        Some(t) => t,
        None => return,
    };
    if std::time::Instant::now() < deadline {
        return;
    }
    self.forfeit_after = None;
    self.dice_roll = None;
    let gs = match self.game_state.take() {
        Some(gs) => gs,
        None => return,
    };
    if let Some(new_gs) = gs.forfeit_turn() {
        let next_player = new_gs.current_player;
        self.game_state = Some(new_gs);
        if next_player == ur_core::player::Player::Player2 {
            self.start_ai_turn();
        } else {
            self.pending_roll = true;
            self.roll_after = Some(
                std::time::Instant::now()
                    + std::time::Duration::from_millis(AUTO_ROLL_DELAY_MS),
            );
        }
    }
}
```

- [ ] **Step 5: Update `start_ai_turn` — use `forfeit_after` for AI no-moves too**

Find the `if moves.is_empty()` block in `start_ai_turn`:

```rust
        if moves.is_empty() {
            self.log
                .push("AI has no moves — turn forfeited".to_string());
            if let Some(new_gs) = gs.forfeit_turn() {
                let next_player = new_gs.current_player;
                self.game_state = Some(new_gs);
                // Re-trigger if AI still has the turn (e.g., after a rosette extra-turn that rolls 0)
                if next_player == ur_core::player::Player::Player2 {
                    self.start_ai_turn();
                }
            }
            return;
        }
```

Replace with:

```rust
        if moves.is_empty() {
            self.log
                .push("AI has no moves — passing turn".to_string());
            // Show the no-moves (red) state in the AI panel for FORFEIT_DISPLAY_MS.
            self.dice_roll = Some(roll); // panel needs the value to render red
            self.forfeit_after = Some(
                std::time::Instant::now()
                    + std::time::Duration::from_millis(FORFEIT_DISPLAY_MS),
            );
            return;
        }
```

- [ ] **Step 6: Run — expect PASS**

```bash
cargo test -p ur-cli app::tests::test_on_animation_done_sets_forfeit_after_on_no_moves
cargo test -p ur-cli app::tests::test_tick_forfeit_delay_advances_to_ai_when_deadline_past
cargo test -p ur-cli app::tests::test_tick_forfeit_delay_waits_until_deadline
```

- [ ] **Step 7: Run full suite**

```bash
cargo test -p ur-cli
```

- [ ] **Step 8: Commit**

```bash
git add ur-cli/src/app.rs
git commit -m "feat: implement tick_forfeit_delay; show no-moves red state for 1s before auto-advance"
```

---

### Task 9: Call `tick_auto_roll` and `tick_forfeit_delay` from `animation::tick`

**Files:**
- Modify: `ur-cli/src/animation.rs`

This is the final wiring step that makes the two helpers run every frame.

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)]` block in `ur-cli/src/animation.rs`:

```rust
#[test]
fn test_tick_calls_tick_auto_roll() {
    use crate::app::{App, Screen};
    use ur_core::{dice::Dice, state::{GameRules, GameState}};

    let mut app = App::new();
    app.screen = Screen::Game;
    app.game_state = Some(GameState::new(&GameRules::finkel()));
    // Set pending_roll with an already-elapsed deadline
    app.pending_roll = true;
    app.roll_after = Some(std::time::Instant::now()
        - std::time::Duration::from_millis(1));

    tick(&mut app);

    assert!(
        matches!(app.animation, Some(Animation::DiceRoll { .. })),
        "tick() must fire auto-roll when pending_roll is set and deadline is past"
    );
}

#[test]
fn test_tick_calls_tick_forfeit_delay() {
    use crate::app::{App, Screen};
    use ur_core::{dice::Dice, state::{GameRules, GameState}};

    let mut app = App::new();
    app.screen = Screen::Game;
    app.game_state = Some(GameState::new(&GameRules::finkel()));
    app.dice_roll = Some(Dice(0));
    app.forfeit_after = Some(std::time::Instant::now()
        - std::time::Duration::from_millis(1));

    tick(&mut app);

    assert!(app.forfeit_after.is_none(), "tick() must fire forfeit when deadline is past");
    assert!(app.dice_roll.is_none(), "dice_roll must be cleared after forfeit");
}
```

- [ ] **Step 2: Run — expect FAIL**

```bash
cargo test -p ur-cli animation::tests::test_tick_calls_tick_auto_roll
```

- [ ] **Step 3: Add calls to `tick()` in `animation.rs`**

Find the `pub fn tick(app: &mut App)` function. After the `// AI spinner` block at the end, add:

```rust
    // Auto-roll and forfeit delay
    app.tick_auto_roll();
    app.tick_forfeit_delay();
```

The complete `tick` function should end:

```rust
    // AI spinner
    if app.ai_thinking {
        app.ai_spinner_frame = (app.ai_spinner_frame + 1) % 4;
        app.poll_ai_move();
    }

    // Auto-roll and forfeit delay
    app.tick_auto_roll();
    app.tick_forfeit_delay();
}
```

- [ ] **Step 4: Run — expect PASS**

```bash
cargo test -p ur-cli animation::tests::test_tick_calls_tick_auto_roll
cargo test -p ur-cli animation::tests::test_tick_calls_tick_forfeit_delay
```

- [ ] **Step 5: Run full workspace tests**

```bash
cargo test --workspace
```

- [ ] **Step 6: Lint check**

```bash
cargo clippy --all-targets --all-features 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 7: Commit**

```bash
git add ur-cli/src/animation.rs
git commit -m "feat: call tick_auto_roll and tick_forfeit_delay every frame — auto-roll fully wired"
```

---

## Done

After Task 9 the feature is complete:

- Dice shown as ▲/△ inside both player panels in all five states
- Rolling is automatic — Space key removed
- No-moves state shown in red for 1 s before auto-advancing
- AI's last roll shown dimmed in the AI panel when it is the human's turn
- Status bar no longer shows dice or the `Space=Roll` hint
