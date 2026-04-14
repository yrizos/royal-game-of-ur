use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
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

use crate::app::App;

// ── Color constants ──────────────────────────────────────────────────────────

pub const COLOR_P1: Color = Color::LightBlue;
pub const COLOR_P2: Color = Color::LightRed;
pub const COLOR_ROSETTE_BG: Color = Color::Rgb(61, 43, 31);
pub const COLOR_ROSETTE_FG: Color = Color::Yellow;
pub const COLOR_SELECT_BG: Color = Color::Yellow;

// ── Widget ───────────────────────────────────────────────────────────────────

/// Renders the Royal Game of Ur board into a ratatui buffer.
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

// ── Gameplay screen ──────────────────────────────────────────────────────────

/// Assembles the full gameplay screen: player panels, board, and status bar.
pub fn render_game(f: &mut Frame, app: &App) {
    let area = f.size();

    // Layout: main area over 1-line status bar
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    let main = rows[0];
    let status_area = rows[1];

    // Board: 8 cols × 5 chars + 1 border = 41 wide. Panels split the remainder.
    let board_w = 41u16;
    let panel_w = main.width.saturating_sub(board_w) / 2;

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(panel_w),
            Constraint::Length(board_w),
            Constraint::Min(panel_w),
        ])
        .split(main);

    let game_state = match &app.game_state {
        Some(gs) => gs,
        None => return,
    };

    let rules = &game_state.rules;

    // Collect selected source/target squares
    let selected_move = app.legal_moves.get(app.selected_move_idx);
    let selected_sq = selected_move.and_then(move_source);
    // Show only move destinations for the currently selected piece (same source square).
    let target_sqs: Vec<_> = app
        .legal_moves
        .iter()
        .filter(|m| move_source(m) == selected_sq)
        .filter_map(move_target)
        .collect();

    // Render player 1 panel (left)
    render_player_panel(
        f,
        cols[0],
        ur_core::player::Player::Player1,
        true, // human
        game_state.current_player == ur_core::player::Player::Player1,
        game_state.unplayed[0],
        game_state.scored[0],
        rules.piece_count,
        app.stats.captures[0],
    );

    // Render board (center), offset 1 row for visual breathing room
    let board_area = Rect::new(
        cols[1].x,
        cols[1].y + 1,
        cols[1].width,
        cols[1].height.saturating_sub(2),
    );
    BoardWidget {
        rules,
        board: &game_state.board,
        selected_square: selected_sq,
        target_squares: &target_sqs,
    }
    .render(board_area, f.buffer_mut());

    // Render player 2 panel (right)
    render_player_panel(
        f,
        cols[2],
        ur_core::player::Player::Player2,
        false, // AI
        game_state.current_player == ur_core::player::Player::Player2,
        game_state.unplayed[1],
        game_state.scored[1],
        rules.piece_count,
        app.stats.captures[1],
    );

    // Status bar
    let elapsed = app
        .stats
        .start_time
        .map(|t| t.elapsed())
        .unwrap_or(std::time::Duration::ZERO);
    render_status_bar(
        f,
        status_area,
        app.dice_roll,
        app.stats.moves,
        elapsed,
        app.log.last().map(|s| s.as_str()),
        app.log_visible,
        app.ai_thinking,
        app.ai_spinner_frame,
    );

    // Log overlay
    if app.log_visible {
        render_log_overlay(f, area, &app.log);
    }
}

/// Renders a floating log overlay over the gameplay screen.
fn render_log_overlay(f: &mut Frame, area: Rect, log: &[String]) {
    use ratatui::widgets::{List, ListItem};
    let overlay = Rect::new(
        area.x + area.width / 4,
        area.y + 2,
        area.width / 2,
        area.height.saturating_sub(4),
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Game Log [L to close] ");
    let items: Vec<ListItem> = log
        .iter()
        .rev()
        .map(|e| ListItem::new(e.as_str()))
        .collect();
    let list = List::new(items).block(block);
    f.render_widget(ratatui::widgets::Clear, overlay);
    f.render_widget(list, overlay);
}
