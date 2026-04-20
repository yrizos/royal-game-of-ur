use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
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

use super::theme::{
    COLOR_ACCENT, COLOR_P1, COLOR_P2, COLOR_ROSETTE_BG, COLOR_SELECTED_BG, COLOR_TARGET_BG,
};
const COLOR_ROSETTE_FG: Color = COLOR_ACCENT;

/// Describes what the dice widget inside a player panel should show.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelDice {
    /// Nothing to show (panel is inactive and no prior roll to display).
    Hidden,
    /// Auto-roll is queued; waiting for the delay to elapse (carry a tick counter for animation).
    Pending(u32),
    /// Dice roll animation is in progress; carries `frames_remaining` for the rolling pattern.
    Animating(u32),
    /// Roll landed with legal moves available; carries the final value.
    Result(Dice),
    /// Roll landed with no legal moves; displayed in red before auto-forfeit.
    NoMoves(Dice),
    /// Opponent's last roll, shown dimmed in the inactive panel.
    LastRoll(Dice),
    /// Rosette extra turn granted; immediate re-roll incoming.
    RosettePending,
}

/// Renders four binary dice as three-row ASCII boxes:
///
/// ```text
///   ┌───┐ ┌───┐ ┌───┐ ┌───┐
///   │ ▲ │ │ ▲ │ │   │ │   │
///   └───┘ └───┘ └───┘ └───┘
/// ```
///
/// The first `value` dice show a filled pip in `color`; the rest are blank
/// and drawn in `DarkGray`. When `dim` is true all borders are `DarkGray`
/// (used for the inactive "last roll" display).
/// Shared builder: `filled[i]` determines whether die `i` shows a pip.
fn build_dice_lines(
    filled: [bool; 4],
    pip_color: Color,
    border_color: Color,
) -> Vec<Line<'static>> {
    let border = Style::default().fg(border_color);
    let mut top_spans = vec![Span::raw("  ")];
    let mut mid_spans = vec![Span::raw("  ")];
    let mut bot_spans = vec![Span::raw("  ")];

    for (i, &show) in filled.iter().enumerate() {
        top_spans.push(Span::styled("┌───┐", border));
        bot_spans.push(Span::styled("└───┘", border));
        mid_spans.push(Span::styled("│", border));
        if show {
            mid_spans.push(Span::styled(
                " ▲ ",
                Style::default().fg(pip_color).add_modifier(Modifier::BOLD),
            ));
        } else {
            mid_spans.push(Span::styled("   ", Style::default().fg(Color::DarkGray)));
        }
        mid_spans.push(Span::styled("│", border));
        if i < 3 {
            top_spans.push(Span::raw(" "));
            mid_spans.push(Span::raw(" "));
            bot_spans.push(Span::raw(" "));
        }
    }

    vec![
        Line::from(top_spans),
        Line::from(mid_spans),
        Line::from(bot_spans),
    ]
}

fn dice_box_lines(value: u8, color: Color, dim: bool) -> Vec<Line<'static>> {
    let filled = std::array::from_fn(|i| (i as u8) < value);
    let border_color = if dim { Color::DarkGray } else { color };
    build_dice_lines(filled, color, border_color)
}

/// Renders four dice boxes with a rapidly cycling pip pattern to convey rolling.
fn dice_rolling_lines(frame: u32, color: Color) -> Vec<Line<'static>> {
    const PATTERNS: [u8; 16] = [
        0b1010, 0b0101, 0b1100, 0b0011, 0b1001, 0b0110, 0b1111, 0b0000, 0b1110, 0b0111, 0b1011,
        0b1101, 0b0001, 0b1000, 0b0100, 0b0010,
    ];
    let pattern = PATTERNS[((frame / 2) as usize) % PATTERNS.len()];
    let filled = std::array::from_fn(|i| (pattern >> i) & 1 == 1);
    build_dice_lines(filled, color, color)
}

