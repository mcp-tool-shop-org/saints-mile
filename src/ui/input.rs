//! Input handling — crossterm event dispatch by screen.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::ui::mod_types::{App, AppScreen, InputResult, QuitOption, PauseOption};

/// Number of save slots available. Centralizes the limit so it isn't scattered.
const SAVE_SLOT_COUNT: usize = 3;

/// Handle a crossterm event. Returns what happened.
pub fn handle_event(app: &mut App, event: Event) -> InputResult {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Resize(_, _) => InputResult::Redraw,
        _ => InputResult::None,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> InputResult {
    // Quit confirmation screen gets its own handler — no universal shortcuts
    if matches!(app.screen, AppScreen::ConfirmQuit { .. }) {
        return handle_confirm_quit_key(app, key);
    }

    // Error screen — dismiss with Enter or Esc
    if matches!(app.screen, AppScreen::Error { .. }) {
        return handle_error_key(key);
    }

    // Pause screen — its own handler
    if matches!(app.screen, AppScreen::Pause { .. }) {
        return handle_pause_key(app, key);
    }

    // Status screen — dismiss with Esc or Tab
    if matches!(app.screen, AppScreen::Status { .. }) {
        return handle_status_key(key);
    }

    // Universal: Ctrl+Q — immediate quit on title, confirmation elsewhere
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
        return match &app.screen {
            AppScreen::Title => InputResult::Quit,
            _ => InputResult::RequestQuit,
        };
    }

    // Universal: Ctrl+S saves (when in scene)
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        if matches!(app.screen, AppScreen::Scene { .. }) {
            return InputResult::QuickSave;
        }
    }

    // Universal: Ctrl+P opens pause from Scene or Combat
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
        if matches!(app.screen, AppScreen::Scene { .. } | AppScreen::Combat) {
            return InputResult::OpenPause;
        }
    }

    match &app.screen {
        AppScreen::Title => handle_title_key(app, key),
        AppScreen::Scene { .. } => handle_scene_key(app, key),
        AppScreen::Standoff => handle_standoff_key(app, key),
        AppScreen::StandoffResult => handle_standoff_result_key(key),
        AppScreen::Combat => handle_combat_key(app, key),
        AppScreen::CombatOutcome => handle_combat_outcome_key(key),
        AppScreen::SaveLoad { .. } => handle_save_load_key(app, key),
        AppScreen::ConfirmQuit { .. } => unreachable!(), // handled above
        AppScreen::Error { .. } => unreachable!(),       // handled above
        AppScreen::Pause { .. } => unreachable!(),       // handled above
        AppScreen::Status { .. } => unreachable!(),      // handled above
    }
}

fn handle_title_key(_app: &mut App, key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Char('n') | KeyCode::Char('N') => InputResult::NewGame,
        KeyCode::Char('l') | KeyCode::Char('L') => InputResult::LoadScreen,
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => InputResult::Quit,
        _ => InputResult::None,
    }
}

fn handle_scene_key(app: &mut App, key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.reveal.all_complete {
                app.choice_cursor = app.choice_cursor.saturating_sub(1);
            }
            InputResult::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.reveal.all_complete {
                let max = app.choice_count().saturating_sub(1);
                if app.choice_cursor < max {
                    app.choice_cursor += 1;
                }
            }
            InputResult::None
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if !app.reveal.all_complete {
                let line_lengths = app.current_line_lengths();
                app.reveal.skip_line(&line_lengths);
                InputResult::None
            } else if app.choice_count() == 0 {
                InputResult::AdvanceScene
            } else {
                InputResult::ConfirmChoice(app.choice_cursor)
            }
        }
        KeyCode::Tab => InputResult::OpenStatus,
        KeyCode::Esc => InputResult::OpenPause,
        _ => InputResult::None,
    }
}

fn handle_standoff_key(app: &mut App, key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => InputResult::StandoffCyclePosture(-1),
        KeyCode::Down | KeyCode::Char('j') => InputResult::StandoffCyclePosture(1),
        KeyCode::Tab => InputResult::StandoffCycleFocus(1),
        KeyCode::BackTab => InputResult::StandoffCycleFocus(-1),
        KeyCode::Enter => InputResult::StandoffConfirm,
        KeyCode::Esc => InputResult::BackToTitle,
        _ => InputResult::None,
    }
}

fn handle_standoff_result_key(key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Enter | KeyCode::Char(' ') => InputResult::AdvanceCombat,
        _ => InputResult::None,
    }
}

fn handle_combat_key(_app: &mut App, key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => InputResult::CombatCycleAction(-1),
        KeyCode::Down | KeyCode::Char('j') => InputResult::CombatCycleAction(1),
        KeyCode::Tab => InputResult::CombatCycleTarget(1),
        KeyCode::BackTab => InputResult::CombatCycleTarget(-1),
        KeyCode::Enter => InputResult::CombatConfirmAction,
        KeyCode::Esc => InputResult::OpenPause,
        _ => InputResult::None,
    }
}

fn handle_combat_outcome_key(key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Enter | KeyCode::Char(' ') => InputResult::AdvanceCombat,
        _ => InputResult::None,
    }
}

fn handle_save_load_key(app: &mut App, key: KeyEvent) -> InputResult {
    // Delete confirmation sub-mode
    if let Some(slot_idx) = app.delete_confirming {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.delete_confirming = None;
                InputResult::ConfirmDeleteSave(slot_idx)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.delete_confirming = None;
                InputResult::CancelDeleteSave
            }
            _ => InputResult::None,
        };
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.save_cursor = app.save_cursor.saturating_sub(1);
            InputResult::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.save_cursor < SAVE_SLOT_COUNT.saturating_sub(1) {
                app.save_cursor += 1;
            }
            InputResult::None
        }
        KeyCode::Enter => InputResult::ConfirmSaveLoad(app.save_cursor),
        KeyCode::Char('d') | KeyCode::Char('D') => InputResult::RequestDeleteSave(app.save_cursor),
        KeyCode::Esc => InputResult::BackToTitle,
        _ => InputResult::None,
    }
}

fn handle_error_key(key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => InputResult::DismissError,
        _ => InputResult::None,
    }
}

fn handle_pause_key(app: &mut App, key: KeyEvent) -> InputResult {
    let options = PauseOption::all();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.pause_cursor = app.pause_cursor.saturating_sub(1);
            InputResult::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.pause_cursor < options.len().saturating_sub(1) {
                app.pause_cursor += 1;
            }
            InputResult::None
        }
        KeyCode::Enter => {
            let selected = options.get(app.pause_cursor).copied()
                .unwrap_or(PauseOption::Resume);
            InputResult::ConfirmPauseOption(selected)
        }
        KeyCode::Esc => InputResult::CancelPause,
        _ => InputResult::None,
    }
}

fn handle_status_key(key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Esc | KeyCode::Tab => InputResult::CloseStatus,
        _ => InputResult::None,
    }
}

fn handle_confirm_quit_key(app: &mut App, key: KeyEvent) -> InputResult {
    let options = QuitOption::all();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.quit_cursor = app.quit_cursor.saturating_sub(1);
            InputResult::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.quit_cursor < options.len().saturating_sub(1) {
                app.quit_cursor += 1;
            }
            InputResult::None
        }
        KeyCode::Enter => {
            let selected = options.get(app.quit_cursor).copied()
                .unwrap_or(QuitOption::Cancel);
            InputResult::ConfirmQuitOption(selected)
        }
        KeyCode::Esc => InputResult::CancelQuit,
        _ => InputResult::None,
    }
}
