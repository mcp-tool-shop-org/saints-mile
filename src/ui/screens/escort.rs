//! Escort pressure screen — fragile motion.
//!
//! The convoy is moving. Cargo is taking damage. Protection priorities shift.
//! The emotional grammar is fragility in transit — you can't stop to fix things,
//! you can only decide what to sacrifice next.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};

use crate::pressure::types::*;
use crate::ui::theme;

/// Escort-specific UI state.
#[derive(Debug)]
pub struct EscortUi {
    pub action_cursor: usize,
    pub turn: u32,
    pub log: Vec<EscortLogEntry>,
}

#[derive(Debug, Clone)]
pub struct EscortLogEntry {
    pub text: String,
    pub style: Style,
}

impl EscortUi {
    pub fn new() -> Self {
        Self { action_cursor: 0, turn: 0, log: Vec::new() }
    }

    pub fn push_log(&mut self, text: String, style: Style) {
        self.log.push(EscortLogEntry { text, style });
        while self.log.len() > 6 { self.log.remove(0); }
    }
}

/// Render the escort pressure screen.
pub fn render_escort(
    frame: &mut Frame,
    area: Rect,
    encounter: &PressureEncounter,
    ui: &EscortUi,
    cargo: &[CargoItem],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // title
            Constraint::Length(cargo.len() as u16 + 2), // cargo status
            Constraint::Length(1),  // separator
            Constraint::Length(encounter.pressure_bars.iter().filter(|b| b.visible).count() as u16 + 2), // pressure bars
            Constraint::Length(1),  // separator
            Constraint::Min(3),    // log
            Constraint::Length(1),  // separator
            Constraint::Min(5),    // action menu
        ])
        .split(area);

    // Title — fragile motion language
    render_escort_title(frame, chunks[0], ui.turn);

    // Cargo status — each item with its integrity
    render_cargo_status(frame, chunks[1], cargo);

    render_separator(frame, chunks[2]);

    // Route pressure bars
    render_pressure_bars(frame, chunks[3], &encounter.pressure_bars);

    render_separator(frame, chunks[4]);

    // Event log
    render_escort_log(frame, chunks[5], ui);

    render_separator(frame, chunks[6]);

    // Action menu
    render_escort_actions(frame, chunks[7], encounter, ui);
}

fn render_escort_title(frame: &mut Frame, area: Rect, turn: u32) {
    let line = Line::from(vec![
        Span::styled(
            " ESCORT",
            Style::default().fg(Color::Rgb(200, 180, 140)).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" \u{2014} Leg {}", turn + 1),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(vec![line, Line::from("")]), area);
}

fn render_cargo_status(frame: &mut Frame, area: Rect, cargo: &[CargoItem]) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Cargo ", Style::default().fg(Color::Rgb(200, 180, 140))));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = cargo.iter().map(|item| {
        let ratio = if item.max_integrity > 0 {
            (item.integrity as f64 / item.max_integrity as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let pct = (ratio * 100.0) as i32;

        let color = if pct > 60 {
            Color::Green
        } else if pct > 30 {
            Color::Yellow
        } else if pct > 0 {
            Color::Red
        } else {
            Color::DarkGray
        };

        let status = if item.integrity <= 0 {
            "LOST".to_string()
        } else {
            format!("{}/{}", item.integrity, item.max_integrity)
        };

        let bar = crate::ui::screens::combat::bar_chars(item.integrity, item.max_integrity, 12);

        Line::from(vec![
            Span::styled(format!(" {:<20}", item.name), Style::default().fg(Color::White)),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(format!(" {}", status), Style::default().fg(color)),
        ])
    }).collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_pressure_bars(frame: &mut Frame, area: Rect, bars: &[PressureBar]) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Route Status ", theme::dim_style()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible: Vec<&PressureBar> = bars.iter().filter(|b| b.visible).collect();
    let lines: Vec<Line> = visible.iter().map(|bar| {
        let color = theme::gauge_color(bar.current, bar.max);
        let danger = bar.current <= bar.fail_at;
        let display_color = if danger { Color::Red } else { color };

        let bar_str = crate::ui::screens::combat::bar_chars(bar.current, bar.max, 16);
        let danger_mark = if danger { " !" } else { "" };

        Line::from(vec![
            Span::styled(format!(" {:<18}", bar.label), Style::default().fg(Color::White)),
            Span::styled(bar_str, Style::default().fg(display_color)),
            Span::styled(
                format!(" {}/{}{}", bar.current, bar.max, danger_mark),
                Style::default().fg(display_color),
            ),
        ])
    }).collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_escort_log(frame: &mut Frame, area: Rect, ui: &EscortUi) {
    let max = area.height as usize;
    let start = ui.log.len().saturating_sub(max);
    let lines: Vec<Line> = ui.log[start..].iter().map(|e| {
        Line::from(Span::styled(e.text.clone(), e.style))
    }).collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_escort_actions(
    frame: &mut Frame,
    area: Rect,
    encounter: &PressureEncounter,
    ui: &EscortUi,
) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(200, 180, 140)))
        .title(Span::styled("  Protect what you can.  ", Style::default().fg(Color::Rgb(200, 180, 140))));

    let items: Vec<ListItem> = encounter.party_actions.iter().flat_map(|pa| {
        pa.actions.iter().enumerate().map(move |(i, action)| {
            let global_idx = encounter.party_actions.iter()
                .take_while(|p| p.character != pa.character)
                .map(|p| p.actions.len())
                .sum::<usize>() + i;

            let selected = global_idx == ui.action_cursor;
            let prefix = if selected { " > " } else { "   " };

            let char_label = humanize_id(&pa.character.0);
            let effect_hint = if action.delta > 0 {
                format!("+{} {}", action.delta, action.target_bar)
            } else {
                format!("{} {}", action.delta, action.target_bar)
            };

            let style = if selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{}{:<10}", prefix, char_label), style),
                Span::styled(
                    format!("\u{2014} {:<24}", action.label),
                    if selected { style } else { Style::default().fg(Color::Rgb(180, 180, 180)) },
                ),
                Span::styled(effect_hint, theme::dim_style()),
            ]))
        })
    }).collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new(Line::from(Span::styled(
        "\u{2500}".repeat(area.width as usize),
        Style::default().fg(Color::Rgb(60, 60, 60)),
    )));
    frame.render_widget(sep, area);
}

fn humanize_id(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let rest: String = chars.collect();
            format!("{}{}", first.to_uppercase(), rest)
        }
    }
}
