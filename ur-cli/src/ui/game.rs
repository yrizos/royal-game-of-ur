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
    dice::Dice,
    player::Player,
    state::{Board, GameRules, PieceLocation},
};

use crate::app::App;

// ── Color constants ──────────────────────────────────────────────────────────

pub const COLOR_P1: Color = Color::LightBlue;
pub const COLOR_P2: Color = Color::LightRed;
pub const COLOR_ROSETTE_BG: Color = Color::Rgb(61, 43, 31);
pub const COLOR_ROSETTE_FG: Color = Color::Yellow;
const COLOR_SELECTED_BG: Color = Color::Rgb(30, 60, 30);
const COLOR_TARGET_BG: Color = Color::Rgb(40, 20, 60);

/// Describes what the dice widget inside a player panel should show.
#[derive(Debug, Clone, Copy)]
pub enum PanelDice {
    /// Nothing to show (panel is inactive and no prior roll to display).
    Hidden,
    /// Auto-roll is queued; waiting for the delay to elapse.
    Pending,
    /// Dice roll animation is in progress; carries the current cycling display value.
    Animating(Dice),
    /// Roll landed with legal moves available; carries the final value.
    Result(Dice),
    /// Roll landed with no legal moves; displayed in red before auto-forfeit.
    NoMoves(Dice),
    /// Opponent's last roll, shown dimmed in the inactive panel.
    LastRoll(Dice),
    /// Rosette extra turn granted; immediate re-roll incoming.
    RosettePending,
}

/// Builds a `Line` showing four tetrahedral dice: ▲ for scored-side-up, △ for blank.
/// `value` filled dice are drawn in `color`; the rest in `DarkGray`.
fn dice_pips_line(value: u8, color: Color) -> Line<'static> {
    const FILLED: &str = "\u{25b2}"; // ▲
    const EMPTY:  &str = "\u{25b3}"; // △
    let mut spans = vec![Span::raw("  ")];
    for i in 0..4u8 {
        let (sym, c) = if i < value {
            (FILLED, color)
        } else {
            (EMPTY, Color::DarkGray)
        };
        spans.push(Span::styled(sym.to_string(), Style::default().fg(c)));
        if i < 3 {
            spans.push(Span::raw("  "));
        }
    }
    Line::from(spans)
}

// ── Board geometry helpers ───────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum BK {
    Top,
    Fh,
    Tc,
    Nh,
    To,
    Bot,
}

fn draw_hborder(buf: &mut Buffer, bx: u16, y: u16, cw: u16, kind: BK, style: Style) {
    let ml = cw + 1;
    let mr = 2 * cw + 2;
    let rr = 3 * cw + 3;

    match kind {
        BK::Nh => {
            buf.get_mut(bx + ml, y)
                .set_char('\u{251c}')
                .set_style(style);
            for dx in 1..=cw {
                buf.get_mut(bx + ml + dx, y)
                    .set_char('\u{2500}')
                    .set_style(style);
            }
            buf.get_mut(bx + mr, y)
                .set_char('\u{2524}')
                .set_style(style);
        }
        _ => {
            let (l, lm, rm, r) = match kind {
                BK::Top => ('\u{250c}', '\u{252c}', '\u{252c}', '\u{2510}'),
                BK::Fh => ('\u{251c}', '\u{253c}', '\u{253c}', '\u{2524}'),
                BK::Tc => ('\u{2514}', '\u{253c}', '\u{253c}', '\u{2518}'),
                BK::To => ('\u{250c}', '\u{253c}', '\u{253c}', '\u{2510}'),
                BK::Bot => ('\u{2514}', '\u{2534}', '\u{2534}', '\u{2518}'),
                BK::Nh => unreachable!(),
            };
            buf.get_mut(bx, y).set_char(l).set_style(style);
            for dx in 1..ml {
                buf.get_mut(bx + dx, y)
                    .set_char('\u{2500}')
                    .set_style(style);
            }
            buf.get_mut(bx + ml, y).set_char(lm).set_style(style);
            for dx in (ml + 1)..mr {
                buf.get_mut(bx + dx, y)
                    .set_char('\u{2500}')
                    .set_style(style);
            }
            buf.get_mut(bx + mr, y).set_char(rm).set_style(style);
            for dx in (mr + 1)..rr {
                buf.get_mut(bx + dx, y)
                    .set_char('\u{2500}')
                    .set_style(style);
            }
            buf.get_mut(bx + rr, y).set_char(r).set_style(style);
        }
    }
}

