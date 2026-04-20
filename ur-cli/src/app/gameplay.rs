use super::navigation::{nav_cursor, NavDir};
use super::{
    App, LogEntry, Screen, AUTO_ROLL_DELAY_MS, CURSOR_BEAR_OFF, DICE_ROLL_ANIMATION_FRAMES,
    FORFEIT_DISPLAY_MS,
};
use crate::animation::Animation;
use ur_core::{
    dice::Dice,
    player::Player,
    state::{GameState, Move},
};

impl App {
    /// Starts the game with the given first player, initialising game state and
    /// transitioning to the `Game` screen.
    pub fn begin_game(&mut self, first_player: Player) {
        let rules = ur_core::state::GameRules::finkel();
        let mut gs = GameState::new(&rules);
        gs.current_player = first_player;
        self.game_state = Some(gs);
        self.stats = super::GameStats {
            start_time: Some(std::time::Instant::now()),
            ..Default::default()
        };
        self.log.clear();
        self.dice_roll = None;
        self.legal_moves.clear();
        self.cursor_path_pos = 0;
        self.animation = None;
        self.ai_thinking = false;
        self.ai_receiver = None;
        self.ai_spinner_frame = 0;
        self.last_roll = [None, None];
        self.last_event = [None, None];
        self.turn_log = [vec![vec![]], vec![vec![]]];
        self.rosette_reroll = false;
        self.forfeit_after = None;
        self.screen = Screen::Game;

        if first_player == Player::Player2 {
            self.start_ai_turn();
        } else {
            self.pending_roll = true;
            self.roll_after = Some(
                std::time::Instant::now() + std::time::Duration::from_millis(AUTO_ROLL_DELAY_MS),
            );
        }
    }

    /// Rolls the dice for Player 1, starting a dice animation.
    ///
    /// Only allowed during Player1's turn, in `WaitingForRoll` phase, when no
    /// animation is currently running and no roll is already pending.
    #[allow(dead_code)]
    pub fn handle_roll_dice(&mut self) {
        let gs = match &self.game_state {
            Some(gs) => gs,
            None => return,
        };
        if gs.current_player != Player::Player1 {
            return;
        }
        if !matches!(gs.phase, ur_core::state::GamePhase::WaitingForRoll) {
            return;
        }
        if self.animation.is_some() {
            return;
        }
        if self.dice_roll.is_some() {
            return;
        }

        let final_value = Dice::roll(&mut self.rng);
        self.animation = Some(Animation::DiceRoll {
            frames_remaining: DICE_ROLL_ANIMATION_FRAMES,
            final_value,
            display: Dice::new(0).unwrap(),
        });
        self.dice_roll = Some(final_value);
    }

    /// Moves the cursor one step in `dir` on the 2-D board grid.
    /// Only active when legal moves are available (i.e. human player's turn).
    pub fn handle_nav(&mut self, dir: NavDir) {
        if self.legal_moves.is_empty() {
            return;
        }
        self.cursor_path_pos = nav_cursor(self.cursor_path_pos, dir);
    }

    /// Confirms the move at the current cursor position, if one exists.
    pub fn handle_confirm_move(&mut self) {
        if self.animation.is_some() {
            return;
        }
        let mv = match self.legal_move_at_cursor() {
            Some(m) => m.clone(),
            None => return,
        };
        self.apply_move(mv);
    }

    /// Returns the legal move whose source matches the current cursor position, if any.
    pub fn legal_move_at_cursor(&self) -> Option<&Move> {
        let gs = self.game_state.as_ref()?;
        let from = if self.cursor_path_pos == 0 {
            ur_core::state::PieceLocation::Unplayed
        } else {
            let sq = gs
                .rules
                .path_for(gs.current_player)
                .get(self.cursor_path_pos - 1)?;
            ur_core::state::PieceLocation::OnBoard(sq)
        };
        self.legal_moves.iter().find(|mv| mv.from == from)
    }

