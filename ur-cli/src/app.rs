use crate::animation::Animation;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;
use ur_core::{
    dice::Dice,
    state::{GameState, Move, PieceLocation},
};

/// Which screen is currently active.
#[derive(Debug)]
#[allow(dead_code)]
pub enum Screen {
    Title,
    DifficultySelect {
        selected: usize,
    },
    DiceOff {
        state: DiceOffState,
    },
    Game,
    /// Pause overlay shown when pressing Esc during gameplay.
    /// `selected`: 0 = Resume, 1 = How to Play, 2 = Quit
    PauseMenu {
        selected: usize,
    },
    /// Rules + key-binding help screen.
    /// `from_game`: true when reached via the pause menu; Back returns to PauseMenu.
    /// false when reached from the title screen; Back returns to Title.
    Help {
        from_game: bool,
    },
    GameOver,
}

/// State for the first-player dice-off animation.
#[derive(Debug)]
#[allow(dead_code)]
pub struct DiceOffState {
    pub p1_frames: u32,
    pub p2_frames: u32,
    pub p1_final: Dice,
    pub p2_final: Dice,
    pub p1_display: Dice,
    pub p2_display: Dice,
    pub winner: Option<ur_core::player::Player>,
    pub acknowledged: bool,
}

/// Number of animation ticks for the dice-off countdown (≈1.5 s at 12 fps).
const DICE_OFF_ANIMATION_FRAMES: u32 = 18;

/// Number of animation ticks for a dice roll animation (≈0.6 s at 30 fps).
const DICE_ROLL_ANIMATION_FRAMES: u32 = 18;

/// Delay in ms before the dice roll animation fires automatically.
const AUTO_ROLL_DELAY_MS: u64 = 300;
/// How long (ms) to display a no-moves result before forfeiting.
const FORFEIT_DISPLAY_MS: u64 = 1000;

/// Difficulty level maps to expectiminimax search depth.
pub const DIFFICULTIES: [(&str, u32); 3] = [("Easy", 2), ("Medium", 4), ("Hard", 6)];

/// Game statistics accumulated during play.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct GameStats {
    pub moves: u32,
    pub start_time: Option<Instant>,
    pub captures: [u32; 2],
}