/// Computes cell dimensions from available grid height.
///
/// Returns `(cell_w, cell_h)`. Only odd heights (1, 3, 5) are used so
/// symbols always land on a true center row. Cell width is `2*h + 1`
/// for a roughly-square look in a monospace terminal.
fn cell_dims(grid_h: u16) -> (u16, u16) {
    // 8 cells + 9 border rows → ch=5 needs 49, ch=3 needs 33, ch=1 needs 17
    let ch: u16 = if grid_h >= 49 {
        5
    } else if grid_h >= 33 {
        3
    } else {
        1
    };
    let cw = ch * 2 + 1;
    (cw, ch)
}

// ── Widget ───────────────────────────────────────────────────────────────────

/// Renders the Royal Game of Ur board with dynamic cell sizing and full borders.
pub struct BoardWidget<'a> {
    pub rules: &'a GameRules,
    pub board: &'a Board,
    /// The currently selected piece's source square (highlighted bg).
    pub selected_square: Option<Square>,
    /// Where the selected piece would land (preview bg).
    pub target_square: Option<Square>,
    /// Piece counts: [Player1, Player2].
    pub unplayed: [u8; 2],
    pub scored: [u8; 2],
    /// True when the selected move enters from pool (highlight pool indicator).
    pub pool_selected: bool,
    pub animation: Option<&'a crate::animation::Animation>,
    pub cell_w: u16,
    pub cell_h: u16,
}

