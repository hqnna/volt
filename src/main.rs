//! Volt — TUI Settings Editor for Amp.

mod app;
mod config;
mod settings;
mod ui;

use std::io;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Focus};
use config::Config;

/// Volt — TUI Settings Editor for Amp
#[derive(Parser, Debug)]
#[command(name = "volt", version, about)]
struct Cli {
    /// Path to the settings.json file (overrides default)
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = match cli.config {
        Some(p) => p,
        None => Config::default_path()?,
    };

    let config = Config::load(&config_path)?;
    let mut app = App::new(config);

    // Set up terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Clear status message on any key press
            app.status_message = None;

            if app.editing {
                handle_edit_input(app, key.code);
            } else {
                handle_normal_input(app, key.code, key.modifiers);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_edit_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter => app.commit_edit(),
        KeyCode::Esc => app.cancel_edit(),
        KeyCode::Backspace => {
            app.edit_buffer.pop();
        }
        KeyCode::Char(c) => {
            app.edit_buffer.push(c);
        }
        _ => {}
    }
}

fn handle_normal_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Tab | KeyCode::BackTab => app.toggle_focus(),
        KeyCode::Enter => {
            if app.focus == Focus::Settings {
                app.activate_setting();
            } else {
                app.toggle_focus();
            }
        }
        KeyCode::Char('r') => {
            if app.focus == Focus::Settings {
                app.reset_setting();
            }
        }
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.save();
        }
        _ => {}
    }
}