/// Returns exactly 3 `Line`s for the dice box display area. Always 3 lines — never
/// empty — so the panel layout section height is stable regardless of dice state.
pub fn dice_section_lines(panel_dice: PanelDice, color: Color) -> Vec<Line<'static>> {
    match panel_dice {
        PanelDice::Hidden => dice_box_lines(0, Color::DarkGray, true),
        PanelDice::Pending(frame) | PanelDice::Animating(frame) => dice_rolling_lines(frame, color),
        PanelDice::Result(roll) => dice_box_lines(roll.value(), color, false),
        PanelDice::NoMoves(roll) => dice_box_lines(roll.value(), Color::Red, false),
        PanelDice::LastRoll(roll) => dice_box_lines(roll.value(), color, true),
        PanelDice::RosettePending => dice_box_lines(0, Color::DarkGray, true),
    }
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
    /// True when the cursor is parked on the bear-off slot (highlight scored area).
    pub bear_off_selected: bool,
    /// True when the cursor move would score the piece (highlight scored area).
    pub target_will_score: bool,
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
        // Board is rendered top-to-bottom with col 7 at top, col 0 at bottom.
        // The H-shape gap (cols 4-5, only valid on the middle row) maps to
        // visual rows 2-3, so the transition borders are at i=2, 3, 4.
        for i in 0..=8u16 {
            let y = by + i * (ch + 1);
            let kind = match i {
                0 => BK::Top,
                2 => BK::Tc,
                3 => BK::Nh,
                4 => BK::To,
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

        // ── Step-number lookup (1-indexed path position per square) ───
        let p1_path = self.rules.path_for(Player::Player1);
        let p2_path = self.rules.path_for(Player::Player2);
        let step_for = |sq: Square| -> Option<usize> {
            let path = if sq.row == 0 { p2_path } else { p1_path };
            path.squares().iter().position(|&s| s == sq).map(|i| i + 1)
        };

        // ── Cell contents ──────────────────────────────────────────────
        for &sq in self.rules.board_shape.valid_squares() {
            let sc = 2u16.saturating_sub(sq.row as u16);
            let cx = bx + sc * (cw + 1) + 1;
            let cy = by + (7 - sq.col) as u16 * (ch + 1) + 1;

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

            // Step number in top-right corner, very dim
            if let Some(step) = step_for(sq) {
                let dim = Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM);
                let s = step.to_string();
                let chars: Vec<char> = s.chars().collect();
                let num_w = chars.len() as u16;
                if cw >= num_w {
                    for (i, &c) in chars.iter().enumerate() {
                        let nx = cx + cw - num_w + i as u16;
                        buf.get_mut(nx, cy).set_char(c).set_style(dim);
                    }
                }
            }
        }

        // ── Vertical borders ───────────────────────────────────────────
        for game_col in 0..8u8 {
            let narrow = game_col == 4 || game_col == 5;
            let cy_base = by + (7 - game_col) as u16 * (ch + 1) + 1;

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

            // Unplayed (gap col 4, flipped to visual row 3)
            let count = self.unplayed[pi];
            let is_pool_hl = pi == 0 && self.pool_selected;
            if count > 0 || is_pool_hl {
                let gy = by + 3 * (ch + 1) + 1;
                let mid_x = cx + (cw - 1) / 2;
                let mid_y = gy + (ch.saturating_sub(1)) / 2;
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
                if count > 0 {
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
            }

            // Scored (gap col 5, flipped to visual row 2)
            let scored = self.scored[pi];
            let is_score_target = pi == 0 && self.target_will_score;
            let is_bear_off_cursor = pi == 0 && self.bear_off_selected;
            if scored > 0 || is_score_target || is_bear_off_cursor {
                let gy = by + 2 * (ch + 1) + 1;
                let mid_x = cx + (cw - 1) / 2;
                let mid_y = gy + (ch.saturating_sub(1)) / 2;
                let bg = if is_score_target {
                    COLOR_TARGET_BG
                } else if is_bear_off_cursor {
                    COLOR_SELECTED_BG
                } else {
                    Color::Reset
                };
                if is_score_target || is_bear_off_cursor {
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
                if scored > 0 {
                    buf.get_mut(sx, mid_y)
                        .set_char('\u{25cf}')
                        .set_style(Style::default().fg(pc).bg(bg).add_modifier(Modifier::BOLD));
                    if cw >= 5 {
                        let digit = char::from(b'0' + scored);
                        buf.get_mut(sx + 2, mid_y)
                            .set_char(digit)
                            .set_style(Style::default().fg(pc).bg(bg));
                    }
                }
            }
        }
    }
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Renders a player status panel with fixed-height sections so nothing shifts.
#[allow(clippy::too_many_arguments)]
pub fn render_player_panel(
    f: &mut Frame,
    area: Rect,
    player: Player,
    is_human: bool,
    is_current: bool,
    captures: u32,
    unplayed: u8,
    scored: u8,
    panel_dice: PanelDice,
    turn_log: &[Vec<String>],
) {
    use crate::ui::styled_box::StyledBox;

    let color = match player {
        Player::Player1 => COLOR_P1,
        Player::Player2 => COLOR_P2,
    };
    let label = if is_human { "You" } else { "AI" };
    let player_num = match player {
        Player::Player1 => 1,
        Player::Player2 => 2,
    };
    let title = format!("Player {} ({})", player_num, label);

    let border_color = if is_current { color } else { Color::DarkGray };

    let sb = StyledBox {
        title: &title,
        border_color,
        scrollable: false,
    };
    let inner = sb.render(f, area);

    // Fixed-height sections — dice/stats are fixed; turn log grows to fill space.
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // [0] turn indicator
            Constraint::Length(1), // [1] blank
            Constraint::Length(3), // [2] dice boxes (3 rows)
            Constraint::Length(1), // [3] pick-a-move prompt
            Constraint::Length(1), // [4] blank margin before turn log
            Constraint::Min(2),    // [5] turn log (grows with available space)
            Constraint::Length(1), // [6] scored
            Constraint::Length(1), // [7] pool
            Constraint::Length(1), // [8] captures
        ])
        .split(inner);

    // [0] Turn indicator
    let turn_text = if is_current {
        if is_human {
            "\u{25b6} YOUR TURN"
        } else {
            "\u{25b6} THINKING..."
        }
    } else {
        ""
    };
    f.render_widget(
        Paragraph::new(turn_text).style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
        sections[0],
    );

    // [2] Dice boxes (always rendered, 3 lines)
    let dice_lines = dice_section_lines(panel_dice, color);
    f.render_widget(Paragraph::new(dice_lines), sections[2]);

    // [3] Pick-a-move prompt (1 line)
    let prompt = if matches!(panel_dice, PanelDice::Result(_)) && is_human {
        Span::styled(
            "\u{25b6} pick a move",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };
    f.render_widget(Paragraph::new(Line::from(prompt)), sections[3]);
    // [4] blank margin — separates dice area from turn log

    // [5] Turn log — most recent turn at top (white), older turns below (dark gray)
    {
        use ratatui::widgets::{List, ListItem};
        let mut all_lines: Vec<(String, Color)> = Vec::new();
        for (i, turn) in turn_log.iter().rev().enumerate() {
            let c = if i == 0 {
                Color::White
            } else {
                Color::DarkGray
            };
            if i > 0 {
                all_lines.push((String::new(), Color::DarkGray));
            }
            for line in turn {
                all_lines.push((line.clone(), c));
            }
        }
        let available = sections[5].height as usize;
        let items: Vec<ListItem> = all_lines
            .into_iter()
            .take(available)
            .map(|(text, c)| ListItem::new(Span::styled(text, Style::default().fg(c))))
            .collect();
        f.render_widget(List::new(items), sections[5]);
    }

    // [6] Scored dots
    let scored_str = if scored == 0 {
        String::new()
    } else {
        let dots: String = "\u{25cf} ".repeat(scored as usize);
        dots.trim_end().to_string()
    };
    f.render_widget(
        Paragraph::new(format!("Scored  {}", scored_str)).style(Style::default().fg(color)),
        sections[6],
    );

    // [7] Pool dots
    let pool_str = if unplayed == 0 {
        String::new()
    } else {
        let dots: String = "\u{25cf} ".repeat(unplayed as usize);
        dots.trim_end().to_string()
    };
    f.render_widget(
        Paragraph::new(format!("Pool    {}", pool_str))
            .style(Style::default().fg(color).add_modifier(Modifier::DIM)),
        sections[7],
    );

    // [8] Captures
    f.render_widget(
        Paragraph::new(format!("Captures: {}", captures)).style(Style::default().fg(Color::Gray)),
        sections[8],
    );
}

/// Computes what the dice widget should display for `player`'s panel.
fn compute_panel_dice(app: &crate::app::App, player: Player) -> PanelDice {
    let gs = match &app.game_state {
        Some(gs) => gs,
        None => return PanelDice::Hidden,
    };

    if gs.current_player == player {
        // Active player — show current roll state.
        if app.rosette_reroll && app.pending_roll {
            return PanelDice::RosettePending;
        }
        if app.pending_roll {
            return PanelDice::Pending(app.frame_count);
        }
        if let Some(roll) = app.dice_roll {
            if let Some(crate::animation::Animation::DiceRoll { .. }) = &app.animation {
                return PanelDice::Animating(app.frame_count);
            }
            if app.forfeit_after.is_some() {
                return PanelDice::NoMoves(roll);
            }
            return PanelDice::Result(roll);
        }
        PanelDice::Hidden
    } else {
        // Inactive panel — show the player's last roll dimmed.
        if let Some(roll) = app.last_roll[player.index()] {
            return PanelDice::LastRoll(roll);
        }
        PanelDice::Hidden
    }
}

/// Renders the status bar at the bottom of the screen with move count, time, and log info.
#[allow(clippy::too_many_arguments)]
pub fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    moves: u32,
    elapsed: std::time::Duration,
    last_log: Option<&str>,
    log_visible: bool,
    ai_thinking: bool,
    ai_spinner_frame: u32,
) {
    let time_str = super::theme::format_duration(elapsed);

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
        "Moves: {}  Time: {}  {}  {}",
        moves, time_str, ai_str, log_entry
    );
    let right = format!(
        "  \u{2191}\u{2193}=Select  Enter=Move  Esc=Pause  {}  [H] help",
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

    let board_col_w = board_w + 4; // 2-char margin on each side
    let panel_w = main.width.saturating_sub(board_col_w) / 2;

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(panel_w),
            Constraint::Length(board_col_w),
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
    let bear_off_selected = app.cursor_path_pos == crate::app::CURSOR_BEAR_OFF;
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
    let target_will_score = cursor_move
        .map(|mv| matches!(mv.to, PieceLocation::Scored))
        .unwrap_or(false);

    // Player panels
    render_player_panel(
        f,
        cols[0],
        Player::Player1,
        true,
        game_state.current_player == Player::Player1,
        app.stats.captures[0],
        game_state.unplayed[0],
        game_state.scored[0],
        compute_panel_dice(app, Player::Player1),
        &app.turn_log[0],
    );
    render_player_panel(
        f,
        cols[2],
        Player::Player2,
        false,
        game_state.current_player == Player::Player2,
        app.stats.captures[1],
        game_state.unplayed[1],
        game_state.scored[1],
        compute_panel_dice(app, Player::Player2),
        &app.turn_log[1],
    );

    // Column headers — 1 blank row above for breathing room
    let header_y = cols[1].y + 1;
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

    // Board area (below blank + header rows, vertically centered in remaining space)
    let below_header = main.height.saturating_sub(1);
    let vert_pad = below_header.saturating_sub(board_h) / 2;
    let board_area = Rect::new(
        cols[1].x,
        cols[1].y + 2 + vert_pad,
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
        bear_off_selected,
        target_will_score,
        animation: app.animation.as_ref(),
        cell_w: cw,
        cell_h: ch,
    }
    .render(board_area, f.buffer_mut());

    // Status bar
    let elapsed = app.stats.elapsed();
    render_status_bar(
        f,
        status_area,
        app.stats.moves,
        elapsed,
        app.log.last().map(|e| e.text.as_str()),
        app.log_visible,
        app.ai_thinking,
        app.ai_spinner_frame,
    );

    // Log overlay
    if app.log_visible {
        render_log_overlay(f, area, &app.log);
    }
}

