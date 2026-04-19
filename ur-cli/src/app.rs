use crate::animation::Animation;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;
use ur_core::{
    dice::Dice,
    player::Player,
    state::{GameState, Move},
};

/// A single entry in the game event log.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    /// Which player caused this event. `None` for system messages.
    pub player: Option<Player>,
    pub text: String,
}

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

/// Number of animation ticks for a dice roll animation (≈1 s at 30 fps).
const DICE_ROLL_ANIMATION_FRAMES: u32 = 30;

/// Delay in ms before the dice roll animation fires automatically.
const AUTO_ROLL_DELAY_MS: u64 = 300;
/// How long (ms) to display a no-moves result before forfeiting.
const FORFEIT_DISPLAY_MS: u64 = 1000;

/// Difficulty level maps to expectiminimax search depth.
pub const DIFFICULTIES: [(&str, u32); 3] = [("Easy", 2), ("Medium", 4), ("Hard", 6)];

/// Cursor position representing the off-board "bearing off" (scored) slot.
/// Positions 1-14 are on-board path steps; 0 is the pool; 15 is the scoring area.
pub const CURSOR_BEAR_OFF: usize = 15;

/// 2-D navigation grid: NAV_GRID[visual_row][col] → cursor_path_pos.
/// Visual row 0 = top of board (finish end); row 7 = bottom (entry end).
/// Col 0 = left (player private lane + virtual off-board slots).
/// Col 1 = right (shared middle column).
///
/// Board layout after vertical flip:
///   row 0: [step 13, step 12]
///   row 1: [step 14✦, step 11]
///   row 2: [BEAR_OFF, step 10]   ← H-gap: virtual slots on left
///   row 3: [pool (0), step 9]    ← H-gap
///   row 4: [step 1, step 8✦]
///   row 5: [step 2, step 7]
///   row 6: [step 3, step 6]
///   row 7: [step 4✦, step 5]
const NAV_GRID: [[usize; 2]; 8] = [
    [13, 12],
    [14, 11],
    [CURSOR_BEAR_OFF, 10],
    [0, 9],
    [1, 8],
    [2, 7],
    [3, 6],
    [4, 5],
];

/// Direction for 2-D cursor navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavDir {
    Up,
    Down,
    Left,
    Right,
}

/// Returns the (row, col) grid coordinates for a cursor position, or `None`
/// if `pos` is not represented in the grid.
fn cursor_to_grid(pos: usize) -> Option<(usize, usize)> {
    for (row, row_vals) in NAV_GRID.iter().enumerate() {
        for (col, &cell) in row_vals.iter().enumerate() {
            if cell == pos {
                return Some((row, col));
            }
        }
    }
    None
}

/// Returns the cursor position reached by moving one step in `dir` from `pos`.
/// UP/DOWN wrap across columns at the board edges.
pub fn nav_cursor(pos: usize, dir: NavDir) -> usize {
    let (row, col) = match cursor_to_grid(pos) {
        Some(p) => p,
        None => return pos,
    };
    let (new_row, new_col) = match dir {
        NavDir::Left => (row, col.saturating_sub(1)),
        NavDir::Right => (row, (col + 1).min(1)),
        NavDir::Up => {
            if row > 0 {
                (row - 1, col)
            } else {
                (0, 1 - col) // wrap: swap column at top row
            }
        }
        NavDir::Down => {
            if row < 7 {
                (row + 1, col)
            } else {
                (7, 1 - col) // wrap: swap column at bottom row
            }
        }
    };
    NAV_GRID[new_row][new_col]
}

