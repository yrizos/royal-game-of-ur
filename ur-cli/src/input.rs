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
    RollDice,
    SelectPrev,
    SelectNext,
    ConfirmMove,
    ToggleLog,
    QuitPrompt,
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
            KeyCode::Char(' ') => Some(Action::RollDice),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::SelectPrev),
            KeyCode::Right => Some(Action::SelectNext),
            // Note: `l` is ToggleLog per spec, not vi-style SelectNext.
            // Use Right arrow for SelectNext instead.
            KeyCode::Char('l') => Some(Action::ToggleLog),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::SelectPrev),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::SelectNext),
            KeyCode::Enter => Some(Action::ConfirmMove),
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
        Screen::Help => match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('q') => {
                Some(Action::Back)
            }
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
    fn test_space_maps_to_roll_dice_in_game() {
        let action = map_key(key(KeyCode::Char(' ')), &crate::app::Screen::Game);
        assert_eq!(action, Some(Action::RollDice));
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
}
