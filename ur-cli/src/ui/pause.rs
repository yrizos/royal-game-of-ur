use ratatui::{
    layout::Rect,
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
            " \u{23f8}  Paused ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

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

    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(List::new(items), inner);
}

// ── Help screen ──────────────────────────────────────────────────────────────

// ── Formatting helpers ───────────────────────────────────────────────────────

fn h(s: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        s.to_string(),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn p(s: &'static str) -> Line<'static> {
    Line::from(s)
}

fn key(k: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{k:<14}"), Style::default().fg(Color::Cyan)),
        Span::raw(desc),
    ])
}

// ── Board diagram styles ─────────────────────────────────────────────────────

fn sty_border() -> Style {
    Style::default().fg(Color::DarkGray)
}
fn sty_p1() -> Style {
    Style::default().fg(Color::LightBlue)
}
fn sty_rosette() -> Style {
    Style::default().fg(Color::Yellow)
}
fn sty_you() -> Style {
    Style::default()
        .fg(Color::LightBlue)
        .add_modifier(Modifier::BOLD)
}
fn sty_mid() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}
fn sty_arrow() -> Style {
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD)
}

// ── Cell helpers (each totals 3 display chars) ───────────────────────────────

type Cell = Vec<Span<'static>>;

fn c_empty() -> Cell {
    vec![Span::raw("   ".to_string())]
}
fn c_p1() -> Cell {
    vec![Span::styled(" ● ".to_string(), sty_p1())]
}
fn c_p2() -> Cell {
    vec![Span::styled(
        " ○ ".to_string(),
        Style::default().fg(Color::LightRed),
    )]
}
fn c_up() -> Cell {
    vec![Span::styled(" \u{2191} ".to_string(), sty_arrow())]
}
fn c_down() -> Cell {
    vec![Span::styled(" \u{2193} ".to_string(), sty_arrow())]
}
fn c_left() -> Cell {
    vec![Span::styled(" \u{2190} ".to_string(), sty_arrow())]
}
fn c_right() -> Cell {
    vec![Span::styled(" \u{2192} ".to_string(), sty_arrow())]
}
/// Player piece sitting on a rosette: " ●" blue + "✦" yellow = 3 chars.
fn c_p1_on_r() -> Cell {
    vec![
        Span::styled(" ●".to_string(), sty_p1()),
        Span::styled("✦".to_string(), sty_rosette()),
    ]
}

fn sfmt(n: u8) -> String {
    if n < 10 {
        format!(" {} ", n)
    } else {
        format!("{} ", n)
    }
}
fn spre(n: u8) -> String {
    if n < 10 {
        format!(" {}", n)
    } else {
        format!("{}", n)
    }
}

fn c_you(n: u8) -> Cell {
    vec![Span::styled(sfmt(n), sty_you())]
}
fn c_you_r(n: u8) -> Cell {
    vec![
        Span::styled(spre(n), sty_you()),
        Span::styled("✦".to_string(), sty_rosette()),
    ]
}
fn c_sh(n: u8) -> Cell {
    vec![Span::styled(sfmt(n), sty_mid())]
}
fn c_sh_r(n: u8) -> Cell {
    vec![
        Span::styled(spre(n), sty_mid()),
        Span::styled("✦".to_string(), sty_rosette()),
    ]
}

// ── Board layout engine ──────────────────────────────────────────────────────

type Board = [[Cell; 3]; 8];

#[derive(Clone, Copy)]
enum BL {
    Hdr,
    Top,
    R(usize),
    HB,
    TC,
    NR(usize),
    Nhb,
    TO,
    Bot,
}

const SEQ: [BL; 18] = [
    BL::Hdr,
    BL::Top,
    BL::R(0), // col=7: steps 13/12
    BL::HB,
    BL::R(1),  // col=6: steps 14✦/11
    BL::TC,    // narrow gap opens
    BL::NR(2), // col=5: step 10
    BL::Nhb,
    BL::NR(3), // col=4: step 9
    BL::TO,    // narrow gap closes
    BL::R(4),  // col=3: steps 1/8✦
    BL::HB,
    BL::R(5), // col=2: steps 2/7
    BL::HB,
    BL::R(6), // col=1: steps 3/6
    BL::HB,
    BL::R(7), // col=0: steps 4✦/5
    BL::Bot,
];