/// Renders a floating log overlay using StyledBox — padding and borders consistent
/// with the help modal.
fn render_log_overlay(f: &mut Frame, area: Rect, log: &[crate::app::LogEntry]) {
    use crate::ui::styled_box::StyledBox;
    use ratatui::widgets::{Clear, List, ListItem};

    let overlay = Rect::new(
        area.x + area.width / 4,
        area.y + 2,
        area.width / 2,
        area.height.saturating_sub(4),
    );

    f.render_widget(Clear, overlay);

    let sb = StyledBox {
        title: "Game Log",
        border_color: Color::Yellow,
        scrollable: true,
    };
    let content = sb.render(f, overlay);

    let items: Vec<ListItem> = log
        .iter()
        .rev()
        .map(|entry| {
            let (prefix, prefix_style) = match entry.player {
                Some(Player::Player1) => (
                    "You  ",
                    Style::default().fg(COLOR_P1).add_modifier(Modifier::BOLD),
                ),
                Some(Player::Player2) => (
                    "AI   ",
                    Style::default().fg(COLOR_P2).add_modifier(Modifier::BOLD),
                ),
                None => ("     ", Style::default().fg(Color::DarkGray)),
            };
            let line = Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::styled(entry.text.clone(), Style::default().fg(Color::Gray)),
            ]);
            ListItem::new(line)
        })
        .collect();

    f.render_widget(List::new(items), content);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Animation;
    use crate::app::App;
    use ur_core::{
        dice::Dice,
        player::Player,
        state::{GameRules, GameState},
    };

    #[test]
    fn test_render_log_overlay_does_not_panic_with_entries() {
        use crate::app::LogEntry;
        use ratatui::{backend::TestBackend, Terminal};

        let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
        let log = vec![
            LogEntry {
                player: Some(Player::Player1),
                text: "rolled 3".to_string(),
            },
            LogEntry {
                player: Some(Player::Player2),
                text: "captured".to_string(),
            },
            LogEntry {
                player: None,
                text: "system event".to_string(),
            },
        ];
        terminal
            .draw(|f| {
                render_log_overlay(f, f.size(), &log);
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(
            content.contains("You"),
            "Player1 log entry must render 'You' prefix"
        );
        assert!(
            content.contains("AI"),
            "Player2 log entry must render 'AI' prefix"
        );
    }

    fn make_app_with_game() -> App {
        let mut app = App::new();
        app.game_state = Some(GameState::new(&GameRules::finkel()));
        app
    }

    /// Branch 1: active player + rosette_reroll => RosettePending
    #[test]
    fn test_compute_panel_dice_rosette_pending() {
        let mut app = make_app_with_game();
        app.rosette_reroll = true;
        app.pending_roll = true;
        let result = compute_panel_dice(&app, Player::Player1);
        assert!(
            matches!(result, PanelDice::RosettePending),
            "expected RosettePending, got {:?}",
            result
        );
    }

    /// Branch 2: active player + pending_roll (no rosette) => Pending
    #[test]
    fn test_compute_panel_dice_pending() {
        let mut app = make_app_with_game();
        app.pending_roll = true;
        let result = compute_panel_dice(&app, Player::Player1);
        assert!(
            matches!(result, PanelDice::Pending(_)),
            "expected Pending(_), got {:?}",
            result
        );
    }

    /// Branch 3: active player + DiceRoll animation => Animating
    #[test]
    fn test_compute_panel_dice_animating() {
        let mut app = make_app_with_game();
        app.dice_roll = Some(Dice::new(3).unwrap());
        app.animation = Some(Animation::DiceRoll {
            frames_remaining: 5,
            final_value: Dice::new(3).unwrap(),
            display: Dice::new(2).unwrap(),
        });
        let result = compute_panel_dice(&app, Player::Player1);
        assert!(
            matches!(result, PanelDice::Animating(_)),
            "expected Animating(_), got {:?}",
            result
        );
    }

    /// Branch 4: active player + forfeit_after set + dice_roll set => NoMoves
    #[test]
    fn test_compute_panel_dice_no_moves() {
        let mut app = make_app_with_game();
        app.dice_roll = Some(Dice::new(1).unwrap());
        app.forfeit_after = Some(std::time::Instant::now());
        let result = compute_panel_dice(&app, Player::Player1);
        assert_eq!(result, PanelDice::NoMoves(Dice::new(1).unwrap()));
    }

    /// Branch 5: active player + dice_roll set, no animation, no forfeit => Result
    #[test]
    fn test_compute_panel_dice_result() {
        let mut app = make_app_with_game();
        app.dice_roll = Some(Dice::new(2).unwrap());
        let result = compute_panel_dice(&app, Player::Player1);
        assert_eq!(result, PanelDice::Result(Dice::new(2).unwrap()));
    }

    /// Branch 6: inactive player + last_roll set => LastRoll (works for both players)
    #[test]
    fn test_compute_panel_dice_last_roll() {
        let mut app = make_app_with_game();
        app.last_roll[1] = Some(Dice::new(4).unwrap());
        let result = compute_panel_dice(&app, Player::Player2);
        assert_eq!(result, PanelDice::LastRoll(Dice::new(4).unwrap()));
    }

    /// Branch 6b: inactive Player1 panel shows its last roll too
    #[test]
    fn test_compute_panel_dice_last_roll_player1_when_inactive() {
        let rules = GameRules::finkel();
        let mut gs = GameState::new(&rules);
        gs.current_player = Player::Player2;
        let mut app = App::new();
        app.game_state = Some(gs);
        app.last_roll[0] = Some(Dice::new(2).unwrap());
        let result = compute_panel_dice(&app, Player::Player1);
        assert_eq!(result, PanelDice::LastRoll(Dice::new(2).unwrap()));
    }

    /// Branch 7: inactive player with no last roll => Hidden
    #[test]
    fn test_compute_panel_dice_hidden_inactive() {
        let app = make_app_with_game();
        // Player2 is inactive, no last_opponent_roll
        let result = compute_panel_dice(&app, Player::Player2);
        assert!(
            matches!(result, PanelDice::Hidden),
            "expected Hidden, got {:?}",
            result
        );
    }

    /// Branch 7 (active path): active player + no roll, no pending => Hidden
    #[test]
    fn test_compute_panel_dice_hidden_active_no_roll() {
        let app = make_app_with_game();
        let result = compute_panel_dice(&app, Player::Player1);
        assert!(
            matches!(result, PanelDice::Hidden),
            "expected Hidden, got {:?}",
            result
        );
    }

    #[test]
    fn test_panel_dice_hidden_renders_empty_boxes() {
        // PanelDice::Hidden must still produce 3 dice box lines (not 0).
        let lines = dice_section_lines(PanelDice::Hidden, COLOR_P1);
        assert_eq!(lines.len(), 3, "hidden state must still render 3 dice rows");
    }

    #[test]
    fn test_render_player_panel_shows_event_msg_in_buffer() {
        use ratatui::{backend::TestBackend, Terminal};
        let backend = TestBackend::new(40, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render_player_panel(
                    f,
                    f.size(),
                    Player::Player1,
                    true,
                    false,
                    0,
                    7,
                    0,
                    PanelDice::LastRoll(Dice::new(3).unwrap()),
                    &[vec![
                        "rolled 3".to_string(),
                        "captured at step 10".to_string(),
                    ]],
                );
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(
            content.contains("rolled 3"),
            "turn_log entry 'rolled 3' must appear in rendered panel"
        );
        assert!(
            content.contains("captured at step 10"),
            "turn_log entry 'captured at step 10' must appear in rendered panel"
        );
    }
}
