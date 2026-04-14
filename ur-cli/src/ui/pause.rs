use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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

fn h(s: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        s,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn sep() -> Line<'static> {
    Line::from(Span::styled(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        Style::default().fg(Color::Rgb(60, 50, 40)),
    ))
}

fn p(s: &'static str) -> Line<'static> {
    Line::from(s)
}

fn key(k: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {k:<12}"), Style::default().fg(Color::Cyan)),
        Span::raw(desc),
    ])
}

fn help_lines() -> Vec<Line<'static>> {
    vec![
        h("THE ROYAL GAME OF UR"),
        sep(),
        p("One of the oldest known board games, dating to ~2600 BCE in ancient"),
        p("Mesopotamia. Sets were excavated from the Royal Tombs of Ur (modern-day"),
        p("Iraq) by Sir Leonard Woolley in the 1920s."),
        p(""),
        p("These rules follow the reconstruction by Irving Finkel of the British"),
        p("Museum, based on a cuneiform clay tablet written around 177 BCE."),
        p(""),
        h("OBJECTIVE"),
        sep(),
        p("Be the first player to move all 7 of your pieces off the board."),
        p("Each piece must travel the full path before it can bear off."),
        p(""),
        h("THE BOARD"),
        sep(),
        p("The board is H-shaped with 20 squares in 3 columns:"),
        p(""),
        p("  YOU  (blue)  \u{2014} left column    (your private lane,  6 squares)"),
        p("  SHARED       \u{2014} middle column  (contested zone,     8 squares)"),
        p("  AI   (red)   \u{2014} right column   (AI's private lane,  6 squares)"),
        p(""),
        p("The private lanes are exclusive \u{2014} no capturing there. The shared"),
        p("middle column is where both players compete and captures happen."),
        p(""),
        h("THE PATH \u{2014} HOW PIECES MOVE"),
        sep(),
        p("Each player's route is 14 squares long (then the piece bears off):"),
        p(""),
        p("  Steps 1\u{20134}  : Your private lane. Enter at row 4, travel UP to row 1."),
        p("             Only your pieces can occupy these 4 squares."),
        p(""),
        p("  Steps 5-12 : Shared middle column. Travel DOWN from row 1 to row 8."),
        p("             Both players share these 8 squares \u{2014} danger zone!"),
        p(""),
        p("  Steps 13-14: Return to your private lane exit leg, rows 7 then 6."),
        p("             These 2 squares are also private \u{2014} safe from capture."),
        p(""),
        p("  Bear off  : After step 14, roll EXACTLY the right number to move"),
        p("             the piece off the board and score it. Overshooting is"),
        p("             not allowed \u{2014} you must land exactly on the exit."),
        p(""),
        h("TURNS"),
        sep(),
        p("On your turn:"),
        p(""),
        p("  1. Press SPACE to roll 4 binary dice. Each die shows 0 or 1."),
        p("     The total (0\u{20134}) is how many squares you must move ONE piece."),
        p(""),
        p("  2. Use \u{2191}/\u{2193} to cycle through your pieces that have a legal move."),
        p("     The selected piece is highlighted in yellow. Available"),
        p("     destination squares appear as green dots (\u{00b7})."),
        p(""),
        p("  3. Press ENTER to confirm and move the selected piece."),
        p(""),
        p("  \u{2022} Rolling 0 means no move is possible. Your turn is forfeited."),
        p("  \u{2022} If you have no legal move for any roll, the turn is also"),
        p("    forfeited automatically (the game moves on)."),
        p("  \u{2022} You cannot move a piece to a square already occupied by one"),
        p("    of your own pieces."),
        p("  \u{2022} You must land EXACTLY on the exit square to bear a piece off."),
        p(""),
        h("ROSETTES  \u{2726}"),
        sep(),
        p("Five squares on the board are marked with a gold rosette (\u{2726}):"),
        p(""),
        p("  \u{2022} Your private lane: rows 1 and 6 of your column"),
        p("  \u{2022} AI's private lane:  rows 1 and 6 of the AI's column"),
        p("  \u{2022} Shared column:      row 4 (the exact middle)"),
        p(""),
        p("Landing on a rosette grants TWO benefits:"),
        p("  1. You receive an EXTRA TURN \u{2014} roll and move again immediately."),
        p("  2. Your piece is SAFE \u{2014} it cannot be captured while on a rosette."),
        p(""),
        p("Rosettes are among the most strategically important squares. Holding"),
        p("the central shared rosette (row 4) blocks the opponent's path."),
        p(""),
        h("CAPTURE"),
        sep(),
        p("If your piece lands on a square in the SHARED column occupied by an"),
        p("opponent's piece that is NOT on a rosette, the opponent's piece is"),
        p("captured: it is returned to their waiting pool and must re-enter"),
        p("the board from scratch."),
        p(""),
        p("  \u{2022} Captures are ONLY possible in the shared middle column."),
        p("  \u{2022} Pieces on a rosette are immune to capture."),
        p("  \u{2022} There is no capture in private lanes (each has their own)."),
        p(""),
        h("ENTERING PIECES"),
        sep(),
        p("Pieces in your waiting pool (shown as \u{25cf} in the Waiting row of your"),
        p("panel) can enter the board. A piece enters at the first square of"),
        p("your private lane (row 4 of your column). You can only enter if"),
        p("that square is not already occupied by one of your own pieces."),
        p(""),
        h("DICE PROBABILITIES"),
        sep(),
        p("With 4 binary dice, outcomes follow a binomial distribution:"),
        p(""),
        p("  Roll 0 :  6.25%  (1 in 16)  \u{2014} no move, turn forfeited"),
        p("  Roll 1 : 25.00%  (4 in 16)"),
        p("  Roll 2 : 37.50%  (6 in 16)  \u{2014} most likely result"),
        p("  Roll 3 : 25.00%  (4 in 16)"),
        p("  Roll 4 :  6.25%  (1 in 16)  \u{2014} rare but powerful"),
        p(""),
        h("THE AI OPPONENT"),
        sep(),
        p("The AI uses expectiminimax search \u{2014} a minimax variant that averages"),
        p("over all possible dice rolls at chance nodes, weighted by probability."),
        p("The search depth controls how many moves ahead it looks:"),
        p(""),
        p("  Easy   \u{2014} depth 2  (looks 2 half-moves ahead, plays reactively)"),
        p("  Medium \u{2014} depth 4  (competent play, rarely makes obvious mistakes)"),
        p("  Hard   \u{2014} depth 6  (strong, uses captures and rosettes strategically)"),
        p(""),
        p("The evaluation heuristic rewards: piece advancement, pieces scored,"),
        p("rosette occupation, captures available, and opponent vulnerability."),
        p(""),
        h("KEY BINDINGS"),
        sep(),
        key("Space", "Roll the dice (when it is your turn)"),
        key(
            "\u{2191} / \u{2193}",
            "Cycle through pieces that have legal moves",
        ),
        key("Enter", "Confirm and execute the selected move"),
        key("Esc", "Open pause menu (Resume / How to Play / Quit)"),
        key("L", "Toggle the move-by-move game log overlay"),
        p(""),
        p("  Game over screen:"),
        key("N", "Start a new game"),
        key("Q", "Quit"),
        p(""),
        p("  In this help screen:"),
        key("\u{2191} / \u{2193} or j/k", "Scroll up / down"),
        key("Esc / Enter / Space", "Close and return"),
        p(""),
    ]
}

/// Renders the full-screen scrollable help / rules overlay.
pub fn render_help(f: &mut Frame, scroll: u16) {
    let area = f.size();
    // Use nearly the full screen, leaving a thin margin.
    let popup = Rect::new(
        area.x + 1,
        area.y,
        area.width.saturating_sub(2),
        area.height,
    );
    f.render_widget(Clear, popup);

    let lines = help_lines();
    let total_lines = lines.len() as u16;
    let inner_h = popup.height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(inner_h);
    let clamped = scroll.min(max_scroll);

    let scroll_hint = if max_scroll > 0 {
        format!(
            " \u{2191}\u{2193} scroll ({}/{}) \u{2014} Esc/Enter to close ",
            clamped + 1,
            max_scroll + 1
        )
    } else {
        " Esc or Enter to close ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            " The Royal Game of Ur \u{2014} Rules & How to Play ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            scroll_hint,
            Style::default().fg(Color::DarkGray),
        ));

    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(
        Paragraph::new(lines)
            .scroll((clamped, 0))
            .wrap(Wrap { trim: false }),
        inner,
    );
}

/// Returns a centered rectangle of fixed char width and height, clamped to `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w, h)
}