/// Returns spans for one board line (exactly 13 display chars).
fn bspans(board: &Board, bl: BL) -> Vec<Span<'static>> {
    let b = sty_border();
    match bl {
        BL::Hdr => vec![
            Span::styled(
                " YOU",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("◆", Style::default().fg(Color::DarkGray)),
            Span::raw("   "),
            Span::styled(
                "AI",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ],
        BL::Top => vec![Span::styled("┌───┬───┬───┐".to_string(), b)],
        BL::Bot => vec![Span::styled("└───┴───┴───┘".to_string(), b)],
        BL::HB => vec![Span::styled("├───┼───┼───┤".to_string(), b)],
        BL::TC => vec![Span::styled("└───┼───┼───┘".to_string(), b)],
        BL::TO => vec![Span::styled("┌───┼───┼───┐".to_string(), b)],
        BL::Nhb => vec![
            Span::raw("    ".to_string()),
            Span::styled("├───┤".to_string(), b),
            Span::raw("    ".to_string()),
        ],
        BL::R(i) => {
            let row = &board[i];
            let mut s = vec![Span::styled("│".to_string(), b)];
            s.extend(row[0].clone());
            s.push(Span::styled("│".to_string(), b));
            s.extend(row[1].clone());
            s.push(Span::styled("│".to_string(), b));
            s.extend(row[2].clone());
            s.push(Span::styled("│".to_string(), b));
            s
        }
        BL::NR(i) => {
            let mut s = vec![
                Span::raw("    ".to_string()),
                Span::styled("│".to_string(), b),
            ];
            s.extend(board[i][1].clone());
            s.push(Span::styled("│".to_string(), b));
            s.push(Span::raw("    ".to_string()));
            s
        }
    }
}

/// Side-by-side boards with custom labels above each.
fn render_dual(
    ind: &str,
    gap: &str,
    left_label: &str,
    right_label: &str,
    left: &Board,
    right: &Board,
) -> Vec<Line<'static>> {
    let label_pad = 13usize.saturating_sub(left_label.len()) + gap.len();
    let mut lines = vec![Line::from(vec![
        Span::raw(ind.to_string()),
        Span::styled(left_label.to_string(), Style::default().fg(Color::DarkGray)),
        Span::raw(" ".repeat(label_pad)),
        Span::styled(
            right_label.to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ])];
    for &bl in &SEQ {
        let mut spans = vec![Span::raw(ind.to_string())];
        spans.extend(bspans(left, bl));
        spans.push(Span::raw(gap.to_string()));
        spans.extend(bspans(right, bl));
        lines.push(Line::from(spans));
    }
    lines
}

// ── Default board with all path numbers ──────────────────────────────────────

fn default_path_board() -> Board {
    // Row 0 = top (col 7), row 7 = bottom (col 0) — matches game board orientation.
    [
        [c_you(13), c_sh(12), c_empty()],   // col=7 (top)
        [c_you_r(14), c_sh(11), c_empty()], // col=6: step 14✦
        [c_empty(), c_sh(10), c_empty()],   // col=5 (narrow gap)
        [c_empty(), c_sh(9), c_empty()],    // col=4 (narrow gap)
        [c_you(1), c_sh_r(8), c_empty()],   // col=3: step 8✦
        [c_you(2), c_sh(7), c_empty()],     // col=2
        [c_you(3), c_sh(6), c_empty()],     // col=1
        [c_you_r(4), c_sh(5), c_empty()],   // col=0 (bottom): step 4✦
    ]
}

/// Arrows showing the direction of travel at each step.
fn flow_board() -> Board {
    // Matches flipped orientation: top=col7, bottom=col0.
    [
        [c_down(), c_left(), c_empty()], // col=7: step 13 ↓ (exit), step 12 ← (exit lane)
        [c_down(), c_up(), c_empty()],   // col=6: step 14 ↓ (scored), step 11 ↑
        [c_empty(), c_up(), c_empty()],  // col=5: step 10 ↑
        [c_empty(), c_up(), c_empty()],  // col=4: step 9 ↑
        [c_down(), c_up(), c_empty()],   // col=3: step 1 ↓, step 8 ↑
        [c_down(), c_up(), c_empty()],   // col=2: step 2 ↓, step 7 ↑
        [c_down(), c_up(), c_empty()],   // col=1: step 3 ↓, step 6 ↑
        [c_right(), c_up(), c_empty()],  // col=0: step 4 → (enters shared), step 5 ↑
    ]
}

// ── Help content ─────────────────────────────────────────────────────────────

fn help_lines() -> Vec<Line<'static>> {
    let ind = " ";
    let gap = "          "; // 10-char gap between dual boards
    let mut lines = Vec::with_capacity(150);

    // ── Top padding ──────────────────────────────────────────────────────
    lines.push(p(""));

    // ── Introduction (no section title — flows from the top) ─────────────
    lines.push(p(
        "One of the oldest known board games, dating to about 2600 BCE in",
    ));
    lines.push(p(
        "ancient Mesopotamia. Sets were excavated from the Royal Tombs of",
    ));
    lines.push(p(
        "Ur (modern-day Iraq) by Sir Leonard Woolley in the 1920s.",
    ));
    lines.push(p(""));
    lines.push(p(
        "These rules follow Irving Finkel's reconstruction at the British",
    ));
    lines.push(p(
        "Museum, based on a cuneiform tablet from around 177 BCE.",
    ));
    lines.push(p(""));
    lines.push(p(
        "Objective: race all 7 pieces through a 14-step path and off the",
    ));
    lines.push(Line::from(vec![
        Span::raw("board before the AI. "),
        Span::styled("SPACE", Style::default().fg(Color::Cyan)),
        Span::raw(" roll \u{2192} \u{2191}\u{2193} select piece \u{2192} "),
        Span::styled("ENTER", Style::default().fg(Color::Cyan)),
        Span::raw(" move."),
    ]));
    lines.push(p(""));

    // ── YOUR PATH ────────────────────────────────────────────────────────
    lines.push(h("YOUR PATH (14 steps \u{2192} exit)"));
    lines.push(p(""));
    lines.push(p(
        "Steps 1\u{2013}4 your private lane (left). Steps 5\u{2013}12 the shared",
    ));
    lines.push(p(
        "column (both players!). Steps 13\u{2013}14 your exit lane (left).",
    ));
    lines.push(p(
        "\u{2726} = Rosette: extra turn + safe. Must roll exactly to exit.",
    ));
    lines.push(p("The AI\u{2019}s path mirrors yours on the right column."));
    lines.push(p(""));

    lines.extend(render_dual(
        ind,
        gap,
        "STEPS",
        "FLOW",
        &default_path_board(),
        &flow_board(),
    ));
    lines.push(p(""));

    // ── EXAMPLE: CAPTURING ───────────────────────────────────────────────
    lines.push(h("EXAMPLE: CAPTURING"));
    lines.push(p(""));
    lines.push(Line::from(vec![
        Span::raw("You roll 3. Your "),
        Span::styled("\u{25cf}", sty_p1()),
        Span::raw(" on step 7 captures the AI\u{2019}s "),
        Span::styled("\u{25cb}", Style::default().fg(Color::LightRed)),
        Span::raw(" at step 10."),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Captured "),
        Span::styled("\u{25cb}", Style::default().fg(Color::LightRed)),
        Span::raw(" returns to the AI\u{2019}s pool. Captures only happen in"),
    ]));
    lines.push(p(
        "the shared \u{25c6} column \u{2014} pieces on a \u{2726} are safe.",
    ));
    lines.push(p(""));

    let mut cap_before = default_path_board();
    cap_before[5][1] = c_p1(); // ● at step 7 (row 5, mid)
    cap_before[4][1] = c_up(); // ↑ passing through step 8
    cap_before[3][1] = c_up(); // ↑ passing through step 9
    cap_before[2][1] = c_p2(); // ○ at step 10 (row 2, mid)

    let mut cap_after = default_path_board();
    cap_after[2][1] = c_p1(); // ● landed at step 10

    lines.extend(render_dual(
        ind,
        gap,
        "BEFORE",
        "AFTER",
        &cap_before,
        &cap_after,
    ));
    lines.push(p(""));

    // ── EXAMPLE: ROSETTE ─────────────────────────────────────────────────
    lines.push(h("EXAMPLE: ROSETTE = EXTRA TURN"));
    lines.push(p(""));
    lines.push(Line::from(vec![
        Span::raw("You roll 1. Your "),
        Span::styled("\u{25cf}", sty_p1()),
        Span::raw(" on step 7 lands on step 8 (\u{2726}). Rosette \u{2014} you"),
    ]));
    lines.push(p("roll again! Pieces on a rosette are safe from capture."));
    lines.push(p(""));

    let mut ros_before = default_path_board();
    ros_before[5][1] = c_p1(); // ● at step 7 (row 5, mid)
    ros_before[4][1] = c_up(); // ↑ heading to step 8

    let mut ros_after = default_path_board();
    ros_after[4][1] = c_p1_on_r(); // ●✦ at step 8 rosette (row 4)

    lines.extend(render_dual(
        ind,
        gap,
        "BEFORE",
        "AFTER",
        &ros_before,
        &ros_after,
    ));
    lines.push(p(""));

    // ── EXAMPLE: BEARING OFF ─────────────────────────────────────────────
    lines.push(h("EXAMPLE: BEARING OFF"));
    lines.push(p(""));
    lines.push(Line::from(vec![
        Span::raw("You roll 1. Your "),
        Span::styled("\u{25cf}", sty_p1()),
        Span::raw(" on step 14 exits the board \u{2014} scored! You must"),
    ]));
    lines.push(p("roll exactly to bear off. First to score all 7 wins."));
    lines.push(p(""));

    let mut bo_before = default_path_board();
    bo_before[1][0] = vec![
        Span::styled(" \u{25cf}".to_string(), sty_p1()),
        Span::styled("\u{2193}".to_string(), sty_arrow()),
    ]; // ●↓ piece at step 14 (row 1, left), about to exit

    let bo_after = default_path_board(); // piece scored → square returns to default

    lines.extend(render_dual(
        ind, gap, "BEFORE", "AFTER", &bo_before, &bo_after,
    ));
    lines.push(p(""));

    // ── DICE ─────────────────────────────────────────────────────────────
    lines.push(h("DICE"));
    lines.push(p(""));
    lines.push(p("4 binary dice \u{2192} total 0 to 4. Roll 0 = no move."));
    lines.push(p(
        "0: 6%  \u{00b7}  1: 25%  \u{00b7}  2: 38%  \u{00b7}  3: 25%  \u{00b7}  4: 6%",
    ));
    lines.push(p(""));

    // ── AI OPPONENT ──────────────────────────────────────────────────────
    lines.push(h("AI OPPONENT"));
    lines.push(p(""));
    lines.push(p("Expectiminimax search, weighted by dice probability."));
    lines.push(p("Easy (depth 2) \u{00b7} Medium (4) \u{00b7} Hard (6)"));
    lines.push(p(""));

    // ── CONTROLS ─────────────────────────────────────────────────────────
    lines.push(h("CONTROLS"));
    lines.push(p(""));
    lines.push(key("Space", "Roll the dice"));
    lines.push(key(
        "\u{2191}\u{2193} / \u{2190}\u{2192}",
        "Select which piece to move",
    ));
    lines.push(key("Enter", "Confirm the move"));
    lines.push(key("H", "Help (this screen)"));
    lines.push(key("L", "Toggle game log"));
    lines.push(key("Esc", "Pause menu"));
    lines.push(p(""));
    lines.push(p(" In this screen:"));
    lines.push(key("\u{2191}\u{2193} or j/k", "Scroll"));
    lines.push(key("Esc / Enter", "Close"));

    // ── Bottom padding ───────────────────────────────────────────────────
    lines.push(p(""));

    lines
}

/// Renders the full-screen scrollable help / rules overlay.
///
/// Clamps `*scroll` to the valid range so the display never over-scrolls.
pub fn render_help(f: &mut Frame, scroll: &mut u16) {
    use crate::ui::styled_box::StyledBox;
    let area = f.size();
    let popup = centered_rect(72, area.height.saturating_sub(2), area);
    f.render_widget(Clear, popup);

    let lines = help_lines();
    let total_lines = lines.len() as u16;

    // Render the box first to get the content rect.
    let sb = StyledBox {
        title: "The Royal Game of Ur",
        border_color: Color::Yellow,
        scrollable: true,
    };
    let content = sb.render(f, popup);

    // Clamp scroll using actual available height.
    let max_scroll = total_lines.saturating_sub(content.height);
    *scroll = (*scroll).min(max_scroll);

    f.render_widget(Paragraph::new(lines).scroll((*scroll, 0)), content);
}

/// Returns a centered rectangle of fixed char width and height, clamped to `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w, h)
}
