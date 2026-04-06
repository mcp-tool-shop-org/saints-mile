//! Saint's Mile — A frontier JRPG for the adults who loved those games first.
//!
//! Terminal entry point: setup, app loop, teardown.

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use saints_mile::ui::{App, AppScreen, InputResult, QuitOption};
use saints_mile::ui::input::handle_event;
use saints_mile::ui::screens::{title, scene, standoff, combat, save_load};
use saints_mile::ui::screens::save_load::{SaveLoadMode, SaveSlotInfo};

const VERSION: &str = env!("CARGO_PKG_VERSION");

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<()> {
    if let Some(arg) = std::env::args().nth(1) {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("saints-mile {}", VERSION);
                return Ok(());
            }
            "--help" | "-h" => {
                println!("saints-mile v{}\n", VERSION);
                println!("A frontier JRPG for the adults who loved those games first.\n");
                println!("USAGE:");
                println!("  saints-mile              Start the game");
                println!("  saints-mile --version    Print version and exit");
                println!("  saints-mile --help       Show this help and exit");
                return Ok(());
            }
            _ => {}
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let save_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("saves");

    let mut app = App::new(save_dir);
    let tick_rate = Duration::from_millis(50);
    let result = run_loop(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(terminal: &mut Term, app: &mut App, tick_rate: Duration) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app))?;

        if event::poll(tick_rate)? {
            let ev = event::read()?;
            let result = handle_event(app, ev);
            process_result(app, result);
        }

        app.tick();

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    match &app.screen {
        AppScreen::Title => {
            title::render_title(frame, area);
        }
        AppScreen::Scene { chapter_label, location_label } => {
            if let Some(prepared) = &app.current_prepared {
                if prepared.choices.is_empty() {
                    scene::render_end_scene(
                        frame, area, prepared, &app.reveal,
                        app.age_phase(), chapter_label, location_label,
                        app.memory_objects(),
                    );
                } else {
                    scene::render_scene(
                        frame, area, prepared, &app.reveal,
                        app.choice_cursor, app.age_phase(),
                        chapter_label, location_label, app.memory_objects(),
                    );
                }
            }
        }
        AppScreen::Standoff => {
            if let (Some(state), Some(ui)) = (&app.encounter_state, &app.standoff_ui) {
                let terrain = app.encounter_def.as_ref()
                    .map(|e| e.terrain.name.as_str())
                    .unwrap_or("Unknown");
                standoff::render_standoff(frame, area, state, ui, terrain);
            }
        }
        AppScreen::StandoffResult => {
            if let Some(state) = &app.encounter_state {
                let posture = app.combat_ui.standoff_posture
                    .unwrap_or(saints_mile::combat::types::StandoffPosture::SteadyHand);
                standoff::render_standoff_result(frame, area, state, posture);
            }
        }
        AppScreen::Combat => {
            if let Some(state) = &app.encounter_state {
                combat::render_combat(
                    frame, area, state, &app.combat_ui,
                    app.age_phase(), &app.combat_actions,
                );
            }
        }
        AppScreen::CombatOutcome => {
            render_combat_outcome(frame, area, app);
        }
        AppScreen::SaveLoad { mode } => {
            let slots = discover_save_slots();
            save_load::render_save_load(frame, area, *mode, &slots, app.save_cursor);
        }
        AppScreen::ConfirmQuit { .. } => {
            render_confirm_quit(frame, area, app.quit_cursor);
        }
    }
}

