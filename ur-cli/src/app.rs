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

    #[allow(dead_code)]
    pub fn handle_roll_dice(&mut self) { /* Task 11 */
    }
    #[allow(dead_code)]
    pub fn handle_select_prev(&mut self) { /* Task 11 */
    }
    #[allow(dead_code)]
    pub fn handle_select_next(&mut self) { /* Task 11 */
    }
    #[allow(dead_code)]
    pub fn handle_confirm_move(&mut self) { /* Task 11 */
    }
    #[allow(dead_code)]
    pub fn on_animation_done(&mut self) { /* Task 11 */
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
}
