pub mod game;
pub mod gameover;
pub mod menu;
pub mod title;

use crate::app::App;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    match &app.screen {
        crate::app::Screen::Title => title::render(f, app.title_selected),
        crate::app::Screen::DifficultySelect { selected } => menu::render_difficulty(f, *selected),
        _ => {}
    }
}

pub fn render_too_small(f: &mut Frame) {
    use ratatui::widgets::Paragraph;
    let msg = Paragraph::new("Terminal too small — please resize to at least 80×24");
    f.render_widget(msg, f.size());
}