fn render_combat_outcome(frame: &mut Frame, area: Rect, app: &App) {
    use ratatui::widgets::Paragraph;
    use saints_mile::combat::engine::EncounterResult;
    use saints_mile::ui::theme;

    let outcome = app.encounter_state.as_ref()
        .and_then(|s| s.outcome.as_ref());

    let mut lines = vec![
        Line::from(""),
        Line::from(""),
    ];

    if let Some(outcome) = outcome {
        let (label, color) = match outcome.result {
            EncounterResult::Victory => ("VICTORY", Color::Green),
            EncounterResult::Defeat => ("DEFEAT", Color::Red),
            EncounterResult::Fled => ("FLED", Color::Yellow),
            EncounterResult::ObjectiveComplete => ("OBJECTIVE COMPLETE", Color::Green),
        };

        lines.push(Line::from(Span::styled(
            format!("  {}", label),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Objective summary
        if let Some(state) = &app.encounter_state {
            for obj in &state.objectives {
                let (icon, obj_color) = match obj.status {
                    saints_mile::combat::engine::ObjectiveStatus::Active => ("[ ]", Color::White),
                    saints_mile::combat::engine::ObjectiveStatus::Succeeded => ("[x]", Color::Green),
                    saints_mile::combat::engine::ObjectiveStatus::Failed => ("[\u{2717}]", Color::Red),
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", icon), Style::default().fg(obj_color)),
                    Span::styled(&obj.label, Style::default().fg(obj_color)),
                ]));
            }
        }

        // Last combat log entries
        lines.push(Line::from(""));
        let log_start = app.combat_ui.log.len().saturating_sub(4);
        for entry in &app.combat_ui.log[log_start..] {
            lines.push(Line::from(Span::styled(entry.text.clone(), entry.style)));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Enter] Continue",
        theme::dim_style(),
    )));

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn render_confirm_quit(frame: &mut Frame, area: Rect, cursor: usize) {
    use ratatui::widgets::Paragraph;
    use saints_mile::ui::theme;

    let options = QuitOption::all();
    let mut lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "  QUIT GAME",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  You have unsaved progress. What would you like to do?",
            Style::default().fg(Color::Rgb(160, 150, 130)),
        )),
        Line::from(""),
    ];

    for (i, option) in options.iter().enumerate() {
        let marker = if i == cursor { "> " } else { "  " };
        let color = if i == cursor { Color::White } else { Color::DarkGray };
        lines.push(Line::from(Span::styled(
            format!("  {} {}", marker, option.label()),
            Style::default().fg(color),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Esc] Cancel",
        theme::dim_style(),
    )));

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn process_result(app: &mut App, result: InputResult) {
    match result {
        InputResult::None | InputResult::Redraw => {}
        InputResult::Quit => app.should_quit = true,
        InputResult::NewGame => app.new_game(),
        InputResult::LoadScreen => {
            app.save_cursor = 0;
            app.screen = AppScreen::SaveLoad { mode: SaveLoadMode::Load };
        }
        InputResult::BackToTitle => app.screen = AppScreen::Title,
        InputResult::QuickSave => app.quick_save(),
        InputResult::AdvanceScene => app.advance_no_choice_scene(),
        InputResult::ConfirmChoice(idx) => app.execute_choice(idx),
        InputResult::ConfirmSaveLoad(idx) => handle_save_load(app, idx),

        // Standoff input
        InputResult::StandoffCyclePosture(dir) => {
            if let Some(ui) = &mut app.standoff_ui {
                let max = ui.postures.len();
                if dir > 0 && ui.posture_cursor < max - 1 {
                    ui.posture_cursor += 1;
                } else if dir < 0 && ui.posture_cursor > 0 {
                    ui.posture_cursor -= 1;
                }
            }
        }
        InputResult::StandoffCycleFocus(dir) => {
            if let Some(ui) = &mut app.standoff_ui {
                let max = ui.enemy_count;
                if dir > 0 {
                    ui.focus_cursor = (ui.focus_cursor + 1) % max;
                } else if ui.focus_cursor > 0 {
                    ui.focus_cursor -= 1;
                } else {
                    ui.focus_cursor = max.saturating_sub(1);
                }
            }
        }
        InputResult::StandoffConfirm => app.resolve_standoff(),

        // Combat input
        InputResult::CombatCycleAction(dir) => {
            let max = app.combat_actions.len().saturating_sub(1);
            if dir > 0 && app.combat_ui.action_cursor < max {
                app.combat_ui.action_cursor += 1;
            } else if dir < 0 && app.combat_ui.action_cursor > 0 {
                app.combat_ui.action_cursor -= 1;
            }
        }
        InputResult::CombatCycleTarget(dir) => {
            let max = app.living_enemy_count().saturating_sub(1);
            if dir > 0 && app.combat_ui.target_cursor < max {
                app.combat_ui.target_cursor += 1;
            } else if dir < 0 && app.combat_ui.target_cursor > 0 {
                app.combat_ui.target_cursor -= 1;
            }
        }
        InputResult::CombatConfirmAction => app.execute_combat_action(),

        // Post-standoff / post-combat
        InputResult::AdvanceCombat => {
            match &app.screen {
                AppScreen::StandoffResult => app.begin_combat(),
                AppScreen::CombatOutcome => app.exit_combat(),
                _ => {}
            }
        }

        // Quit confirmation flow
        InputResult::RequestQuit => {
            // Swap the current screen into the return_screen box
            let current = std::mem::replace(&mut app.screen, AppScreen::Title);
            app.quit_cursor = 0;
            app.screen = AppScreen::ConfirmQuit {
                return_screen: Box::new(current),
            };
        }
        InputResult::ConfirmQuitOption(option) => {
            match option {
                QuitOption::SaveAndQuit => {
                    app.quick_save();
                    app.should_quit = true;
                }
                QuitOption::QuitWithoutSaving => {
                    app.should_quit = true;
                }
                QuitOption::Cancel => {
                    // Restore the screen we came from
                    let screen = std::mem::replace(&mut app.screen, AppScreen::Title);
                    if let AppScreen::ConfirmQuit { return_screen } = screen {
                        app.screen = *return_screen;
                    }
                }
            }
        }
        InputResult::CancelQuit => {
            let screen = std::mem::replace(&mut app.screen, AppScreen::Title);
            if let AppScreen::ConfirmQuit { return_screen } = screen {
                app.screen = *return_screen;
            }
        }
    }
}

