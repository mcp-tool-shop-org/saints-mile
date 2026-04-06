//! Input handling — crossterm event dispatch by screen.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::ui::mod_types::{App, AppScreen, InputResult};

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
    // Universal: Ctrl+Q quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
        return InputResult::Quit;
    }

    // Universal: Ctrl+S saves (when in scene)
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        if matches!(app.screen, AppScreen::Scene { .. }) {
            return InputResult::QuickSave;
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
        KeyCode::Esc => InputResult::BackToTitle,
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

fn handle_combat_key(app: &mut App, key: KeyEvent) -> InputResult {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => InputResult::CombatCycleAction(-1),
        KeyCode::Down | KeyCode::Char('j') => InputResult::CombatCycleAction(1),
        KeyCode::Tab => InputResult::CombatCycleTarget(1),
        KeyCode::BackTab => InputResult::CombatCycleTarget(-1),
        KeyCode::Enter => InputResult::CombatConfirmAction,
        KeyCode::Esc => InputResult::BackToTitle,
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
        KeyCode::Esc => InputResult::BackToTitle,
        _ => InputResult::None,
    }
}
