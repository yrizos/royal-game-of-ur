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
    DifficultySelect { selected: usize },
    DiceOff { state: DiceOffState },
    Game,
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

    pub fn quit(&mut self) {
        self.should_quit = true;
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
            Screen::DiceOff { .. } => { /* handled in animation::tick */ }
            _ => {}
        }
    }

    pub fn handle_back(&mut self) {
        if let Screen::DifficultySelect { .. } = &self.screen {
            self.screen = Screen::Title;
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

    /// Stub for starting the AI turn (implemented in Task 12).
    #[allow(dead_code)]
    pub fn start_ai_turn(&mut self) {
        // Implemented in Task 12
    }

    #[allow(dead_code)]
    pub fn poll_ai_move(&mut self) { /* Task 12 */
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
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
