//! Volt — TUI Settings Editor for Amp.

mod app;
mod config;
mod editor;
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

use app::{App, EditorRequest, Focus, InputMode};
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

            if app.is_editing() {
                let editor_req = handle_modal_input(app, key.code);
                if let Some(req) = editor_req {
                    run_editor(terminal, app, &req)?;
                }
            } else {
                let editor_req = handle_normal_input(app, key.code, key.modifiers);
                if let Some(req) = editor_req {
                    run_editor(terminal, app, &req)?;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Suspends the TUI, runs `$EDITOR`, and applies the result.
fn run_editor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    request: &EditorRequest,
) -> Result<()> {
    // Suspend TUI
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    let result = editor::edit_value_in_editor(&request.value);

    // Restore TUI
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    terminal.clear()?;

    match result {
        Ok(edited) => app.apply_editor_result(request, edited),
        Err(e) => app.status_message = Some(format!("Editor error: {e}")),
    }

    Ok(())
}

fn handle_modal_input(app: &mut App, key: KeyCode) -> Option<EditorRequest> {
    match app.input_mode {
        InputMode::EditingValue => {
            match key {
                KeyCode::Enter => app.commit_edit(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Backspace => {
                    app.edit_buffer.pop();
                }
                KeyCode::Char(c) => app.edit_buffer.push(c),
                _ => {}
            }
            None
        }
        InputMode::EnteringKeyName => {
            match key {
                KeyCode::Enter => app.commit_key_name(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Backspace => {
                    app.edit_buffer.pop();
                }
                KeyCode::Char(c) => app.edit_buffer.push(c),
                _ => {}
            }
            None
        }
        InputMode::SelectingType => {
            match key {
                KeyCode::Enter => return app.commit_type_selection(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Up | KeyCode::Char('k') => app.type_select_up(),
                KeyCode::Down | KeyCode::Char('j') => app.type_select_down(),
                _ => {}
            }
            None
        }
        InputMode::EnteringCustomValue => {
            match key {
                KeyCode::Enter => app.commit_custom_value(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Backspace => {
                    app.edit_buffer.pop();
                }
                KeyCode::Char(c) => app.edit_buffer.push(c),
                _ => {}
            }
            None
        }
        InputMode::EnteringPermissionTool => {
            match key {
                KeyCode::Enter => app.commit_permission_tool(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Backspace => {
                    app.edit_buffer.pop();
                }
                KeyCode::Char(c) => app.edit_buffer.push(c),
                _ => {}
            }
            None
        }
        InputMode::SelectingPermissionLevel => {
            match key {
                KeyCode::Enter => app.commit_permission_level(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Up | KeyCode::Char('k') => app.permission_level_up(),
                KeyCode::Down | KeyCode::Char('j') => app.permission_level_down(),
                _ => {}
            }
            None
        }
        InputMode::EnteringDelegateTo => {
            match key {
                KeyCode::Enter => app.commit_delegate_to(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Backspace => {
                    app.edit_buffer.pop();
                }
                KeyCode::Char(c) => app.edit_buffer.push(c),
                _ => {}
            }
            None
        }
        InputMode::ConfirmAdvancedEdit => match key {
            KeyCode::Char('y') | KeyCode::Enter => app.confirm_advanced_edit(),
            KeyCode::Char('n') | KeyCode::Esc => {
                app.decline_advanced_edit();
                None
            }
            _ => None,
        },
        InputMode::EnteringMcpMatchField => {
            match key {
                KeyCode::Enter => app.commit_mcp_match_field(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Backspace => {
                    app.edit_buffer.pop();
                }
                KeyCode::Char(c) => app.edit_buffer.push(c),
                _ => {}
            }
            None
        }
        InputMode::EnteringMcpMatchValue => {
            match key {
                KeyCode::Enter => app.commit_mcp_match_value(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Backspace => {
                    app.edit_buffer.pop();
                }
                KeyCode::Char(c) => app.edit_buffer.push(c),
                _ => {}
            }
            None
        }
        InputMode::SelectingMcpPermissionLevel => {
            match key {
                KeyCode::Enter => app.commit_mcp_permission_level(),
                KeyCode::Esc => app.cancel_edit(),
                KeyCode::Up | KeyCode::Char('k') => app.mcp_permission_level_up(),
                KeyCode::Down | KeyCode::Char('j') => app.mcp_permission_level_down(),
                _ => {}
            }
            None
        }
        InputMode::ConfirmMcpEdit => match key {
            KeyCode::Char('y') | KeyCode::Enter => app.confirm_mcp_edit(),
            KeyCode::Char('n') | KeyCode::Esc => {
                app.decline_mcp_edit();
                None
            }
            _ => None,
        },
        InputMode::Normal => None,
    }
}

fn handle_normal_input(
    app: &mut App,
    key: KeyCode,
    modifiers: KeyModifiers,
) -> Option<EditorRequest> {
    match key {
        KeyCode::Char('q') => {
            app.should_quit = true;
            None
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_up();
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_down();
            None
        }
        KeyCode::Tab | KeyCode::BackTab => {
            app.toggle_focus();
            None
        }
        KeyCode::Enter => {
            if app.focus == Focus::Settings {
                app.activate_setting()
            } else {
                app.toggle_focus();
                None
            }
        }
        KeyCode::Char('e') => {
            if app.focus == Focus::Settings {
                app.force_editor()
            } else {
                None
            }
        }
        KeyCode::Char('a') => {
            if app.focus == Focus::Settings {
                app.add_array_item();
            }
            None
        }
        KeyCode::Char('d') => {
            if app.focus == Focus::Settings {
                app.delete_array_item();
            }
            None
        }
        KeyCode::Char('r') => {
            if app.focus == Focus::Settings {
                app.reset_setting();
            }
            None
        }
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.save();
            None
        }
        _ => None,
    }
}