    /// Applies a move to the current game state and handles turn transitions.
    pub fn apply_move(&mut self, mv: Move) {
        let gs = match self.game_state.take() {
            Some(gs) => gs,
            None => return,
        };
        let current_player = gs.current_player;
        let result = gs.apply_move(mv.clone());
        self.stats.moves += 1;

        let player_idx = mv.piece.player.index();
        let panel_event = self.build_move_log(&mv, &result, current_player);
        self.last_event[player_idx] = panel_event;

        self.game_state = Some(result.new_state.clone());
        self.last_roll[player_idx] = self.dice_roll;
        self.dice_roll = None;
        self.legal_moves.clear();

        if result.game_over {
            self.stats.end_time = Some(std::time::Instant::now());
            self.screen = Screen::GameOver;
            return;
        }

        self.set_move_animation(&gs.rules, &mv, &result);
        self.transition_turn(&result);
    }

    fn emit_event(&mut self, player: Player, player_idx: usize, text: String) {
        self.log.push(LogEntry {
            player: Some(player),
            text: text.clone(),
        });
        if let Some(cur) = self.turn_log[player_idx].last_mut() {
            cur.push(text);
        }
    }

    /// Builds log entries for a move and returns the panel event string.
    fn build_move_log(
        &mut self,
        mv: &Move,
        result: &ur_core::state::MoveResult,
        current_player: Player,
    ) -> Option<String> {
        let idx = mv.piece.player.index();
        match &mv.to {
            ur_core::state::PieceLocation::OnBoard(sq) => {
                let step = result
                    .new_state
                    .rules
                    .path_for(mv.piece.player)
                    .squares()
                    .iter()
                    .position(|&s| s == *sq)
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let t = if result.captured.is_some() {
                    self.stats.captures[idx] += 1;
                    format!("Captured piece at {}!", step)
                } else if result.landed_on_rosette {
                    format!("Moved to rosette at {}", step)
                } else {
                    format!("Moved to {}", step)
                };
                self.emit_event(current_player, idx, t.clone());
                if result.landed_on_rosette {
                    self.emit_event(current_player, idx, "Extra turn!".to_string());
                }
                Some(t)
            }
            ur_core::state::PieceLocation::Scored => {
                let t = "Scored!".to_string();
                self.emit_event(current_player, idx, t.clone());
                Some(t)
            }
            _ => None,
        }
    }

    /// Sets the cosmetic animation for a completed move (capture flash or piece walk).
    fn set_move_animation(
        &mut self,
        rules: &ur_core::state::GameRules,
        mv: &Move,
        result: &ur_core::state::MoveResult,
    ) {
        if result.captured.is_some() {
            if let ur_core::state::PieceLocation::OnBoard(sq) = mv.to {
                self.animation = Some(Animation::CaptureFlash {
                    square: sq,
                    frames_remaining: 18,
                });
            }
        }
        if self.animation.is_none() {
            let path_squares = rules.move_path(mv);
            if path_squares.len() > 1 {
                let is_player1 = mv.piece.player == Player::Player1;
                self.animation = Some(Animation::PieceMove {
                    remaining: path_squares,
                    current_idx: 0,
                    frames_per_step: 7,
                    frames_this_step: 7,
                    is_player1,
                });
            }
        }
    }

    /// Schedules the next player's turn after a move completes.
    fn transition_turn(&mut self, result: &ur_core::state::MoveResult) {
        if result.new_state.current_player == Player::Player2 {
            self.start_ai_turn();
        } else {
            self.pending_roll = true;
            self.rosette_reroll = result.landed_on_rosette;
            self.roll_after = if result.landed_on_rosette {
                None
            } else {
                Some(
                    std::time::Instant::now()
                        + std::time::Duration::from_millis(AUTO_ROLL_DELAY_MS),
                )
            };
        }
    }

