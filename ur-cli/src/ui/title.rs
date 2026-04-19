use super::theme::{COLOR_ACCENT, COLOR_DIM, COLOR_SUB};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const LOGO: &[&str] = &[
    " ██╗   ██╗██████╗ ",
    " ██║   ██║██╔══██╗",
    " ██║   ██║██████╔╝",
    " ██║   ██║██╔══██╗",
    " ╚██████╔╝██║  ██║",
    "  ╚═════╝ ╚═╝  ╚═╝",
];

const CUNEIFORM_BORDER: &str = "𒀭  𒀭  𒀭  𒀭  𒀭  𒀭  𒀭  𒀭";
const CUNEIFORM_SCRIPT: &str = "𒆳𒆳 𒀭𒂗𒍪 𒆳𒆳";

pub fn render(f: &mut Frame, selected: usize) {
    let area = f.size();

    // Centre vertically: border + empty + header + logo(6) + cuneiform + subtitle + empty + menu(3) + empty + border = 17
    let total_h = 17u16;
    let vpad = area.height.saturating_sub(total_h) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vpad),
            Constraint::Min(total_h),
            Constraint::Length(vpad),
        ])
        .split(area);
    let inner = chunks[1];

    let gold = Style::default().fg(COLOR_ACCENT);
    let dim = Style::default().fg(COLOR_DIM);
    let sub = Style::default().fg(COLOR_SUB);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(CUNEIFORM_BORDER, dim)).alignment(Alignment::Center));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("The Royal Game of", sub)).alignment(Alignment::Center));

    for logo_line in LOGO {
        lines.push(
            Line::from(Span::styled(*logo_line, gold.add_modifier(Modifier::BOLD)))
                .alignment(Alignment::Center),
        );
    }

    lines.push(Line::from(Span::styled(CUNEIFORM_SCRIPT, dim)).alignment(Alignment::Center));
    lines.push(
        Line::from(Span::styled("circa 2600 BCE · Mesopotamia", sub)).alignment(Alignment::Center),
    );
    lines.push(Line::from(""));

    let menu_items = ["New Game", "How to Play", "Quit"];
    for (i, item) in menu_items.iter().enumerate() {
        let style = if i == selected {
            Style::default().fg(Color::Black).bg(COLOR_ACCENT)
        } else {
            sub
        };
        lines.push(
            Line::from(Span::styled(format!("[ {item} ]"), style)).alignment(Alignment::Center),
        );
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(CUNEIFORM_BORDER, dim)).alignment(Alignment::Center));

    f.render_widget(Paragraph::new(lines), inner);
}
