use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use ur_core::{
    board::Square,
    player::Player,
    state::{Board, GameRules, Move, PieceLocation},
};

// ── Color constants ──────────────────────────────────────────────────────────

pub const COLOR_P1: Color = Color::LightBlue;
pub const COLOR_P2: Color = Color::LightRed;
pub const COLOR_ROSETTE_BG: Color = Color::Rgb(61, 43, 31);
pub const COLOR_ROSETTE_FG: Color = Color::Yellow;
pub const COLOR_SELECT_BG: Color = Color::Yellow;

// ── Widget ───────────────────────────────────────────────────────────────────

/// Renders the Royal Game of Ur board into a ratatui buffer.
// Wired in Task 10 (gameplay screen layout).
#[allow(dead_code)]
pub struct BoardWidget<'a> {
    pub rules: &'a GameRules,
    pub board: &'a Board,
    pub selected_square: Option<Square>,
    pub target_squares: &'a [Square],
}

impl<'a> Widget for BoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for &sq in self.rules.board_shape.valid_squares() {
            let cx = area.x + sq.col as u16 * 5;
            let cy = area.y + sq.row as u16;

            // Skip squares that fall outside the render area.
            if cx + 3 >= area.x + area.width || cy >= area.y + area.height {
                continue;
            }

            let is_selected = self.selected_square == Some(sq);
            let is_target = self.target_squares.contains(&sq);
            let is_rosette = self.rules.board_shape.is_rosette(sq);
            let occupant = self.board.get(sq);

            // Determine background color.
            let bg = if is_selected {
                COLOR_SELECT_BG
            } else if is_rosette {
                COLOR_ROSETTE_BG
            } else if is_target {
                Color::Rgb(20, 40, 20)
            } else {
                Color::Reset
            };

            // Determine content string and foreground color.
            let (content, fg) = if let Some(piece) = occupant {
                let player_color = match piece.player {
                    Player::Player1 => COLOR_P1,
                    Player::Player2 => COLOR_P2,
                };
                let fg = if is_selected {
                    Color::Black
                } else {
                    player_color
                };
                (" \u{25CF} ", fg) // " ● "
            } else if is_target {
                (" \u{00B7} ", Color::DarkGray) // " · "
            } else if is_rosette {
                (" \u{2726} ", COLOR_ROSETTE_FG) // " ✦ "
            } else {
                ("   ", Color::Reset)
            };

            // Draw the left border character.
            buf.get_mut(cx, cy)
                .set_char('\u{2502}') // │
                .set_style(Style::default().fg(Color::DarkGray).bg(bg));

            // Draw the 3-char content cells.
            for (i, ch) in content.chars().enumerate() {
                buf.get_mut(cx + 1 + i as u16, cy)
                    .set_char(ch)
                    .set_style(Style::default().fg(fg).bg(bg));
            }
        }
    }
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Returns the destination square of `mv` if it lands on the board.
pub fn move_target(mv: &Move) -> Option<Square> {
    match mv.to {
        PieceLocation::OnBoard(sq) => Some(sq),
        _ => None,
    }
}

/// Returns the source square of `mv` if the piece was on the board.
pub fn move_source(mv: &Move) -> Option<Square> {
    match mv.from {
        PieceLocation::OnBoard(sq) => Some(sq),
        _ => None,
    }
}