fn handle_save_load(app: &mut App, slot_index: usize) {
    let slots = discover_save_slots();
    if let Some(slot) = slots.get(slot_index) {
        match &app.screen {
            AppScreen::SaveLoad { mode: SaveLoadMode::Save } => {
                let _ = app.store.save(&slot.name);
                app.screen = AppScreen::Title;
            }
            AppScreen::SaveLoad { mode: SaveLoadMode::Load } => {
                let path = app.save_dir().join(format!("{}.ron", slot.name));
                if path.exists() {
                    if let Ok(loaded) = saints_mile::state::store::StateStore::load(&path) {
                        app.store = loaded;
                        // Clone required: load_scene() borrows &mut self,
                        // so we can't hold a reference into app.store.
                        let beat = app.store.state().beat.0.clone();
                        app.load_scene(&beat);
                    }
                }
            }
            _ => {}
        }
    }
}

fn discover_save_slots() -> Vec<SaveSlotInfo> {
    let save_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("saves");

    (1..=3)
        .map(|i| {
            let name = format!("slot{}", i);
            // Validate slot name contains only safe characters
            if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                return SaveSlotInfo { name, label: String::new(), exists: false };
            }
            let path = save_dir.join(format!("{}.ron", name));
            // Validate resolved path is within the save directory
            if let (Ok(canonical_dir), Ok(canonical_path)) = (
                std::fs::canonicalize(&save_dir),
                std::fs::canonicalize(&path),
            ) {
                if !canonical_path.starts_with(&canonical_dir) {
                    return SaveSlotInfo { name, label: String::new(), exists: false };
                }
            }
            let (exists, label) = if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(contents) => {
                        if let Ok(envelope) = ron::from_str::<saints_mile::state::store::SaveEnvelope>(&contents) {
                            (true, envelope.label)
                        } else {
                            (true, "corrupted save".to_string())
                        }
                    }
                    Err(_) => (false, String::new()),
                }
            } else {
                (false, String::new())
            };
            SaveSlotInfo { name, label, exists }
        })
        .collect()
}
