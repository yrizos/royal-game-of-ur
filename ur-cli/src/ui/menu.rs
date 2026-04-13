use crate::app::DIFFICULTIES;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_difficulty(f: &mut Frame, selected: usize) {
    let area = f.size();
    let vpad = area.height.saturating_sub(12) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vpad),
            Constraint::Min(12),
            Constraint::Length(vpad),
        ])
        .split(area);

    let gold = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let sub = Style::default().fg(Color::Gray);
    let dim = Style::default().fg(Color::DarkGray);

    let descriptions = [
        "Casual play — AI looks 2 moves ahead",
        "Competent opponent — AI looks 4 moves ahead",
        "Strong play — AI looks 6 moves ahead",
    ];

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled("Select Difficulty", gold)).alignment(Alignment::Center),
        Line::from(""),
    ];

    for (i, ((label, _depth), desc)) in DIFFICULTIES.iter().zip(descriptions.iter()).enumerate() {
        if i == selected {
            lines.push(
                Line::from(vec![
                    Span::styled(
                        format!("▶ {label:<8}"),
                        Style::default().fg(Color::Black).bg(Color::Yellow),
                    ),
                    Span::styled(format!("  {desc}"), Style::default().fg(Color::Yellow)),
                ])
                .alignment(Alignment::Center),
            );
        } else {
            lines.push(
                Line::from(vec![
                    Span::styled(format!("  {label:<8}"), sub),
                    Span::styled(format!("  {desc}"), dim),
                ])
                .alignment(Alignment::Center),
            );
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled("[Esc] Back", dim)).alignment(Alignment::Center));

    f.render_widget(Paragraph::new(lines), chunks[1]);
}
