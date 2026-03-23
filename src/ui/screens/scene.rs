//! Scene screen — narrative dialogue + choices.
//!
//! The most common screen. Dialogue lines revealed progressively.
//! Choice menu with age-phase identity. Memory echoes inline.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::scene::runner::{DisplayedLine, PreparedScene};
use crate::scene::types::MemoryRef;
use crate::state::types::MemoryObject;
use crate::types::AgePhase;
use crate::ui::theme;
use crate::ui::text_reveal::TextReveal;
use crate::ui::widgets::{dialogue, choice_menu, memory_echo};

/// Render the scene screen.
pub fn render_scene(
    frame: &mut Frame,
    area: Rect,
    prepared: &PreparedScene,
    reveal: &TextReveal,
    choice_cursor: usize,
    age_phase: AgePhase,
    chapter_label: &str,
    location_label: &str,
    memory_objects: &[MemoryObject],
) {
    let choice_count = prepared.choices.len();
    let choice_height = if reveal.all_complete && choice_count > 0 {
        choice_count as u16 + 3 // choices + border + title + padding
    } else {
        1 // just the border line, no choices yet
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top bar
            Constraint::Min(6),   // dialogue area
            Constraint::Length(choice_height),
        ])
        .split(area);

    // ─── Top bar ──────────────────────────────────────────────
    render_top_bar(frame, chunks[0], chapter_label, location_label, age_phase);

    // ─── Dialogue area ────────────────────────────────────────
    render_dialogue_area(
        frame,
        chunks[1],
        &prepared.lines,
        reveal,
        &prepared.memory_callbacks,
        memory_objects,
        prepared.pacing,
    );

    // ─── Choice menu ──────────────────────────────────────────
    let choice_list = choice_menu::render_choice_menu(
        &prepared.choices,
        choice_cursor,
        age_phase,
        reveal.all_complete,
    );
    frame.render_widget(choice_list, chunks[2]);
}

fn render_top_bar(
    frame: &mut Frame,
    area: Rect,
    chapter: &str,
    location: &str,
    age_phase: AgePhase,
) {
    let accent = theme::age_accent(age_phase);
    let label = theme::age_label(age_phase);

    let left = format!(" {} \u{2014} {}", chapter, location);
    let right = format!("{} ", label);
    let padding = area.width.saturating_sub(left.len() as u16 + right.len() as u16);

    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::White)),
        Span::raw(" ".repeat(padding as usize)),
        Span::styled(right, Style::default().fg(accent)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn render_dialogue_area(
    frame: &mut Frame,
    area: Rect,
    lines: &[DisplayedLine],
    reveal: &TextReveal,
    callbacks: &[MemoryRef],
    memory_objects: &[MemoryObject],
    pacing: crate::scene::types::PacingTag,
) {
    // Build the dialogue paragraph
    let dialogue_para = dialogue::render_dialogue(lines, reveal, area.height);

    // Memory echoes (appended after dialogue when all text is revealed)
    let echo_lines = if reveal.all_complete {
        memory_echo::render_echoes(callbacks, memory_objects)
    } else {
        Vec::new()
    };

    let border = theme::pacing_border(pacing);
    let block = match border {
        theme::PacingBorder::None => Block::default().borders(Borders::NONE),
        theme::PacingBorder::Plain => Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
        theme::PacingBorder::Double => Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(180, 80, 60))),
        theme::PacingBorder::Thick => Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    };

    // For now render dialogue into the block area
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(dialogue_para, inner);

    // Render echo lines below dialogue if space permits
    if !echo_lines.is_empty() {
        let echo_start = inner.y + inner.height.saturating_sub(echo_lines.len() as u16 + 1);
        if echo_start > inner.y {
            let echo_area = Rect::new(inner.x, echo_start, inner.width, echo_lines.len() as u16);
            let echo_para = Paragraph::new(echo_lines);
            frame.render_widget(echo_para, echo_area);
        }
    }
}

/// Render a scene that has no choices (end scene — just dialogue, then press any key).
pub fn render_end_scene(
    frame: &mut Frame,
    area: Rect,
    prepared: &PreparedScene,
    reveal: &TextReveal,
    age_phase: AgePhase,
    chapter_label: &str,
    location_label: &str,
    memory_objects: &[MemoryObject],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Length(2),
        ])
        .split(area);

    render_top_bar(frame, chunks[0], chapter_label, location_label, age_phase);
    render_dialogue_area(
        frame,
        chunks[1],
        &prepared.lines,
        reveal,
        &prepared.memory_callbacks,
        memory_objects,
        prepared.pacing,
    );

    // "Press any key to continue" hint
    if reveal.all_complete {
        let hint = Paragraph::new(Line::from(Span::styled(
            "  [Press Enter to continue]",
            theme::dim_style(),
        )));
        frame.render_widget(hint, chunks[2]);
    }
}
