use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};

/// Unified box renderer used by player panels, log modal, and help modal.
///
/// Renders a bordered box with a consistent 1-char inner padding on all sides
/// and returns the padded content `Rect` to the caller.
pub struct StyledBox<'a> {
    /// Text shown in the top-left of the border (wrapped in spaces automatically).
    pub title: &'a str,
    /// Border and title colour.
    pub border_color: Color,
    /// Optional text shown in the bottom border (used for scroll hints).
    pub bottom_title: Option<String>,
}

impl<'a> StyledBox<'a> {
    /// Render the box into `area`, clear the area, and return the inner `Rect`
    /// with 1 char of padding removed on every side beyond the border.
    pub fn render(self, f: &mut Frame, area: Rect) -> Rect {
        let title_style = Style::default().fg(self.border_color);
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.border_color))
            .title(Span::styled(format!(" {} ", self.title), title_style));

        if let Some(ref bt) = self.bottom_title {
            block = block.title_bottom(Span::styled(
                bt.clone(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let inner = block.inner(area);
        f.render_widget(block, area);

        // 1-char padding on all four sides inside the border.
        Rect::new(
            inner.x.saturating_add(1),
            inner.y.saturating_add(1),
            inner.width.saturating_sub(2),
            inner.height.saturating_sub(2),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn make_terminal(w: u16, h: u16) -> Terminal<TestBackend> {
        Terminal::new(TestBackend::new(w, h)).unwrap()
    }

    #[test]
    fn test_styled_box_returns_padded_rect() {
        let mut terminal = make_terminal(20, 10);
        terminal
            .draw(|f| {
                let area = Rect::new(0, 0, 20, 10);
                let sb = StyledBox {
                    title: "Test",
                    border_color: Color::Yellow,
                    bottom_title: None,
                };
                let content = sb.render(f, area);
                // border=1, padding=1 on each side → content starts at (2,2)
                assert_eq!(content.x, 2);
                assert_eq!(content.y, 2);
                // width=20 − 2*border − 2*pad = 16
                assert_eq!(content.width, 16);
                // height=10 − 2*border − 2*pad = 6
                assert_eq!(content.height, 6);
            })
            .unwrap();
    }

    #[test]
    fn test_styled_box_with_bottom_title_does_not_panic() {
        let mut terminal = make_terminal(30, 8);
        terminal
            .draw(|f| {
                let area = Rect::new(0, 0, 30, 8);
                let sb = StyledBox {
                    title: "Log",
                    border_color: Color::Yellow,
                    bottom_title: Some(" ↑↓ scroll (1/5) ".to_string()),
                };
                let content = sb.render(f, area);
                assert_eq!(content.x, 2);
                assert_eq!(content.y, 2);
            })
            .unwrap();
    }

    #[test]
    fn test_styled_box_tiny_area_does_not_underflow() {
        let mut terminal = make_terminal(4, 4);
        terminal
            .draw(|f| {
                let area = Rect::new(0, 0, 4, 4);
                let sb = StyledBox {
                    title: "X",
                    border_color: Color::Red,
                    bottom_title: None,
                };
                let content = sb.render(f, area);
                // Should not panic; width/height saturate at 0.
                assert_eq!(content.width, 0);
                assert_eq!(content.height, 0);
            })
            .unwrap();
    }
}
