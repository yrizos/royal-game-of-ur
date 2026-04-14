use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
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

/// Renders a player status panel showing piece counts, captures, and turn indicator.
// Wired in Task 10 (gameplay screen layout).
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn render_player_panel(
    f: &mut Frame,
    area: Rect,
    player: Player,
    is_human: bool,
    is_current: bool,
    unplayed: u8,
    scored: u8,
    total: u8,
    captures: u32,
) {
    let color = if player == Player::Player1 {
        COLOR_P1
    } else {
        COLOR_P2
    };
    let label = if is_human { "You" } else { "AI" };
    let player_num = match player {
        Player::Player1 => 1,
        Player::Player2 => 2,
    };
    let name = format!("Player {} ({})", player_num, label);

    let title_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_current {
            Style::default().fg(color)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(Span::styled(format!(" {} ", name), title_style));

    let on_board = total.saturating_sub(unplayed).saturating_sub(scored);

    let unplayed_str = "● ".repeat(unplayed as usize);
    let scored_str = "● ".repeat(scored as usize);
    let on_board_str = "● ".repeat(on_board as usize);

    let turn_indicator = if is_current {
        if is_human {
            "▶ YOUR TURN"
        } else {
            "▶ THINKING..."
        }
    } else {
        ""
    };

    let text = vec![
        Line::from(Span::styled(
            format!("Waiting: {}", unplayed_str),
            Style::default().fg(color),
        )),
        Line::from(Span::styled(
            format!("Board:   {}", on_board_str),
            Style::default().fg(color),
        )),
        Line::from(Span::styled(
            format!("Scored:  {}", scored_str),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Captures: {}", captures),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            turn_indicator,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
    ];

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(text), inner);
}

/// Renders the status bar at the bottom of the screen with dice, move count, time, and log info.
// Wired in Task 10 (gameplay screen layout).
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    dice_roll: Option<ur_core::dice::Dice>,
    moves: u32,
    elapsed: std::time::Duration,
    last_log: Option<&str>,
    log_visible: bool,
    ai_thinking: bool,
    ai_spinner_frame: u32,
) {
    let secs = elapsed.as_secs();
    let time_str = format!("{:02}:{:02}", secs / 60, secs % 60);

    let dice_str = match dice_roll {
        Some(d) => {
            let filled = "●".repeat(d.value() as usize);
            let empty = "○".repeat((4 - d.value()) as usize);
            format!("Dice: {}{} = {}  ", filled, empty, d.value())
        }
        None => "Dice: —        ".to_string(),
    };

    let spinner = ["⠋", "⠙", "⠹", "⠸"][ai_spinner_frame as usize % 4];
    let ai_str = if ai_thinking {
        format!("{} AI thinking  ", spinner)
    } else {
        String::new()
    };

    let log_hint = if log_visible {
        "[L] close log"
    } else {
        "[L] log"
    };
    let log_entry = last_log.unwrap_or("");

    let left = format!(
        "{} Moves: {}  Time: {}  {}  {}",
        dice_str, moves, time_str, ai_str, log_entry
    );
    let right = format!("  {}", log_hint);

    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::Gray)),
        Span::styled(right, Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(vec![line]), area);
}
