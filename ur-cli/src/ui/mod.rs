pub mod title;
pub mod menu;
pub mod game;
pub mod gameover;

use ratatui::Frame;
use crate::app::App;

pub fn render(f: &mut Frame, app: &App) {
    // placeholder — will be implemented in Tasks 5-10
    let _ = app;
    let _ = f;
}

pub fn render_too_small(f: &mut Frame) {
    use ratatui::widgets::Paragraph;
    let msg = Paragraph::new("Terminal too small — please resize to at least 80×24");
    f.render_widget(msg, f.size());
}