impl<'a> Widget for BoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let cw = self.cell_w;
        let ch = self.cell_h;
        let total_w = 3 * cw + 4;
        let total_h = 8 * ch + 9;

        if area.width < total_w || area.height < total_h {
            return;
        }

        let bx = area.x + (area.width - total_w) / 2;
        let by = area.y + (area.height - total_h) / 2;
        let bs = Style::default().fg(Color::DarkGray);

        // ── Horizontal borders ─────────────────────────────────────────
        for i in 0..=8u16 {
            let y = by + i * (ch + 1);
            let kind = match i {
                0 => BK::Top,
                4 => BK::Tc,
                5 => BK::Nh,
                6 => BK::To,
                8 => BK::Bot,
                _ => BK::Fh,
            };
            draw_hborder(buf, bx, y, cw, kind, bs);
        }

        // ── Animation state ────────────────────────────────────────────
        let capture_flash_sq: Option<Square> = match self.animation {
            Some(crate::animation::Animation::CaptureFlash {
                square,
                frames_remaining,
            }) if *frames_remaining % 2 == 1 => Some(*square),
            _ => None,
        };
        let piece_move_ghost: Option<Square> = match self.animation {
            Some(crate::animation::Animation::PieceMove { remaining, .. }) => {
                remaining.first().copied()
            }
            _ => None,
        };
        let ghost_is_p1: bool = matches!(
            self.animation,
            Some(crate::animation::Animation::PieceMove {
                is_player1: true,
                ..
            })
        );

        // ── Cell contents ──────────────────────────────────────────────
        for &sq in self.rules.board_shape.valid_squares() {
            let sc = 2u16.saturating_sub(sq.row as u16);
            let cx = bx + sc * (cw + 1) + 1;
            let cy = by + sq.col as u16 * (ch + 1) + 1;

            let is_selected = self.selected_square == Some(sq);
            let is_target = self.target_square == Some(sq);
            let is_rosette = self.rules.board_shape.is_rosette(sq);
            let occupant = self.board.get(sq);

            let (sym, fg, bg) = if capture_flash_sq == Some(sq) {
                ('\u{25cf}', Color::LightRed, Color::Reset)
            } else if piece_move_ghost == Some(sq) {
                let gc = if ghost_is_p1 { COLOR_P1 } else { COLOR_P2 };
                let gb = if is_rosette {
                    COLOR_ROSETTE_BG
                } else {
                    Color::Reset
                };
                ('\u{25cf}', gc, gb)
            } else if is_selected {
                if let Some(piece) = occupant {
                    let pc = match piece.player {
                        Player::Player1 => COLOR_P1,
                        Player::Player2 => COLOR_P2,
                    };
                    ('\u{25cf}', pc, COLOR_SELECTED_BG)
                } else if is_rosette {
                    ('\u{2726}', COLOR_ROSETTE_FG, COLOR_SELECTED_BG)
                } else {
                    (' ', Color::Reset, COLOR_SELECTED_BG)
                }
            } else if is_target {
                if let Some(piece) = occupant {
                    let pc = match piece.player {
                        Player::Player1 => COLOR_P1,
                        Player::Player2 => COLOR_P2,
                    };
                    ('\u{25cf}', pc, COLOR_TARGET_BG)
                } else if is_rosette {
                    ('\u{2726}', COLOR_ROSETTE_FG, COLOR_TARGET_BG)
                } else {
                    (' ', Color::Reset, COLOR_TARGET_BG)
                }
            } else if let Some(piece) = occupant {
                let pc = match piece.player {
                    Player::Player1 => COLOR_P1,
                    Player::Player2 => COLOR_P2,
                };
                let bg = if is_rosette {
                    COLOR_ROSETTE_BG
                } else {
                    Color::Reset
                };
                ('\u{25cf}', pc, bg)
            } else if is_rosette {
                ('\u{2726}', COLOR_ROSETTE_FG, COLOR_ROSETTE_BG)
            } else {
                (' ', Color::Reset, Color::Reset)
            };

            if bg != Color::Reset {
                let cell_bg = Style::default().bg(bg);
                for dy in 0..ch {
                    for dx in 0..cw {
                        buf.get_mut(cx + dx, cy + dy)
                            .set_char(' ')
                            .set_style(cell_bg);
                    }
                }
            }

            if sym != ' ' {
                let sx = cx + (cw - 1) / 2;
                let sy = cy + (ch.saturating_sub(1)) / 2;
                let ss = Style::default().fg(fg).bg(bg);
                buf.get_mut(sx, sy).set_char(sym).set_style(ss);
            }
        }

        // ── Vertical borders ───────────────────────────────────────────
        for game_col in 0..8u8 {
            let narrow = game_col == 4 || game_col == 5;
            let cy_base = by + game_col as u16 * (ch + 1) + 1;

            for dy in 0..ch {
                let y = cy_base + dy;
                if narrow {
                    buf.get_mut(bx + cw + 1, y)
                        .set_char('\u{2502}')
                        .set_style(bs);
                    buf.get_mut(bx + 2 * cw + 2, y)
                        .set_char('\u{2502}')
                        .set_style(bs);
                } else {
                    for &vx in &[0, cw + 1, 2 * cw + 2, 3 * cw + 3] {
                        buf.get_mut(bx + vx, y).set_char('\u{2502}').set_style(bs);
                    }
                }
            }
        }

        // ── Unplayed / scored indicators in the H-gap ───────────────────
        // P1 = row 2 → screen col 0, P2 = row 0 → screen col 2.
        // Col 4 (below entry) = unplayed, Col 5 (above exit) = scored.
        let colors = [COLOR_P1, COLOR_P2];
        let screen_cols: [u16; 2] = [0, 2];
        for pi in 0..2usize {
            let sc = screen_cols[pi];
            let pc = colors[pi];
            let cx = bx + sc * (cw + 1) + 1;

            // Unplayed (gap col 4)
            let count = self.unplayed[pi];
            if count > 0 {
                let gy = by + 4 * (ch + 1) + 1;
                let mid_x = cx + (cw - 1) / 2;
                let mid_y = gy + (ch.saturating_sub(1)) / 2;

                let is_pool_hl = pi == 0 && self.pool_selected;
                let bg = if is_pool_hl {
                    COLOR_SELECTED_BG
                } else {
                    Color::Reset
                };
                if is_pool_hl {
                    let cell_bg = Style::default().bg(bg);
                    for dy in 0..ch {
                        for dx in 0..cw {
                            buf.get_mut(cx + dx, gy + dy)
                                .set_char(' ')
                                .set_style(cell_bg);
                        }
                    }
                }

                let sx = if cw >= 5 { mid_x - 1 } else { mid_x };
                buf.get_mut(sx, mid_y)
                    .set_char('\u{25cf}')
                    .set_style(Style::default().fg(pc).bg(bg));
                if cw >= 5 {
                    let digit = char::from(b'0' + count);
                    buf.get_mut(sx + 2, mid_y)
                        .set_char(digit)
                        .set_style(Style::default().fg(pc).bg(bg).add_modifier(Modifier::DIM));
                }
            }

            // Scored (gap col 5)
            let scored = self.scored[pi];
            if scored > 0 {
                let gy = by + 5 * (ch + 1) + 1;
                let mid_x = cx + (cw - 1) / 2;
                let mid_y = gy + (ch.saturating_sub(1)) / 2;
                let sx = if cw >= 5 { mid_x - 1 } else { mid_x };
                buf.get_mut(sx, mid_y)
                    .set_char('\u{25cf}')
                    .set_style(Style::default().fg(pc).add_modifier(Modifier::BOLD));
                if cw >= 5 {
                    let digit = char::from(b'0' + scored);
                    buf.get_mut(sx + 2, mid_y)
                        .set_char(digit)
                        .set_style(Style::default().fg(pc));
                }
            }
        }
    }
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Renders a player status panel showing captures, turn indicator, and dice widget.
pub fn render_player_panel(
    f: &mut Frame,
    area: Rect,
    player: Player,
    is_human: bool,
    is_current: bool,
    captures: u32,
    panel_dice: PanelDice,
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

    let turn_indicator = if is_current {
        if is_human {
            "\u{25b6} YOUR TURN"
        } else {
            "\u{25b6} THINKING..."
        }
    } else {
        ""
    };

    let mut text = vec![
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

    // Dice widget
    match panel_dice {
        PanelDice::Hidden => {}
        PanelDice::Pending => {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                "  rolling...",
                Style::default().fg(Color::DarkGray),
            )));
        }
        PanelDice::Animating(display) => {
            text.push(Line::from(""));
            text.push(dice_pips_line(display.value(), color));
            text.push(Line::from(Span::styled(
                "  = ?",
                Style::default().fg(Color::DarkGray),
            )));
        }
        PanelDice::Result(roll) => {
            text.push(Line::from(""));
            text.push(dice_pips_line(roll.value(), color));
            text.push(Line::from(Span::styled(
                format!("  = {}", roll.value()),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )));
            text.push(Line::from(Span::styled(
                "  pick a move",
                Style::default().fg(Color::Green),
            )));
        }
        PanelDice::NoMoves(roll) => {
            text.push(Line::from(""));
            text.push(dice_pips_line(roll.value(), Color::Red));
            text.push(Line::from(Span::styled(
                format!("  = {}  no moves", roll.value()),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            text.push(Line::from(Span::styled(
                "  passing turn...",
                Style::default().fg(Color::DarkGray),
            )));
        }
        PanelDice::LastRoll(roll) => {
            text.push(Line::from(""));
            text.push(dice_pips_line(roll.value(), Color::DarkGray));
            text.push(Line::from(Span::styled(
                format!("  = {} (last roll)", roll.value()),
                Style::default().fg(Color::DarkGray),
            )));
        }
        PanelDice::RosettePending => {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                "  \u{2736} rosette bonus!",
                Style::default().fg(Color::Yellow),
            )));
            text.push(Line::from(Span::styled(
                "  rolling again...",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

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
            let filled = "\u{25cf}".repeat(d.value() as usize);
            let empty = "\u{25cb}".repeat((4 - d.value()) as usize);
            format!("Dice: {}{} = {}  ", filled, empty, d.value())
        }
        None => "Dice: \u{2014}        ".to_string(),
    };

    let spinner = ["\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}"][ai_spinner_frame as usize % 4];
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
    let right = format!(
        "  Spc=Roll  \u{2191}\u{2193}=Select  Enter=Move  Esc=Pause  {}",
        log_hint
    );

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

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    let main = rows[0];
    let status_area = rows[1];

    // Dynamic cell sizing: fill available height with properly bordered squares.
    let grid_avail = main.height.saturating_sub(1); // 1 row for column headers
    let (cw, ch) = cell_dims(grid_avail);
    let board_w = 3 * cw + 4;
    let board_h = 8 * ch + 9;

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

    let path = rules.path_for(game_state.current_player);
    let pool_selected = app.cursor_path_pos == 0;
    let selected_square = if app.cursor_path_pos == 0 {
        None
    } else {
        path.get(app.cursor_path_pos - 1)
    };

    let cursor_move = app.legal_move_at_cursor();
    let target_square = cursor_move.and_then(|mv| match mv.to {
        PieceLocation::OnBoard(sq) => Some(sq),
        _ => None,
    });

    // Player panels
    render_player_panel(
        f,
        cols[0],
        Player::Player1,
        true,
        game_state.current_player == Player::Player1,
        app.stats.captures[0],
        PanelDice::Hidden,
    );
    render_player_panel(
        f,
        cols[2],
        Player::Player2,
        false,
        game_state.current_player == Player::Player2,
        app.stats.captures[1],
        PanelDice::Hidden,
    );

    // Column headers centered above each board column
    let header_y = cols[1].y;
    if header_y < cols[1].y + cols[1].height {
        let bx_center = cols[1].x + (cols[1].width.saturating_sub(board_w)) / 2;
        let hbuf = f.buffer_mut();
        let headers: [(&str, Color); 3] = [
            ("YOU", COLOR_P1),
            ("\u{25c6}", Color::DarkGray),
            ("AI", COLOR_P2),
        ];
        for (i, (label, fg)) in headers.iter().enumerate() {
            let col_x = bx_center + i as u16 * (cw + 1) + 1;
            let label_w: u16 = label.chars().count() as u16;
            let pad = cw.saturating_sub(label_w) / 2;
            for (j, c) in label.chars().enumerate() {
                let x = col_x + pad + j as u16;
                if x < cols[1].x + cols[1].width {
                    hbuf.get_mut(x, header_y)
                        .set_char(c)
                        .set_style(Style::default().fg(*fg).add_modifier(Modifier::BOLD));
                }
            }
        }
    }

    // Board area (below header, vertically centered in remaining space)
    let below_header = main.height.saturating_sub(1);
    let vert_pad = below_header.saturating_sub(board_h) / 2;
    let board_area = Rect::new(
        cols[1].x,
        cols[1].y + 1 + vert_pad,
        cols[1].width,
        board_h.min(below_header),
    );

    BoardWidget {
        rules,
        board: &game_state.board,
        selected_square,
        target_square,
        unplayed: game_state.unplayed,
        scored: game_state.scored,
        pool_selected,
        animation: app.animation.as_ref(),
        cell_w: cw,
        cell_h: ch,
    }
    .render(board_area, f.buffer_mut());

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
