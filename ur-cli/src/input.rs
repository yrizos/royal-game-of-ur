use crate::app::Screen;
use crossterm::event::{KeyCode, KeyEvent};

/// High-level actions derived from raw key events.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    // Navigation (menus and title)
    MenuUp,
    MenuDown,
    Confirm,
    Back,
    // Gameplay
    SelectPrev,
    SelectNext,
    ConfirmMove,
    ToggleLog,
    QuitPrompt,
    // Scrollable overlays
    ScrollUp,
    ScrollDown,
    // Game over
    NewGame,
    Quit,
}

/// Maps a raw key event to an `Action` given the current screen.
/// Returns `None` for keys that have no binding in this context.
pub fn map_key(key: KeyEvent, screen: &Screen) -> Option<Action> {
    match screen {
        Screen::Title => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter | KeyCode::Char(' ') => Some(Action::Confirm),
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
            _ => None,
        },
        Screen::DifficultySelect { .. } => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::Confirm),
            KeyCode::Esc => Some(Action::Back),
            _ => None,
        },
        Screen::DiceOff { .. } => match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => Some(Action::Confirm),
            KeyCode::Esc => Some(Action::QuitPrompt),
            _ => None,
        },
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
        Screen::PauseMenu { .. } => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter | KeyCode::Char(' ') => Some(Action::Confirm),
            KeyCode::Esc | KeyCode::Char('r') => Some(Action::Back),
            _ => None,
        },
        Screen::Help { .. } => match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('q') => {
                Some(Action::Back)
            }
            KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollDown),
            _ => None,
        },
        Screen::GameOver => match key.code {
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Enter => Some(Action::NewGame),
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Action::Quit),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_space_in_game_returns_none_after_roll_removed() {
        let action = map_key(key(KeyCode::Char(' ')), &crate::app::Screen::Game);
        assert_eq!(action, None);
    }

    #[test]
    fn test_enter_maps_to_confirm_on_title() {
        let action = map_key(key(KeyCode::Enter), &crate::app::Screen::Title);
        assert_eq!(action, Some(Action::Confirm));
    }

    #[test]
    fn test_escape_maps_to_back_on_difficulty() {
        let action = map_key(
            key(KeyCode::Esc),
            &crate::app::Screen::DifficultySelect { selected: 0 },
        );
        assert_eq!(action, Some(Action::Back));
    }

    #[test]
    fn test_l_toggles_log_in_game() {
        let action = map_key(key(KeyCode::Char('l')), &crate::app::Screen::Game);
        assert_eq!(action, Some(Action::ToggleLog));
    }

    #[test]
    fn test_n_starts_new_game_on_gameover() {
        let action = map_key(key(KeyCode::Char('n')), &crate::app::Screen::GameOver);
        assert_eq!(action, Some(Action::NewGame));
    }

    #[test]
    fn test_unrecognised_key_returns_none() {
        let action = map_key(key(KeyCode::Char('z')), &crate::app::Screen::Game);
        assert_eq!(action, None);
    }

    #[test]
    fn test_escape_maps_to_quit_prompt_in_game() {
        let action = map_key(key(KeyCode::Esc), &crate::app::Screen::Game);
        assert_eq!(action, Some(Action::QuitPrompt));
    }

    #[test]
    fn test_j_maps_to_select_next_in_game() {
        let action = map_key(key(KeyCode::Char('j')), &crate::app::Screen::Game);
        assert_eq!(action, Some(Action::SelectNext));
    }

    #[test]
    fn test_right_maps_to_select_next_in_game() {
        let action = map_key(key(KeyCode::Right), &crate::app::Screen::Game);
        assert_eq!(action, Some(Action::SelectNext));
    }

    #[test]
    fn test_k_maps_to_select_prev_in_game() {
        let action = map_key(key(KeyCode::Char('k')), &crate::app::Screen::Game);
        assert_eq!(action, Some(Action::SelectPrev));
    }

    #[test]
    fn test_h_maps_to_select_prev_in_game() {
        let action = map_key(key(KeyCode::Char('h')), &crate::app::Screen::Game);
        assert_eq!(action, Some(Action::SelectPrev));
    }

    #[test]
    fn test_j_maps_to_menu_down_on_difficulty() {
        let action = map_key(
            key(KeyCode::Char('j')),
            &crate::app::Screen::DifficultySelect { selected: 0 },
        );
        assert_eq!(action, Some(Action::MenuDown));
    }

    #[test]
    fn test_k_maps_to_menu_up_on_difficulty() {
        let action = map_key(
            key(KeyCode::Char('k')),
            &crate::app::Screen::DifficultySelect { selected: 1 },
        );
        assert_eq!(action, Some(Action::MenuUp));
    }

    #[test]
    fn test_esc_maps_to_back_on_help() {
        let action = map_key(
            key(KeyCode::Esc),
            &crate::app::Screen::Help { from_game: false },
        );
        assert_eq!(action, Some(Action::Back));
    }

    #[test]
    fn test_up_down_scroll_on_help() {
        let screen = crate::app::Screen::Help { from_game: true };
        assert_eq!(map_key(key(KeyCode::Up), &screen), Some(Action::ScrollUp));
        assert_eq!(
            map_key(key(KeyCode::Down), &screen),
            Some(Action::ScrollDown)
        );
    }

    #[test]
    fn test_uppercase_q_quits_on_gameover() {
        let action = map_key(key(KeyCode::Char('Q')), &crate::app::Screen::GameOver);
        assert_eq!(action, Some(Action::Quit));
    }

    #[test]
    fn test_uppercase_n_new_game_on_gameover() {
        let action = map_key(key(KeyCode::Char('N')), &crate::app::Screen::GameOver);
        assert_eq!(action, Some(Action::NewGame));
    }

    #[test]
    fn test_q_quits_on_title() {
        let action = map_key(key(KeyCode::Char('q')), &crate::app::Screen::Title);
        assert_eq!(action, Some(Action::Quit));
    }

    #[test]
    fn test_space_confirms_on_diceoff() {
        let action = map_key(
            key(KeyCode::Char(' ')),
            &crate::app::Screen::DiceOff {
                state: crate::app::DiceOffState {
                    p1_frames: 0,
                    p2_frames: 0,
                    p1_final: ur_core::dice::Dice(2),
                    p2_final: ur_core::dice::Dice(1),
                    p1_display: ur_core::dice::Dice(0),
                    p2_display: ur_core::dice::Dice(0),
                    winner: None,
                    acknowledged: false,
                },
            },
        );
        assert_eq!(action, Some(Action::Confirm));
    }
}