    /// Called when the active animation completes.
    ///
    /// Computes legal moves from the rolled dice; forfeits the turn if none exist.
    pub fn on_animation_done(&mut self) {
        if self.dice_roll.is_none() {
            return;
        }
        if let (Some(roll), Some(gs)) = (self.dice_roll, self.game_state.as_ref()) {
            let idx = gs.current_player.index();
            if let Some(current) = self.turn_log[idx].last_mut() {
                current.push(format!("Rolled {}", roll.value()));
            }
        }
        if self.ai_thinking {
            return;
        }
        if let Some(roll) = self.dice_roll {
            if let Some(gs) = &self.game_state {
                let moves = gs.legal_moves(roll);
                let current_player = gs.current_player;
                let idx = current_player.index();
                if moves.is_empty() {
                    self.log.push(LogEntry {
                        player: Some(current_player),
                        text: "No moves!".to_string(),
                    });
                    if let Some(current) = self.turn_log[idx].last_mut() {
                        current.push("No moves!".to_string());
                    }
                    self.forfeit_after = Some(
                        std::time::Instant::now()
                            + std::time::Duration::from_millis(FORFEIT_DISPLAY_MS),
                    );
                } else {
                    self.legal_moves = moves;
                    self.cursor_path_pos = self.cursor_path_pos.min(CURSOR_BEAR_OFF);
                }
            }
        }
    }

    /// Rolls dice for the AI and either forfeits the turn or spawns a background
    /// thread to run [`ur_core::ai::choose_move`].
    pub fn start_ai_turn(&mut self) {
        let gs = match self.game_state.as_ref() {
            Some(gs) => gs.clone(),
            None => return,
        };

        let roll = Dice::roll(&mut self.rng);
        self.log.push(LogEntry {
            player: Some(Player::Player2),
            text: format!("Rolled {}", roll.value()),
        });
        self.dice_roll = Some(roll);
        self.turn_log[1].push(vec![]);
        if self.turn_log[1].len() > 5 {
            self.turn_log[1].remove(0);
        }
        self.last_roll[gs.current_player.index()] = Some(roll);

        self.animation = Some(Animation::DiceRoll {
            frames_remaining: DICE_ROLL_ANIMATION_FRAMES,
            final_value: roll,
            display: Dice::new(0).unwrap(),
        });

        let moves = gs.legal_moves(roll);
        if moves.is_empty() {
            return;
        }

        let depth = self.difficulty;
        let (tx, rx) = std::sync::mpsc::channel();
        self.ai_receiver = Some(rx);
        self.ai_thinking = true;

        std::thread::spawn(move || {
            let chosen = ur_core::ai::choose_move(&gs, roll, depth);
            let _ = tx.send(chosen);
        });
    }

