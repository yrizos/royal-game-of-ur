use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Clear, List, ListItem},
    Frame,
};

use crate::ui::styled_box::StyledBox;

const MENU_ITEMS: [&str; 3] = ["Resume", "How to Play", "Quit"];

/// Renders a floating pause menu over the current screen.
pub fn render_pause_menu(f: &mut Frame, selected: usize) {
    let area = f.size();
    let popup = centered_rect(32, 7, area);
    f.render_widget(Clear, popup);

    let sb = StyledBox {
        title: "\u{23f8}  Paused",
        border_color: Color::Yellow,
        scrollable: false,
    };
    let content = sb.render(f, popup);

    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, &label)| {
            if i == selected {
                ListItem::new(format!("  \u{25b6}  {label}  ")).style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(format!("     {label}  ")).style(Style::default().fg(Color::Gray))
            }
        })
        .collect();

    f.render_widget(List::new(items), content);
}

/// Returns a centered rectangle of fixed char width and height, clamped to `area`.
pub(crate) fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w, h)
}