/// Game statistics accumulated during play.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct GameStats {
    pub moves: u32,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
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
    /// Cursor position along the current player's path.
    /// 0 = unplayed-pieces pool; k = path square at index k-1 (1-based).
    pub cursor_path_pos: usize,
    pub log: Vec<LogEntry>,
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
    /// Last dice roll per player (index 0 = P1, 1 = P2). Kept for the inactive panel.
    pub last_roll: [Option<Dice>; 2],
    /// Last notable event per player (capture / rosette / score). Shown below dice.
    pub last_event: [Option<String>; 2],
    /// Per-player turn history: up to 3 past turns, each a list of event strings.
    /// The last element is the current (in-progress) turn. Rosette re-rolls extend
    /// the current turn rather than starting a new one.
    pub turn_log: [Vec<Vec<String>>; 2],
    /// Monotonically increasing counter, incremented every animation tick. Used to
    /// drive UI animations (e.g. rolling dice pattern) independently of game state.
    pub frame_count: u32,
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
            last_roll: [None, None],
            last_event: [None, None],
            turn_log: [vec![vec![]], vec![vec![]]],
            frame_count: 0,
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
        self.last_roll = [None, None];
        self.last_event = [None, None];
        self.turn_log = [vec![vec![]], vec![vec![]]];
        self.rosette_reroll = false;
        self.forfeit_after = None;
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

    /// Opens the pause menu (only when a game is active).
    pub fn open_pause(&mut self) {
        if self.game_state.is_some() {
            self.screen = Screen::PauseMenu { selected: 0 };
        }
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
            Screen::DiceOff { .. } => {
                self.screen = Screen::DifficultySelect { selected: 0 };
            }
            Screen::PauseMenu { .. } => self.screen = Screen::Game,
            Screen::Help { from_game } => {
                if *from_game {
                    self.screen = Screen::Game;
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

    /// Moves the cursor one step backward along the path (toward the pool).
    /// Only active when legal moves are available (i.e. human player's turn).
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

        let player_num = match mv.piece.player {
            Player::Player1 => 1,
            Player::Player2 => 2,
        };
        let player_idx = mv.piece.player.index();
        let panel_event = match &mv.to {
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
                if result.captured.is_some() {
                    self.stats.captures[player_num - 1] += 1;
                    self.log.push(LogEntry {
                        player: Some(current_player),
                        text: format!("Captured piece at {}!", step),
                    });
                    let t = format!("Captured piece at {}!", step);
                    if let Some(cur) = self.turn_log[player_idx].last_mut() {
                        cur.push(t.clone());
                    }
                    Some(t)
                } else if result.landed_on_rosette {
                    self.log.push(LogEntry {
                        player: Some(current_player),
                        text: format!("Moved to rosette at {}", step),
                    });
                    self.log.push(LogEntry {
                        player: Some(current_player),
                        text: "Extra turn!".to_string(),
                    });
                    let t = format!("Moved to rosette at {}", step);
                    if let Some(cur) = self.turn_log[player_idx].last_mut() {
                        cur.push(t.clone());
                        cur.push("Extra turn!".to_string());
                    }
                    Some(t)
                } else {
                    self.log.push(LogEntry {
                        player: Some(current_player),
                        text: format!("Moved to {}", step),
                    });
                    let t = format!("Moved to {}", step);
                    if let Some(cur) = self.turn_log[player_idx].last_mut() {
                        cur.push(t.clone());
                    }
                    Some(t)
                }
            }
            ur_core::state::PieceLocation::Scored => {
                self.log.push(LogEntry {
                    player: Some(current_player),
                    text: "Scored!".to_string(),
                });
                let t = "Scored!".to_string();
                if let Some(cur) = self.turn_log[player_idx].last_mut() {
                    cur.push(t.clone());
                }
                Some(t)
            }
            _ => None,
        };
        self.last_event[player_idx] = panel_event;

        self.game_state = Some(result.new_state.clone());
        // Save each player's roll before clearing so inactive panels can show it.
        self.last_roll[player_idx] = self.dice_roll;
        self.dice_roll = None;
        self.legal_moves.clear();

        if result.game_over {
            self.stats.end_time = Some(std::time::Instant::now());
            self.screen = Screen::GameOver;
            return;
        }

        // Start cosmetic capture-flash animation when a capture occurred.
        if result.captured.is_some() {
            if let ur_core::state::PieceLocation::OnBoard(sq) = mv.to {
                self.animation = Some(Animation::CaptureFlash {
                    square: sq,
                    frames_remaining: 18, // ≈0.6 s
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
                    frames_per_step: 7, // ≈0.23 s per step at 30 fps
                    frames_this_step: 7,
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
        // Push "Rolled N" to the turn log now that the animation is complete.
        if let (Some(roll), Some(gs)) = (self.dice_roll, self.game_state.as_ref()) {
            let idx = gs.current_player.index();
            if let Some(current) = self.turn_log[idx].last_mut() {
                current.push(format!("Rolled {}", roll.value()));
            }
        }
        // If the AI is still computing, the dice-roll animation has finished but the
        // chosen move isn't ready yet. Let poll_ai_move handle the transition.
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
                    // Show the no-moves (red) state for FORFEIT_DISPLAY_MS before
                    // auto-advancing. dice_roll is kept so the panel renders red.
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

    /// Rolls dice for the AI and either forfeits the turn (if roll is 0 or no
    /// legal moves exist) or spawns a background thread to run
    /// [`ur_core::ai::choose_move`], sending the result via an mpsc channel.
    pub fn start_ai_turn(&mut self) {
        let gs = match self.game_state.as_ref() {
            Some(gs) => gs.clone(),
            None => return,
        };

        let roll = ur_core::dice::Dice::roll(&mut self.rng);
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

        // Play the same dice-roll animation as the human so the AI turn is
        // visually consistent. The AI computation thread (if any) starts in
        // parallel and poll_ai_move will wait for the animation to finish.
        self.animation = Some(Animation::DiceRoll {
            frames_remaining: DICE_ROLL_ANIMATION_FRAMES,
            final_value: roll,
            display: ur_core::dice::Dice(0),
        });

        let moves = gs.legal_moves(roll);
        if moves.is_empty() {
            // No thread needed — on_animation_done will detect empty moves and
            // log "No moves!" + set forfeit_after once the dice animation finishes.
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
        let _is_rosette_reroll = self.rosette_reroll;
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
        // Save the forfeited roll before clearing so the inactive panel can still show it.
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
        // Wait for the dice-roll animation to finish before applying the move so
        // the player sees the full roll before pieces start moving.
        if matches!(self.animation, Some(Animation::DiceRoll { .. })) {
            return;
        }
        let mv = match self.ai_receiver.as_ref() {
            Some(rx) => match rx.try_recv() {
                Ok(m) => m,
                Err(std::sync::mpsc::TryRecvError::Empty) => return,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // AI thread panicked — clear thinking state to prevent hang
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
mod log_tests {
    use super::*;
    use ur_core::{
        player::Player,
        state::{GameRules, GameState},
    };

    fn make_app_with_game() -> App {
        let mut app = App::new();
        let rules = GameRules::finkel();
        app.game_state = Some(GameState::new(&rules));
        app
    }

    #[test]
    fn test_log_entry_uses_you_label_for_player1() {
        let entry = LogEntry {
            player: Some(Player::Player1),
            text: "rolled 3".to_string(),
        };
        assert_eq!(entry.player, Some(Player::Player1));
        assert!(entry.text.contains("rolled 3"));
    }

    #[test]
    fn test_log_entry_uses_ai_label_for_player2() {
        let entry = LogEntry {
            player: Some(Player::Player2),
            text: "rolled 2".to_string(),
        };
        assert_eq!(entry.player, Some(Player::Player2));
    }

    #[test]
    fn test_ai_roll_pushes_log_entry_with_player2() {
        let mut app = make_app_with_game();
        // Force Player2's turn
        if let Some(gs) = &mut app.game_state {
            gs.current_player = Player::Player2;
        }
        app.start_ai_turn();
        let last = app.log.last().unwrap();
        assert_eq!(last.player, Some(Player::Player2));
        assert!(
            last.text.contains("Rolled"),
            "expected 'Rolled' in: {}",
            last.text
        );
    }

    #[test]
    fn test_human_auto_roll_pushes_log_entry_with_player1() {
        let mut app = make_app_with_game();
        app.pending_roll = true;
        app.roll_after = None; // fire immediately
        app.tick_auto_roll();
        let has_p1_roll = app
            .log
            .iter()
            .any(|e| e.player == Some(Player::Player1) && e.text.contains("Rolled"));
        assert!(has_p1_roll, "expected a Player1 Rolled entry");
    }
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

    // ── nav_cursor grid navigation tests ──────────────────────────────────────

    // Helper: make an App with legal moves so handle_nav is not a no-op.
    fn app_with_moves() -> App {
        let mut app = App::new();
        let gs = ur_core::state::GameState::new(&ur_core::state::GameRules::finkel());
        app.legal_moves = gs.legal_moves(ur_core::dice::Dice(1));
        app.game_state = Some(gs);
        app
    }

    // User-specified cases ────────────────────────────────────────────────────

    #[test]
    fn test_nav_pos1_right_goes_to_pos8() {
        assert_eq!(nav_cursor(1, NavDir::Right), 8);
    }

    #[test]
    fn test_nav_pos9_left_goes_to_pool() {
        assert_eq!(nav_cursor(9, NavDir::Left), 0);
    }

    #[test]
    fn test_nav_pos10_left_goes_to_bear_off() {
        assert_eq!(nav_cursor(10, NavDir::Left), CURSOR_BEAR_OFF);
    }

    #[test]
    fn test_nav_pos4_right_goes_to_pos5() {
        assert_eq!(nav_cursor(4, NavDir::Right), 5);
    }

    #[test]
    fn test_nav_pos4_down_goes_to_pos5() {
        assert_eq!(nav_cursor(4, NavDir::Down), 5);
    }

    #[test]
    fn test_nav_pos12_left_goes_to_pos13() {
        assert_eq!(nav_cursor(12, NavDir::Left), 13);
    }

    #[test]
    fn test_nav_pos12_up_goes_to_pos13() {
        assert_eq!(nav_cursor(12, NavDir::Up), 13);
    }

    #[test]
    fn test_nav_pool_right_goes_to_pos9() {
        assert_eq!(nav_cursor(0, NavDir::Right), 9);
    }

    #[test]
    fn test_nav_bear_off_right_goes_to_pos10() {
        assert_eq!(nav_cursor(CURSOR_BEAR_OFF, NavDir::Right), 10);
    }

    #[test]
    fn test_nav_bear_off_down_goes_to_pool() {
        assert_eq!(nav_cursor(CURSOR_BEAR_OFF, NavDir::Down), 0);
    }

    #[test]
    fn test_nav_pool_up_goes_to_bear_off() {
        assert_eq!(nav_cursor(0, NavDir::Up), CURSOR_BEAR_OFF);
    }

    // Horizontal (left/right) navigation ─────────────────────────────────────

    #[test]
    fn test_nav_horizontal_pairs() {
        // Each pair: left-col pos ↔ right-col pos at same visual row.
        let pairs = [
            (13usize, 12usize),    // row 0
            (14, 11),              // row 1
            (CURSOR_BEAR_OFF, 10), // row 2
            (0, 9),                // row 3
            (1, 8),                // row 4
            (2, 7),                // row 5
            (3, 6),                // row 6
            (4, 5),                // row 7
        ];
        for (left, right) in pairs {
            assert_eq!(nav_cursor(left, NavDir::Right), right, "{left} right");
            assert_eq!(nav_cursor(right, NavDir::Left), left, "{right} left");
        }
    }

    #[test]
    fn test_nav_left_from_leftmost_stays() {
        for pos in [13, 14, CURSOR_BEAR_OFF, 0, 1, 2, 3, 4] {
            assert_eq!(
                nav_cursor(pos, NavDir::Left),
                pos,
                "pos {pos} already leftmost"
            );
        }
    }

    #[test]
    fn test_nav_right_from_rightmost_stays() {
        for pos in [12, 11, 10, 9, 8, 7, 6, 5] {
            assert_eq!(
                nav_cursor(pos, NavDir::Right),
                pos,
                "pos {pos} already rightmost"
            );
        }
    }

    // Vertical (up/down) navigation within columns ────────────────────────────

    #[test]
    fn test_nav_up_left_column() {
        // Left col (col 0): rows 0-7 → positions [13,14,BEAR,0,1,2,3,4]
        let left_col = [13, 14, CURSOR_BEAR_OFF, 0, 1, 2, 3, 4];
        for i in 1..8 {
            assert_eq!(
                nav_cursor(left_col[i], NavDir::Up),
                left_col[i - 1],
                "up from left_col[{i}]={}",
                left_col[i]
            );
        }
    }

    #[test]
    fn test_nav_down_left_column() {
        let left_col = [13, 14, CURSOR_BEAR_OFF, 0, 1, 2, 3, 4];
        for i in 0..7 {
            assert_eq!(
                nav_cursor(left_col[i], NavDir::Down),
                left_col[i + 1],
                "down from left_col[{i}]={}",
                left_col[i]
            );
        }
    }

    #[test]
    fn test_nav_up_right_column() {
        // Right col (col 1): rows 0-7 → positions [12,11,10,9,8,7,6,5]
        let right_col = [12, 11, 10, 9, 8, 7, 6, 5];
        for i in 1..8 {
            assert_eq!(
                nav_cursor(right_col[i], NavDir::Up),
                right_col[i - 1],
                "up from right_col[{i}]={}",
                right_col[i]
            );
        }
    }

    #[test]
    fn test_nav_down_right_column() {
        let right_col = [12, 11, 10, 9, 8, 7, 6, 5];
        for i in 0..7 {
            assert_eq!(
                nav_cursor(right_col[i], NavDir::Down),
                right_col[i + 1],
                "down from right_col[{i}]={}",
                right_col[i]
            );
        }
    }

    // Edge wraps ──────────────────────────────────────────────────────────────

    #[test]
    fn test_nav_up_wraps_at_top_row() {
        // Top row (row 0): up wraps to other column at row 0.
        assert_eq!(nav_cursor(13, NavDir::Up), 12, "step 13 up → step 12");
        assert_eq!(nav_cursor(12, NavDir::Up), 13, "step 12 up → step 13");
    }

    #[test]
    fn test_nav_down_wraps_at_bottom_row() {
        // Bottom row (row 7): down wraps to other column at row 7.
        assert_eq!(nav_cursor(4, NavDir::Down), 5, "step 4 down → step 5");
        assert_eq!(nav_cursor(5, NavDir::Down), 4, "step 5 down → step 4");
    }

    // Guard: no movement when no legal moves ──────────────────────────────────

    #[test]
    fn test_handle_nav_no_op_when_no_legal_moves() {
        let mut app = App::new();
        app.cursor_path_pos = 3;
        app.handle_nav(NavDir::Right); // legal_moves empty → no change
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
            app.last_roll[1],
            Some(Dice(1)),
            "last_roll[P2] must be saved before dice_roll is cleared"
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
        // Player1 forfeited → Player2 (AI) should now be active → ai_thinking true.
        // Note: start_ai_turn() sets dice_roll to the AI's roll immediately, so
        // dice_roll is Some here (the AI's roll), not None.
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