    /// Called every tick. If `pending_roll` is set and the deadline has elapsed,
    /// fires the dice-roll animation automatically.
    pub fn tick_auto_roll(&mut self) {
        if !self.pending_roll {
            return;
        }
        let is_human_turn = self
            .game_state
            .as_ref()
            .map(|gs| gs.current_player == Player::Player1)
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
            display: Dice::new(0).unwrap(),
        });
        self.dice_roll = Some(final_value);
        self.log.push(LogEntry {
            player: Some(Player::Player1),
            text: format!("Rolled {}", final_value.value()),
        });
        self.turn_log[0].push(vec![]);
        if self.turn_log[0].len() > 5 {
            self.turn_log[0].remove(0);
        }
    }

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
        if let Some(ref gs) = self.game_state {
            self.last_roll[gs.current_player.index()] = self.dice_roll;
        }
        self.dice_roll = None;
        let gs = match self.game_state.take() {
            Some(gs) => gs,
            None => return,
        };
        if let Some(new_gs) = gs.forfeit_turn() {
            let next_player = new_gs.current_player;
            self.game_state = Some(new_gs);
            if next_player == Player::Player2 {
                self.start_ai_turn();
            } else {
                self.pending_roll = true;
                self.roll_after = Some(
                    std::time::Instant::now()
                        + std::time::Duration::from_millis(AUTO_ROLL_DELAY_MS),
                );
            }
        } else {
            self.game_state = Some(gs);
        }
    }

    /// Polls the AI move channel (non-blocking). If a result is ready, clears
    /// the thinking state and applies the move.
    pub fn poll_ai_move(&mut self) {
        if matches!(self.animation, Some(Animation::DiceRoll { .. })) {
            return;
        }
        let mv = match self.ai_receiver.as_ref() {
            Some(rx) => match rx.try_recv() {
                Ok(m) => m,
                Err(std::sync::mpsc::TryRecvError::Empty) => return,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.ai_receiver = None;
                    self.ai_thinking = false;
                    self.log.push(LogEntry {
                        player: Some(Player::Player2),
                        text: "error — turn skipped".to_string(),
                    });
                    return;
                }
            },
            None => return,
        };

        self.ai_receiver = None;
        self.ai_thinking = false;
        self.apply_move(mv);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Screen;

    fn make_app_with_game() -> App {
        let mut app = App::new();
        let rules = ur_core::state::GameRules::finkel();
        app.game_state = Some(GameState::new(&rules));
        app
    }

    fn app_with_moves() -> App {
        let mut app = App::new();
        let gs = GameState::new(&ur_core::state::GameRules::finkel());
        app.legal_moves = gs.legal_moves(Dice::new(1).unwrap());
        app.game_state = Some(gs);
        app
    }

    #[test]
    fn test_roll_dice_starts_dice_animation() {
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(GameState::new(&ur_core::state::GameRules::finkel()));
        app.handle_roll_dice();
        assert!(matches!(app.animation, Some(Animation::DiceRoll { .. })));
    }

    #[test]
    fn test_roll_dice_ignored_when_dice_already_rolled() {
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(GameState::new(&ur_core::state::GameRules::finkel()));
        app.dice_roll = Some(Dice::new(3).unwrap());
        app.animation = None;
        app.handle_roll_dice();
        assert_eq!(app.dice_roll, Some(Dice::new(3).unwrap()));
        assert!(app.animation.is_none());
    }

    #[test]
    fn test_roll_dice_ignored_when_animation_active() {
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(GameState::new(&ur_core::state::GameRules::finkel()));
        app.animation = Some(Animation::Done);
        app.handle_roll_dice();
        assert!(matches!(app.animation, Some(Animation::Done)));
    }

    #[test]
    fn test_handle_nav_no_op_when_no_legal_moves() {
        let mut app = App::new();
        app.cursor_path_pos = 3;
        app.handle_nav(NavDir::Right);
        assert_eq!(app.cursor_path_pos, 3);
    }

    #[test]
    fn test_handle_nav_moves_cursor_when_legal_moves_present() {
        let mut app = app_with_moves();
        app.cursor_path_pos = 1;
        app.handle_nav(NavDir::Right);
        assert_eq!(app.cursor_path_pos, 8);
    }

    #[test]
    fn test_new_app_auto_roll_fields_initial_state() {
        let app = App::new();
        assert!(!app.pending_roll);
        assert!(app.roll_after.is_none());
        assert!(app.forfeit_after.is_none());
        assert!(!app.rosette_reroll);
        assert!(app.last_roll[0].is_none());
        assert!(app.last_roll[1].is_none());
    }

    #[test]
    fn test_tick_auto_roll_fires_when_deadline_past() {
        let mut app = make_app_with_game();
        app.screen = Screen::Game;
        app.pending_roll = true;
        app.roll_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        app.tick_auto_roll();
        assert!(!app.pending_roll);
        assert!(app.dice_roll.is_some());
        assert!(matches!(app.animation, Some(Animation::DiceRoll { .. })));
    }

    #[test]
    fn test_tick_auto_roll_waits_until_deadline() {
        let mut app = make_app_with_game();
        app.screen = Screen::Game;
        app.pending_roll = true;
        app.roll_after = Some(std::time::Instant::now() + std::time::Duration::from_secs(10));
        app.tick_auto_roll();
        assert!(app.pending_roll);
        assert!(app.animation.is_none());
    }

    #[test]
    fn test_tick_auto_roll_blocked_by_active_animation() {
        let mut app = make_app_with_game();
        app.screen = Screen::Game;
        app.pending_roll = true;
        app.roll_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        app.animation = Some(Animation::PieceMove {
            remaining: vec![],
            current_idx: 0,
            frames_per_step: 3,
            frames_this_step: 3,
            is_player1: true,
        });
        app.tick_auto_roll();
        assert!(app.pending_roll);
    }

    #[test]
    fn test_tick_auto_roll_skipped_on_ai_turn() {
        let mut app = App::new();
        app.screen = Screen::Game;
        let mut gs = GameState::new(&ur_core::state::GameRules::finkel());
        gs.current_player = Player::Player2;
        app.game_state = Some(gs);
        app.pending_roll = true;
        app.roll_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        app.tick_auto_roll();
        assert!(app.pending_roll);
    }

    #[test]
    fn test_begin_game_sets_pending_roll_for_player1() {
        let mut app = App::new();
        app.screen = Screen::Game;
        app.begin_game(Player::Player1);
        assert!(app.pending_roll);
        assert!(app.roll_after.is_some());
    }

    #[test]
    fn test_apply_move_sets_last_opponent_roll_from_ai_move() {
        let rules = ur_core::state::GameRules::finkel();
        let mut app = App::new();
        app.screen = Screen::Game;
        let mut gs = GameState::new(&rules);
        gs.current_player = Player::Player2;
        app.game_state = Some(gs.clone());
        app.dice_roll = Some(Dice::new(1).unwrap());
        let moves = gs.legal_moves(Dice::new(1).unwrap());
        let mv = moves
            .into_iter()
            .next()
            .expect("should have at least one legal move");
        app.apply_move(mv);
        assert_eq!(app.last_roll[1], Some(Dice::new(1).unwrap()));
    }

    #[test]
    fn test_on_animation_done_sets_forfeit_after_on_no_moves() {
        let mut app = make_app_with_game();
        app.screen = Screen::Game;
        app.dice_roll = Some(Dice::new(0).unwrap());
        app.on_animation_done();
        assert!(app.forfeit_after.is_some());
        assert!(app.dice_roll.is_some());
        assert!(app.legal_moves.is_empty());
    }

    #[test]
    fn test_tick_forfeit_delay_advances_to_ai_when_deadline_past() {
        let mut app = make_app_with_game();
        app.screen = Screen::Game;
        app.dice_roll = Some(Dice::new(0).unwrap());
        app.forfeit_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        app.tick_forfeit_delay();
        assert!(app.forfeit_after.is_none());
        assert!(app.ai_thinking);
    }

    #[test]
    fn test_tick_forfeit_delay_waits_until_deadline() {
        let mut app = make_app_with_game();
        app.screen = Screen::Game;
        app.dice_roll = Some(Dice::new(0).unwrap());
        app.forfeit_after = Some(std::time::Instant::now() + std::time::Duration::from_secs(10));
        app.tick_forfeit_delay();
        assert!(app.forfeit_after.is_some());
    }

    #[test]
    fn test_apply_move_sets_pending_roll_when_ai_move_returns_to_human() {
        let rules = ur_core::state::GameRules::finkel();
        let mut app = App::new();
        app.screen = Screen::Game;
        let mut gs = GameState::new(&rules);
        gs.current_player = Player::Player2;
        let path = rules.path_for(Player::Player2);
        let to_sq = path.get(0).unwrap();
        assert!(!rules.board_shape.is_rosette(to_sq));
        let moves = gs.legal_moves(Dice::new(1).unwrap());
        let mv = moves
            .into_iter()
            .next()
            .expect("should have at least one legal move");
        app.game_state = Some(gs);
        app.dice_roll = Some(Dice::new(1).unwrap());
        app.apply_move(mv);
        assert!(app.pending_roll);
        assert!(!app.rosette_reroll);
        assert!(app.roll_after.is_some());
    }

    #[test]
    fn test_ai_roll_pushes_log_entry_with_player2() {
        let mut app = make_app_with_game();
        if let Some(gs) = &mut app.game_state {
            gs.current_player = Player::Player2;
        }
        app.start_ai_turn();
        let last = app.log.last().unwrap();
        assert_eq!(last.player, Some(Player::Player2));
        assert!(last.text.contains("Rolled"));
    }

    #[test]
    fn test_human_auto_roll_pushes_log_entry_with_player1() {
        let mut app = make_app_with_game();
        app.pending_roll = true;
        app.roll_after = None;
        app.tick_auto_roll();
        let has_p1_roll = app
            .log
            .iter()
            .any(|e| e.player == Some(Player::Player1) && e.text.contains("Rolled"));
        assert!(has_p1_roll);
    }
}