/// Top-level application state.
#[allow(dead_code)]
pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub difficulty: u32,
    pub game_state: Option<GameState>,
    pub dice_roll: Option<Dice>,
    pub legal_moves: Vec<Move>,
    /// Cursor position along the player's path. 0 = pool, 1..=14 = path steps.
    pub cursor_path_pos: usize,
    pub log: Vec<String>,
    pub log_visible: bool,
    pub stats: GameStats,
    pub rng: SmallRng,
    pub title_selected: usize,
    pub animation: Option<Animation>,
    pub ai_thinking: bool,
    pub ai_spinner_frame: u32,
    pub ai_receiver: Option<std::sync::mpsc::Receiver<ur_core::state::Move>>,
    /// Scroll offset for the help screen (lines scrolled down).
    pub help_scroll: u16,
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
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Title,
            should_quit: false,
            difficulty: 4,
            game_state: None,
            dice_roll: None,
            legal_moves: Vec::new(),
            cursor_path_pos: 0,
            log: Vec::new(),
            log_visible: false,
            stats: GameStats::default(),
            rng: SmallRng::from_entropy(),
            title_selected: 0,
            animation: None,
            ai_thinking: false,
            ai_spinner_frame: 0,
            ai_receiver: None,
            help_scroll: 0,
            pending_roll: false,
            roll_after: None,
            forfeit_after: None,
            rosette_reroll: false,
            last_opponent_roll: None,
        }
    }

    pub fn start_new_game(&mut self) {
        self.screen = Screen::DifficultySelect { selected: 0 };
    }

    pub fn confirm_difficulty(&mut self) {
        let selected = match self.screen {
            Screen::DifficultySelect { selected } => selected,
            _ => unreachable!("confirm_difficulty called from wrong screen"),
        };
        self.difficulty = DIFFICULTIES[selected].1;

        let p1_final = Dice::roll(&mut self.rng);
        let p2_final = Dice::roll(&mut self.rng);
        self.screen = Screen::DiceOff {
            state: DiceOffState {
                p1_frames: DICE_OFF_ANIMATION_FRAMES,
                p2_frames: DICE_OFF_ANIMATION_FRAMES,
                p1_final,
                p2_final,
                p1_display: Dice(0),
                p2_display: Dice(0),
                winner: None,
                acknowledged: false,
            },
        };
    }

    /// Starts the game with the given first player, initialising game state and
    /// transitioning to the `Game` screen.
    pub fn begin_game(&mut self, first_player: ur_core::player::Player) {
        let rules = ur_core::state::GameRules::finkel();
        let mut gs = ur_core::state::GameState::new(&rules);
        gs.current_player = first_player;
        self.game_state = Some(gs);
        self.stats = GameStats {
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
        self.screen = Screen::Game;

        if first_player == ur_core::player::Player::Player2 {
            self.start_ai_turn();
        } else {
            self.pending_roll = true;
            self.roll_after = Some(
                std::time::Instant::now() + std::time::Duration::from_millis(AUTO_ROLL_DELAY_MS),
            );
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Opens the pause menu (called when Esc is pressed during gameplay).
    pub fn open_pause(&mut self) {
        self.screen = Screen::PauseMenu { selected: 0 };
    }

    pub fn handle_confirm(&mut self) {
        match &self.screen {
            Screen::Title => match self.title_selected {
                0 => self.start_new_game(),
                1 => {
                    self.help_scroll = 0;
                    self.screen = Screen::Help { from_game: false };
                }
                _ => self.quit(),
            },
            Screen::DifficultySelect { .. } => self.confirm_difficulty(),
            Screen::DiceOff { state } => {
                if let Some(first_player) = state.winner {
                    if !state.acknowledged {
                        // mark as acknowledged to prevent double-trigger
                        if let Screen::DiceOff { state } = &mut self.screen {
                            state.acknowledged = true;
                        }
                        self.begin_game(first_player);
                    }
                }
            }
            Screen::PauseMenu { selected } => match *selected {
                0 => self.screen = Screen::Game,
                1 => {
                    self.help_scroll = 0;
                    self.screen = Screen::Help { from_game: true };
                }
                _ => self.quit(),
            },
            Screen::Help { from_game } => {
                if *from_game {
                    self.screen = Screen::PauseMenu { selected: 0 };
                } else {
                    self.screen = Screen::Title;
                }
            }
            _ => {}
        }
    }

    pub fn handle_back(&mut self) {
        match &self.screen {
            Screen::DifficultySelect { .. } => self.screen = Screen::Title,
            Screen::PauseMenu { .. } => self.screen = Screen::Game,
            Screen::Help { from_game } => {
                if *from_game {
                    self.screen = Screen::PauseMenu { selected: 0 };
                } else {
                    self.screen = Screen::Title;
                }
            }
            _ => {}
        }
    }

    /// Scroll the help screen up/down.
    pub fn help_scroll_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    pub fn help_scroll_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(1);
    }

    pub fn handle_menu_up(&mut self) {
        match &mut self.screen {
            Screen::Title => {
                if self.title_selected > 0 {
                    self.title_selected -= 1;
                }
            }
            Screen::DifficultySelect { selected } => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            Screen::PauseMenu { selected } => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn handle_menu_down(&mut self) {
        match &mut self.screen {
            Screen::Title => {
                if self.title_selected < 2 {
                    self.title_selected += 1;
                }
            }
            Screen::DifficultySelect { selected } => {
                if *selected < 2 {
                    *selected += 1;
                }
            }
            Screen::PauseMenu { selected } => {
                if *selected < 2 {
                    *selected += 1;
                }
            }
            _ => {}
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
        if gs.current_player != ur_core::player::Player::Player1 {
            return;
        }
        if !matches!(gs.phase, ur_core::state::GamePhase::WaitingForRoll) {
            return;
        }
        if self.animation.is_some() {
            return;
        }
        // dice_roll is set as soon as a roll starts and cleared only when the
        // move is applied (or the turn is forfeited).  Checking it here prevents
        // the player from re-rolling after the animation finishes but before they
        // have made a move — the game-state phase stays WaitingForRoll throughout.
        if self.dice_roll.is_some() {
            return;
        }

        let final_value = Dice::roll(&mut self.rng);
        self.animation = Some(Animation::DiceRoll {
            frames_remaining: DICE_ROLL_ANIMATION_FRAMES,
            final_value,
            display: Dice(0),
        });
        self.dice_roll = Some(final_value);
    }

    /// Total cursor positions: 0 = pool, 1..=14 = path steps.
    const PATH_CURSOR_COUNT: usize = 15;

    /// Moves the path cursor to the previous position (wrapping).
    pub fn handle_select_prev(&mut self) {
        if self.cursor_path_pos == 0 {
            self.cursor_path_pos = Self::PATH_CURSOR_COUNT - 1;
        } else {
            self.cursor_path_pos -= 1;
        }
    }

    /// Moves the path cursor to the next position (wrapping).
    pub fn handle_select_next(&mut self) {
        self.cursor_path_pos = (self.cursor_path_pos + 1) % Self::PATH_CURSOR_COUNT;
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

    /// Returns the legal move whose source matches the current cursor position.
    pub fn legal_move_at_cursor(&self) -> Option<&Move> {
        let gs = self.game_state.as_ref()?;
        let path = gs.rules.path_for(gs.current_player);
        let from_loc = if self.cursor_path_pos == 0 {
            PieceLocation::Unplayed
        } else {
            PieceLocation::OnBoard(path.get(self.cursor_path_pos - 1)?)
        };
        self.legal_moves.iter().find(|m| m.from == from_loc)
    }

    /// Snaps the cursor to the path position of the first legal move.
    fn snap_cursor_to_first_move(&mut self) {
        let first = match self.legal_moves.first() {
            Some(m) => m,
            None => {
                self.cursor_path_pos = 0;
                return;
            }
        };
        match &first.from {
            PieceLocation::Unplayed => {
                self.cursor_path_pos = 0;
            }
            PieceLocation::OnBoard(sq) => {
                if let Some(gs) = &self.game_state {
                    let path = gs.rules.path_for(gs.current_player);
                    self.cursor_path_pos = path.index_of(*sq).map(|i| i + 1).unwrap_or(0);
                }
            }
            _ => {
                self.cursor_path_pos = 0;
            }
        }
    }

    /// Applies a move to the current game state and handles turn transitions.
    pub fn apply_move(&mut self, mv: Move) {
        let gs = match self.game_state.take() {
            Some(gs) => gs,
            None => return,
        };
        let result = gs.apply_move(mv.clone());
        self.stats.moves += 1;

        let player_num = match mv.piece.player {
            ur_core::player::Player::Player1 => 1,
            ur_core::player::Player::Player2 => 2,
        };
        let piece_desc = format!("P{player_num}");
        match &mv.to {
            ur_core::state::PieceLocation::OnBoard(sq) => {
                if result.captured.is_some() {
                    self.stats.captures[player_num - 1] += 1;
                    self.log
                        .push(format!("{piece_desc} captured on ({},{})", sq.row, sq.col));
                } else if result.landed_on_rosette {
                    self.log.push(format!(
                        "{piece_desc} landed on rosette ({},{}) — extra turn!",
                        sq.row, sq.col
                    ));
                } else {
                    self.log
                        .push(format!("{piece_desc} moved to ({},{})", sq.row, sq.col));
                }
            }
            ur_core::state::PieceLocation::Scored => {
                self.log.push(format!("{piece_desc} scored a piece!"));
            }
            _ => {}
        }

        self.game_state = Some(result.new_state.clone());
        // Save AI's roll before clearing, so the AI panel can show it dimmed.
        if mv.piece.player == ur_core::player::Player::Player2 {
            self.last_opponent_roll = self.dice_roll;
        }
        self.dice_roll = None;
        self.legal_moves.clear();
        self.cursor_path_pos = 0;

        if result.game_over {
            self.screen = Screen::GameOver;
            return;
        }

        // Start cosmetic capture-flash animation when a capture occurred.
        if result.captured.is_some() {
            if let ur_core::state::PieceLocation::OnBoard(sq) = mv.to {
                self.animation = Some(Animation::CaptureFlash {
                    square: sq,
                    frames_remaining: 9,
                });
                // After the animation completes, on_animation_done will be called.
                // dice_roll is already None at this point, so it will be a no-op there.
                // We must still start the AI turn after the flash. We store the intent
                // to start the AI turn via the pending_ai_turn flag handled in
                // on_animation_done. For simplicity we start the AI turn immediately
                // even while the flash plays — the flash is purely visual.
            }
        }

        // Start cosmetic piece-move animation, showing a ghost stepping along the path.
        // Only start if no capture flash is already set (capture flash takes priority).
        if self.animation.is_none() {
            let path_squares = compute_move_path(&gs.rules, &mv);
            if path_squares.len() > 1 {
                let is_player1 = mv.piece.player == ur_core::player::Player::Player1;
                self.animation = Some(Animation::PieceMove {
                    remaining: path_squares,
                    frames_per_step: 3,
                    frames_this_step: 3,
                    is_player1,
                });
            }
        }

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
    }

    /// Called when the active animation completes.
    ///
    /// Computes legal moves from the rolled dice; forfeits the turn if none exist.
    pub fn on_animation_done(&mut self) {
        // Only process dice-roll completion. For other animation types (CaptureFlash,
        // PieceMove), dice_roll is None at this point so this is a no-op.
        if self.dice_roll.is_none() {
            return;
        }
        if let Some(roll) = self.dice_roll {
            if let Some(gs) = &self.game_state {
                let moves = gs.legal_moves(roll);
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
                    self.legal_moves = moves;
                    self.snap_cursor_to_first_move();
                }
            }
        }
    }

    /// Rolls dice for the AI and either forfeits the turn (if roll is 0 or no
    /// legal moves exist) or spawns a background thread to run
    /// [`ur_core::ai::choose_move`], sending the result via an mpsc channel.
    pub fn start_ai_turn(&mut self) {
        let gs = match self.game_state.as_ref() {
            Some(gs) => gs.clone(),
            None => return,
        };

        let roll = ur_core::dice::Dice::roll(&mut self.rng);
        self.log.push(format!("AI rolled {}", roll.value()));

        let moves = gs.legal_moves(roll);
        if moves.is_empty() {
            self.log.push("AI has no moves — passing turn".to_string());
            // Show the no-moves (red) state in the AI panel for FORFEIT_DISPLAY_MS.
            self.dice_roll = Some(roll); // panel needs the value to render red
            self.forfeit_after = Some(
                std::time::Instant::now() + std::time::Duration::from_millis(FORFEIT_DISPLAY_MS),
            );
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
        } else {
            // forfeit_turn returned None — restore state to prevent silent corruption
            self.game_state = Some(gs);
        }
    }

    /// Polls the AI move channel (non-blocking). If a result is ready, clears
    /// the thinking state and applies the move.
    pub fn poll_ai_move(&mut self) {
        let mv = match self.ai_receiver.as_ref() {
            Some(rx) => match rx.try_recv() {
                Ok(m) => m,
                Err(std::sync::mpsc::TryRecvError::Empty) => return,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // AI thread panicked — clear thinking state to prevent hang
                    self.ai_receiver = None;
                    self.ai_thinking = false;
                    self.log.push("AI error — turn skipped".to_string());
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the sequence of board squares a piece travels through when making `mv`.
///
/// Returns the intermediate squares (not including the source) up to and including
/// the destination. Returns an empty `Vec` if the move is not an on-board-to-on-board
/// or unplayed-to-on-board move (e.g. bear-off moves return empty).
///
/// Used to populate `Animation::PieceMove::remaining`.
fn compute_move_path(
    rules: &ur_core::state::GameRules,
    mv: &ur_core::state::Move,
) -> Vec<ur_core::board::Square> {
    let to_sq = match mv.to {
        ur_core::state::PieceLocation::OnBoard(sq) => sq,
        _ => return Vec::new(), // bear-off or other — no path to show
    };

    let path = rules.path_for(mv.piece.player);

    let dest_idx = match path.index_of(to_sq) {
        Some(i) => i,
        None => return Vec::new(),
    };

    let start_idx = match mv.from {
        ur_core::state::PieceLocation::OnBoard(from_sq) => {
            match path.index_of(from_sq) {
                Some(i) => i + 1, // start from the square after the current position
                None => return Vec::new(),
            }
        }
        ur_core::state::PieceLocation::Unplayed => 0, // entering piece: show from path[0]
        ur_core::state::PieceLocation::Scored => return Vec::new(),
    };

    // Collect all intermediate squares including destination
    (start_idx..=dest_idx).filter_map(|i| path.get(i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_app_starts_on_title_screen() {
        let app = App::new();
        assert!(matches!(app.screen, Screen::Title));
    }

    #[test]
    fn test_start_new_game_goes_to_difficulty() {
        let mut app = App::new();
        app.start_new_game();
        assert!(matches!(app.screen, Screen::DifficultySelect { .. }));
    }

    #[test]
    fn test_confirm_difficulty_goes_to_diceoff() {
        let mut app = App::new();
        app.start_new_game();
        app.confirm_difficulty();
        assert!(matches!(app.screen, Screen::DiceOff { .. }));
    }

    #[test]
    fn test_quit_sets_should_quit() {
        let mut app = App::new();
        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn test_roll_dice_starts_dice_animation() {
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(ur_core::state::GameState::new(
            &ur_core::state::GameRules::finkel(),
        ));
        app.handle_roll_dice();
        assert!(matches!(
            app.animation,
            Some(crate::animation::Animation::DiceRoll { .. })
        ));
    }

    #[test]
    fn test_roll_dice_ignored_when_dice_already_rolled() {
        // Regression: player must not be able to re-roll after the dice animation
        // finishes but before they have made a move.  At that point animation is
        // None and game-state phase is still WaitingForRoll, so without the
        // dice_roll guard handle_roll_dice would allow a second roll.
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(ur_core::state::GameState::new(
            &ur_core::state::GameRules::finkel(),
        ));
        // Simulate: roll has happened, animation finished, legal moves waiting.
        app.dice_roll = Some(ur_core::dice::Dice(3));
        app.animation = None;
        app.handle_roll_dice();
        // dice_roll should be unchanged — no new animation started.
        assert_eq!(app.dice_roll, Some(ur_core::dice::Dice(3)));
        assert!(app.animation.is_none());
    }

    #[test]
    fn test_roll_dice_ignored_when_animation_active() {
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(ur_core::state::GameState::new(
            &ur_core::state::GameRules::finkel(),
        ));
        app.animation = Some(crate::animation::Animation::Done);
        app.handle_roll_dice();
        // Still Done — the new dice animation was not started
        assert!(matches!(
            app.animation,
            Some(crate::animation::Animation::Done)
        ));
    }

    #[test]
    fn test_handle_back_from_difficulty_goes_to_title() {
        let mut app = App::new();
        app.screen = Screen::DifficultySelect { selected: 1 };
        app.handle_back();
        assert!(matches!(app.screen, Screen::Title));
    }

    #[test]
    fn test_handle_back_from_pause_goes_to_game() {
        let mut app = App::new();
        app.screen = Screen::PauseMenu { selected: 0 };
        app.handle_back();
        assert!(matches!(app.screen, Screen::Game));
    }

    #[test]
    fn test_menu_up_clamps_at_zero() {
        let mut app = App::new();
        app.screen = Screen::DifficultySelect { selected: 0 };
        app.handle_menu_up();
        assert!(matches!(
            app.screen,
            Screen::DifficultySelect { selected: 0 }
        ));
    }

    #[test]
    fn test_menu_down_clamps_at_max() {
        let mut app = App::new();
        app.screen = Screen::DifficultySelect { selected: 2 };
        app.handle_menu_down();
        assert!(matches!(
            app.screen,
            Screen::DifficultySelect { selected: 2 }
        ));
    }

    #[test]
    fn test_select_next_wraps_path_cursor() {
        let mut app = App::new();
        app.cursor_path_pos = App::PATH_CURSOR_COUNT - 1;
        app.handle_select_next();
        assert_eq!(app.cursor_path_pos, 0);
    }

    #[test]
    fn test_select_prev_wraps_path_cursor() {
        let mut app = App::new();
        app.cursor_path_pos = 0;
        app.handle_select_prev();
        assert_eq!(app.cursor_path_pos, App::PATH_CURSOR_COUNT - 1);
    }

    #[test]
    fn test_new_app_auto_roll_fields_initial_state() {
        let app = App::new();
        assert!(!app.pending_roll);
        assert!(app.roll_after.is_none());
        assert!(app.forfeit_after.is_none());
        assert!(!app.rosette_reroll);
        assert!(app.last_opponent_roll.is_none());
    }

    #[test]
    fn test_tick_auto_roll_fires_when_deadline_past() {
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(ur_core::state::GameState::new(
            &ur_core::state::GameRules::finkel(),
        ));
        app.pending_roll = true;
        // Deadline already in the past
        app.roll_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        app.tick_auto_roll();
        assert!(
            !app.pending_roll,
            "pending_roll should be cleared after firing"
        );
        assert!(app.dice_roll.is_some(), "dice_roll should be set");
        assert!(
            matches!(
                app.animation,
                Some(crate::animation::Animation::DiceRoll { .. })
            ),
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
        app.roll_after = Some(std::time::Instant::now() + std::time::Duration::from_secs(10));
        app.tick_auto_roll();
        assert!(
            app.pending_roll,
            "pending_roll should still be set before deadline"
        );
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
        app.roll_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        app.animation = Some(crate::animation::Animation::PieceMove {
            remaining: vec![],
            frames_per_step: 3,
            frames_this_step: 3,
            is_player1: true,
        });
        app.tick_auto_roll();
        assert!(
            app.pending_roll,
            "pending_roll should survive while animation runs"
        );
    }

    #[test]
    fn test_tick_auto_roll_skipped_on_ai_turn() {
        let mut app = App::new();
        app.screen = Screen::Game;
        let mut gs = ur_core::state::GameState::new(&ur_core::state::GameRules::finkel());
        gs.current_player = ur_core::player::Player::Player2;
        app.game_state = Some(gs);
        app.pending_roll = true;
        app.roll_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        app.tick_auto_roll();
        assert!(app.pending_roll, "auto-roll should not fire on AI's turn");
    }

    #[test]
    fn test_begin_game_sets_pending_roll_for_player1() {
        let mut app = App::new();
        app.screen = Screen::Game;
        app.begin_game(ur_core::player::Player::Player1);
        assert!(
            app.pending_roll,
            "pending_roll must be set when Player1 goes first"
        );
        assert!(
            app.roll_after.is_some(),
            "roll_after must be set for the 300ms delay"
        );
    }

    #[test]
    fn test_apply_move_sets_last_opponent_roll_from_ai_move() {
        use ur_core::{
            dice::Dice,
            player::Player,
            state::{GameRules, GameState},
        };
        let rules = GameRules::finkel();
        let mut app = App::new();
        app.screen = Screen::Game;
        // Player2 (AI) enters a piece from Unplayed with roll=1 → lands at path[0].
        let mut gs = GameState::new(&rules);
        gs.current_player = Player::Player2;
        app.game_state = Some(gs.clone());
        app.dice_roll = Some(Dice(1)); // pretend AI rolled 1
                                       // Use legal_moves to get a valid move for roll=1 (enter from Unplayed).
        let moves = gs.legal_moves(Dice(1));
        let mv = moves
            .into_iter()
            .next()
            .expect("should have at least one legal move");
        app.apply_move(mv);
        assert_eq!(
            app.last_opponent_roll,
            Some(Dice(1)),
            "last_opponent_roll must be saved before dice_roll is cleared"
        );
    }

    #[test]
    fn test_on_animation_done_sets_forfeit_after_on_no_moves() {
        let rules = ur_core::state::GameRules::finkel();
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(ur_core::state::GameState::new(&rules));
        // Roll 0 — guaranteed no moves.
        app.dice_roll = Some(ur_core::dice::Dice(0));
        app.on_animation_done();
        assert!(
            app.forfeit_after.is_some(),
            "forfeit_after must be set when there are no legal moves"
        );
        assert!(
            app.dice_roll.is_some(),
            "dice_roll must NOT be cleared yet — panel shows red state until forfeit fires"
        );
        assert!(
            app.legal_moves.is_empty(),
            "no legal moves should be populated"
        );
    }

    #[test]
    fn test_tick_forfeit_delay_advances_to_ai_when_deadline_past() {
        let rules = ur_core::state::GameRules::finkel();
        let mut app = App::new();
        app.screen = Screen::Game;
        app.game_state = Some(ur_core::state::GameState::new(&rules));
        app.dice_roll = Some(ur_core::dice::Dice(0));
        // Deadline already past
        app.forfeit_after = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        app.tick_forfeit_delay();
        assert!(
            app.forfeit_after.is_none(),
            "forfeit_after must be cleared after firing"
        );
        assert!(
            app.dice_roll.is_none(),
            "dice_roll must be cleared after forfeit"
        );
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
        app.forfeit_after = Some(std::time::Instant::now() + std::time::Duration::from_secs(10));
        app.tick_forfeit_delay();
        assert!(
            app.forfeit_after.is_some(),
            "forfeit_after must not fire before deadline"
        );
    }

    #[test]
    fn test_apply_move_sets_pending_roll_when_ai_move_returns_to_human() {
        use ur_core::{
            dice::Dice,
            player::Player,
            state::{GameRules, GameState},
        };
        let rules = GameRules::finkel();
        let mut app = App::new();
        app.screen = Screen::Game;
        // AI (Player2) enters a piece from Unplayed with roll=1.
        // path[0] for Player2 is (0,3) — not a rosette — so control returns to Player1.
        let mut gs = GameState::new(&rules);
        gs.current_player = Player::Player2;
        let path = rules.path_for(Player::Player2);
        let to_sq = path.get(0).unwrap(); // (0,3) — not a rosette
        assert!(
            !rules.board_shape.is_rosette(to_sq),
            "path[0] for Player2 must not be a rosette for this scenario"
        );
        let moves = gs.legal_moves(Dice(1));
        let mv = moves
            .into_iter()
            .next()
            .expect("should have at least one legal move");
        app.game_state = Some(gs);
        app.dice_roll = Some(Dice(1));
        app.apply_move(mv);
        // After AI's move on a non-rosette, Player1 (human) gets the turn.
        assert!(
            app.pending_roll,
            "pending_roll must be set after AI move returns control to human"
        );
        assert!(
            !app.rosette_reroll,
            "non-rosette landing should not set rosette_reroll"
        );
        assert!(
            app.roll_after.is_some(),
            "roll_after must be set for the 300ms delay"
        );
    }
}
