use super::{App, DiceOffState, Screen, DICE_OFF_ANIMATION_FRAMES, DIFFICULTIES};
use ur_core::dice::Dice;

impl App {
    pub fn start_new_game(&mut self) {
        self.screen = Screen::DifficultySelect { selected: 0 };
    }

    pub fn confirm_difficulty(&mut self) {
        let selected = match self.screen {
            Screen::DifficultySelect { selected } => selected,
            _ => return,
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
                p1_display: Dice::new(0).unwrap(),
                p2_display: Dice::new(0).unwrap(),
                winner: None,
                acknowledged: false,
            },
        };
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

    pub fn help_scroll_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    pub fn help_scroll_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(1);
    }

    fn menu_selected_mut(&mut self) -> Option<(&mut usize, usize)> {
        match &mut self.screen {
            Screen::Title => Some((&mut self.title_selected, 2)),
            Screen::DifficultySelect { selected } => Some((selected, 2)),
            Screen::PauseMenu { selected } => Some((selected, 2)),
            _ => None,
        }
    }

    pub fn handle_menu_up(&mut self) {
        if let Some((sel, _)) = self.menu_selected_mut() {
            *sel = sel.saturating_sub(1);
        }
    }

    pub fn handle_menu_down(&mut self) {
        if let Some((sel, max)) = self.menu_selected_mut() {
            if *sel < max {
                *sel += 1;
            }
        }
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
}
