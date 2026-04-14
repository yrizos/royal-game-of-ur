use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

const MENU_ITEMS: [&str; 3] = ["Resume", "How to Play", "Quit"];

/// Renders a floating pause menu over the current screen.
pub fn render_pause_menu(f: &mut Frame, selected: usize) {
    let area = f.size();
    let popup = centered_rect(32, 7, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            " ⏸  Paused ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, &label)| {
            if i == selected {
                ListItem::new(format!("  ▶  {}  ", label)).style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(format!("     {}  ", label)).style(Style::default().fg(Color::Gray))
            }
        })
        .collect();

    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(List::new(items), inner);
}

/// Renders the full-screen help / rules screen.
pub fn render_help(f: &mut Frame) {
    let area = f.size();
    let popup = centered_rect(62, 22, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            " The Royal Game of Ur — Rules & Keys ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "GOAL  ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Be the first to move all 7 of your pieces off the board."),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "TURNS ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Press Space to roll 4 binary dice (result 0–4)."),
        ]),
        Line::from(vec![Span::raw(
            "      Use ↑↓ to select a piece, Enter to move it.",
        )]),
        Line::from(vec![Span::raw("      Rolling 0 forfeits your turn.")]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "BOARD ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("You (blue) are the left column, AI (red) is the right."),
        ]),
        Line::from(vec![Span::raw(
            "      The middle column is the shared battle zone.",
        )]),
        Line::from(vec![Span::raw(
            "      Your pieces enter at row 4, travel UP to row 1,",
        )]),
        Line::from(vec![Span::raw(
            "      cross the shared column DOWN from row 1 to row 8,",
        )]),
        Line::from(vec![Span::raw(
            "      then exit at the bottom of your own column.",
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "✦ ROSETTES ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Landing on a rosette grants an extra turn."),
        ]),
        Line::from(vec![Span::raw(
            "           Pieces on rosettes cannot be captured.",
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "CAPTURE   ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Land on an opponent's piece in the shared column to"),
        ]),
        Line::from(vec![Span::raw(
            "          send it back to their waiting pool.",
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "KEY BINDINGS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Space  ", Style::default().fg(Color::Cyan)),
            Span::raw("Roll dice      "),
            Span::styled("↑ ↓    ", Style::default().fg(Color::Cyan)),
            Span::raw("Select piece"),
        ]),
        Line::from(vec![
            Span::styled("  Enter  ", Style::default().fg(Color::Cyan)),
            Span::raw("Confirm move   "),
            Span::styled("Esc    ", Style::default().fg(Color::Cyan)),
            Span::raw("This menu"),
        ]),
        Line::from(vec![
            Span::styled("  L      ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle game log"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Press Esc, Enter or Space to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(Paragraph::new(text).alignment(Alignment::Left), inner);
}

/// Returns a centered rectangle of fixed char width and height, clamped to `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w, h)
}
