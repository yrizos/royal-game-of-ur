use super::theme::{COLOR_ACCENT, COLOR_DIM, COLOR_P1, COLOR_P2, COLOR_SUB};
use crate::app::{DiceOffState, DIFFICULTIES};
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
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let sub = Style::default().fg(COLOR_SUB);
    let dim = Style::default().fg(COLOR_DIM);

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
                        Style::default().fg(Color::Black).bg(COLOR_ACCENT),
                    ),
                    Span::styled(format!("  {desc}"), Style::default().fg(COLOR_ACCENT)),
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

/// Renders the first-player dice-off screen.
pub fn render_dice_off(f: &mut Frame, state: &DiceOffState) {
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
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD);
    let blue = Style::default().fg(COLOR_P1);
    let red = Style::default().fg(COLOR_P2);
    let sub = Style::default().fg(COLOR_SUB);
    let dim = Style::default().fg(COLOR_DIM);

    fn dice_display(d: ur_core::dice::Dice) -> String {
        let filled = "●".repeat(d.value() as usize);
        let empty = "○".repeat((4 - d.value()) as usize);
        format!("{filled}{empty} = {}", d.value())
    }

    let mut lines = vec![
        Line::from(Span::styled("First Player Roll", gold)).alignment(Alignment::Center),
        Line::from(Span::styled("Higher roll goes first. Ties re-roll.", sub))
            .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(
            format!("Player 1 (You):  {}", dice_display(state.p1_display)),
            blue,
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(
            format!("Player 2 (AI):   {}", dice_display(state.p2_display)),
            red,
        ))
        .alignment(Alignment::Center),
        Line::from(""),
    ];

    let both_done = state.p1_frames == 0 && state.p2_frames == 0;
    if both_done {
        let result = match state.winner {
            Some(ur_core::player::Player::Player1) => "Player 1 goes first!",
            Some(ur_core::player::Player::Player2) => "AI goes first!",
            None => "Tie — rolling again...",
        };
        lines.push(Line::from(Span::styled(result, gold)).alignment(Alignment::Center));
        lines.push(Line::from(""));
        if state.winner.is_some() {
            lines.push(Line::from(Span::styled("[Enter] Begin", dim)).alignment(Alignment::Center));
        }
    }

    f.render_widget(Paragraph::new(lines), chunks[1]);
}
