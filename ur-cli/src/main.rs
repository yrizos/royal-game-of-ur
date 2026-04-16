mod animation;
mod app;
mod input;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use input::{map_key, Action};

fn main() -> io::Result<()> {
    // Ensure terminal is restored if we panic.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let tick_rate = Duration::from_millis(33); // ~30 fps

    let result = run(&mut terminal, &mut app, tick_rate);

    // Restore terminal — run all steps regardless of individual failures.
    let r1 = disable_raw_mode();
    let r2 = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    );
    let r3 = terminal.show_cursor();

    // Return the game result, or the first teardown error if any.
    result.and(r1).and(r2).and(r3)
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    loop {
        let size = terminal.size()?;

        terminal.draw(|f| {
            if size.width < 80 || size.height < 24 {
                ui::render_too_small(f);
            } else {
                ui::render(f, app);
            }
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if let Some(action) = map_key(key, &app.screen) {
                    handle_action(app, action);
                }
            }
        } else {
            // Tick: advance animations
            animation::tick(app);
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_action(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.quit(),
        Action::NewGame => app.start_new_game(),
        Action::Confirm => app.handle_confirm(),
        Action::Back => app.handle_back(),
        Action::MenuUp => app.handle_menu_up(),
        Action::MenuDown => app.handle_menu_down(),
        Action::SelectPrev => app.handle_select_prev(),
        Action::SelectNext => app.handle_select_next(),
        Action::ConfirmMove => app.handle_confirm_move(),
        Action::ToggleLog => app.log_visible = !app.log_visible,
        Action::QuitPrompt => app.open_pause(),
        Action::ScrollUp => app.help_scroll_up(),
        Action::ScrollDown => app.help_scroll_down(),
    }
}
