use super::theme::{COLOR_ACCENT, COLOR_DIM, COLOR_P1, COLOR_P2, COLOR_SUB};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::time::Duration;

/// Data needed to render the game-over screen.
pub struct GameOverData<'a> {
    /// True if the human player (Player 1) won.
    pub winner_is_human: bool,
    /// Total number of moves made in the game.
    pub moves: u32,
    /// Time elapsed since the game started.
    pub elapsed: Duration,
    /// Capture counts: index 0 = Player 1, index 1 = Player 2.
    pub captures: &'a [u32; 2],
}

/// Renders the game-over screen with result, stats, and navigation hints.
pub fn render(f: &mut Frame, data: &GameOverData) {
    let area = f.size();
    let vpad = area.height.saturating_sub(14) / 2;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vpad),
            Constraint::Min(14),
            Constraint::Length(vpad),
        ])
        .split(area);

    let gold = Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let sub = Style::default().fg(COLOR_SUB);
    let dim = Style::default().fg(COLOR_DIM);
    let blue = Style::default().fg(COLOR_P1);
    let red = Style::default().fg(COLOR_P2);

    let winner_text = if data.winner_is_human {
        "You win!"
    } else {
        "AI wins!"
    };
    let winner_style = if data.winner_is_human {
        Style::default().fg(COLOR_P1).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_P2).add_modifier(Modifier::BOLD)
    };

    let time_str = super::theme::format_duration(data.elapsed);

    let lines = vec![
        Line::from(Span::styled("═══ Game Over ═══", gold)).alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(winner_text, winner_style)).alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(format!("Moves:    {}", data.moves), sub))
            .alignment(Alignment::Center),
        Line::from(Span::styled(format!("Time:     {}", time_str), sub))
            .alignment(Alignment::Center),
        Line::from(Span::styled(
            format!("Your captures:  {}", data.captures[0]),
            blue,
        ))
        .alignment(Alignment::Center),
        Line::from(Span::styled(
            format!("AI captures:    {}", data.captures[1]),
            red,
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled("[N] New Game    [Q] Quit", dim)).alignment(Alignment::Center),
    ];

    f.render_widget(Paragraph::new(lines), chunks[1]);
}
