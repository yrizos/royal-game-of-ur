use crate::animation::Animation;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;
use ur_core::{
    dice::Dice,
    state::{GameState, Move},
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
    Help,
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
    pub selected_move_idx: usize,
    pub log: Vec<String>,
    pub log_visible: bool,
    pub stats: GameStats,
    pub rng: SmallRng,
    pub title_selected: usize,
    pub animation: Option<Animation>,
    pub ai_thinking: bool,
    pub ai_spinner_frame: u32,
    pub ai_receiver: Option<std::sync::mpsc::Receiver<ur_core::state::Move>>,
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
            selected_move_idx: 0,
            log: Vec::new(),
            log_visible: false,
            stats: GameStats::default(),
            rng: SmallRng::from_entropy(),
            title_selected: 0,
            animation: None,
            ai_thinking: false,
            ai_spinner_frame: 0,
            ai_receiver: None,
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
        self.selected_move_idx = 0;
        self.animation = None;
        self.ai_thinking = false;
        self.ai_receiver = None;
        self.ai_spinner_frame = 0;
        self.screen = Screen::Game;

        if first_player == ur_core::player::Player::Player2 {
            self.start_ai_turn();
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
            Screen::Title => {
                if self.title_selected == 0 {
                    self.start_new_game();
                } else {
                    self.quit();
                }
            }
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
                1 => self.screen = Screen::Help,
                _ => self.quit(),
            },
            Screen::Help => self.screen = Screen::PauseMenu { selected: 0 },
            _ => {}
        }
    }

    pub fn handle_back(&mut self) {
        match &self.screen {
            Screen::DifficultySelect { .. } => self.screen = Screen::Title,
            Screen::PauseMenu { .. } => self.screen = Screen::Game,
            Screen::Help => self.screen = Screen::PauseMenu { selected: 0 },
            _ => {}
        }
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
                if self.title_selected < 1 {
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

    /// Handles the player pressing the roll-dice key.
    ///
    /// Only allowed during Player1's turn, in `WaitingForRoll` phase, when no
    /// animation is currently running.
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

        let final_value = Dice::roll(&mut self.rng);
        self.animation = Some(Animation::DiceRoll {
            frames_remaining: DICE_ROLL_ANIMATION_FRAMES,
            final_value,
            display: Dice(0),
        });
        self.dice_roll = Some(final_value);
    }

    /// Cycles the move selection to the previous legal move.
    pub fn handle_select_prev(&mut self) {
        if self.legal_moves.is_empty() {
            return;
        }
        if self.selected_move_idx == 0 {
            self.selected_move_idx = self.legal_moves.len() - 1;
        } else {
            self.selected_move_idx -= 1;
        }
    }

    /// Cycles the move selection to the next legal move.
    pub fn handle_select_next(&mut self) {
        if self.legal_moves.is_empty() {
            return;
        }
        self.selected_move_idx = (self.selected_move_idx + 1) % self.legal_moves.len();
    }

    /// Confirms the currently selected move and applies it.
    pub fn handle_confirm_move(&mut self) {
        if self.animation.is_some() {
            return;
        }
        let mv = match self.legal_moves.get(self.selected_move_idx) {
            Some(m) => m.clone(),
            None => return,
        };
        self.apply_move(mv);
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
        self.dice_roll = None;
        self.legal_moves.clear();
        self.selected_move_idx = 0;

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
                    self.legal_moves = moves;
                    self.selected_move_idx = 0;
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

        let depth = self.difficulty;
        let (tx, rx) = std::sync::mpsc::channel();
        self.ai_receiver = Some(rx);
        self.ai_thinking = true;
        self.dice_roll = Some(roll);

        std::thread::spawn(move || {
            let chosen = ur_core::ai::choose_move(&gs, roll, depth);
            let _ = tx.send(chosen);
        });
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
    fn test_select_next_wraps_within_legal_moves() {
        let mut app = App::new();
        app.legal_moves = vec![
            ur_core::state::Move {
                piece: ur_core::player::Piece::new(ur_core::player::Player::Player1, 0),
                from: ur_core::state::PieceLocation::Unplayed,
                to: ur_core::state::PieceLocation::OnBoard(ur_core::board::Square::new(2, 3)),
            },
            ur_core::state::Move {
                piece: ur_core::player::Piece::new(ur_core::player::Player::Player1, 1),
                from: ur_core::state::PieceLocation::Unplayed,
                to: ur_core::state::PieceLocation::OnBoard(ur_core::board::Square::new(2, 3)),
            },
        ];
        app.selected_move_idx = 1;
        app.handle_select_next();
        assert_eq!(app.selected_move_idx, 0); // wraps to 0
    }
}
