pub mod game;
pub mod gameover;
pub mod menu;
pub mod pause;
pub mod title;

use crate::app::App;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    match &app.screen {
        crate::app::Screen::Title => title::render(f, app.title_selected),
        crate::app::Screen::DifficultySelect { selected } => menu::render_difficulty(f, *selected),
        crate::app::Screen::DiceOff { state } => menu::render_dice_off(f, state),
        crate::app::Screen::Game => game::render_game(f, app),
        crate::app::Screen::PauseMenu { selected } => {
            // Render the game underneath, then overlay the pause menu.
            game::render_game(f, app);
            pause::render_pause_menu(f, *selected);
        }
        crate::app::Screen::Help => {
            // Render the game underneath, then overlay the help panel.
            game::render_game(f, app);
            pause::render_help(f);
        }
        crate::app::Screen::GameOver => {
            use crate::ui::gameover::{render, GameOverData};
            use ur_core::player::Player;
            let winner_is_human = match &app.game_state {
                Some(gs) => matches!(
                    &gs.phase,
                    ur_core::state::GamePhase::GameOver(p) if *p == Player::Player1
                ),
                None => false,
            };
            let elapsed = app
                .stats
                .start_time
                .map(|t| t.elapsed())
                .unwrap_or(std::time::Duration::ZERO);
            render(
                f,
                &GameOverData {
                    winner_is_human,
                    moves: app.stats.moves,
                    elapsed,
                    captures: &app.stats.captures,
                },
            );
        }
    }
}

pub fn render_too_small(f: &mut Frame) {
    use ratatui::widgets::Paragraph;
    let msg = Paragraph::new("Terminal too small — please resize to at least 80×24");
    f.render_widget(msg, f.size());
}
