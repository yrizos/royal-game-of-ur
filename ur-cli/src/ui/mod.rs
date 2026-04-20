pub mod game;
pub mod gameover;
pub mod help;
pub mod menu;
pub mod pause;
pub mod styled_box;
pub mod theme;
pub mod title;

use crate::app::App;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &mut App) {
    match &app.screen {
        crate::app::Screen::Title => title::render(f, app.title_selected),
        crate::app::Screen::DifficultySelect { selected } => menu::render_difficulty(f, *selected),
        crate::app::Screen::DiceOff { state } => menu::render_dice_off(f, state),
        crate::app::Screen::Game => game::render_game(f, app),
        crate::app::Screen::PauseMenu { selected } => {
            let sel = *selected;
            game::render_game(f, app);
            pause::render_pause_menu(f, sel);
        }
        crate::app::Screen::Help { from_game } => {
            let from = *from_game;
            if from {
                game::render_game(f, app);
            } else {
                title::render(f, app.title_selected);
            }
            help::render_help(f, &mut app.help_scroll);
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
            let elapsed = app.stats.elapsed();
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
