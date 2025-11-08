mod app;
mod input;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

use app::{App, InputMode};

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    let tick_rate = Duration::from_millis(16); // ~60fps
    let mut last_tick = Instant::now();

    // Main event loop
    let result = run_app(&mut terminal, &mut app, tick_rate, &mut last_tick);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
    last_tick: &mut Instant,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle input
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') if !app.command_palette_open => {
                            // Quit application
                            return Ok(());
                        }
                        KeyCode::Char('l') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            app.enter_insert_mode();
                        }
                        KeyCode::Char('t') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            app.new_tab();
                        }
                        KeyCode::Char('w') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            app.close_current_tab();
                        }
                        KeyCode::Char('p') if key.modifiers.contains(event::KeyModifiers::CONTROL | event::KeyModifiers::SHIFT) => {
                            app.toggle_command_palette();
                        }
                        KeyCode::Char('a') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            app.toggle_ascii_mode();
                        }
                        KeyCode::Char('j') => {
                            // Scroll down (future: send to browser)
                        }
                        KeyCode::Char('k') => {
                            // Scroll up (future: send to browser)
                        }
                        KeyCode::Tab => {
                            app.next_tab();
                        }
                        KeyCode::BackTab => {
                            app.previous_tab();
                        }
                        _ => {}
                    },
                    InputMode::Insert => match key.code {
                        KeyCode::Enter => {
                            app.submit_url();
                        }
                        KeyCode::Esc => {
                            app.exit_insert_mode();
                        }
                        KeyCode::Char(c) => {
                            app.url_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.url_input.pop();
                        }
                        _ => {}
                    },
                    InputMode::Command => match key.code {
                        KeyCode::Esc => {
                            app.toggle_command_palette();
                        }
                        KeyCode::Char(c) => {
                            app.command_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.command_input.pop();
                        }
                        KeyCode::Enter => {
                            app.execute_command();
                        }
                        _ => {}
                    },
                }
            }
        }

        // Tick update
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            *last_tick = Instant::now();
        }
    }
}
