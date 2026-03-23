//! Public Reckoning screen — truth under compression.
//!
//! Five bars tracked simultaneously. The room is a battlefield made of
//! testimony and timing. Sequence matters. Who speaks when changes
//! what the room believes. Eli's defining act is the hinge.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::combat::reckoning::*;
use crate::ui::theme;

/// Reckoning-specific UI state.
#[derive(Debug)]
pub struct ReckoningUi {
    pub action_cursor: usize,
    pub log: Vec<ReckoningLogEntry>,
}

#[derive(Debug, Clone)]
pub struct ReckoningLogEntry {
    pub text: String,
    pub style: Style,
}

impl ReckoningUi {
    pub fn new() -> Self {
        Self { action_cursor: 0, log: Vec::new() }
    }

    pub fn push_action(&mut self, action: &ReckoningAction) {
        self.log.push(ReckoningLogEntry {
            text: format!("  {}", action.description),
            style: Style::default().fg(Color::White),
        });

        // Highlight bar changes
        let e = &action.effects;
        let changes = [
            ("Credibility", e.credibility),
            ("Crowd", e.crowd_nerve),
            ("Witness", e.witness_integrity),
            ("Evidence", e.evidence_continuity),
            ("Procedure", e.procedural_control),
        ];
        let significant: Vec<String> = changes.iter()
            .filter(|(_, v)| v.abs() >= 5)
            .map(|(name, v)| {
                if *v > 0 { format!("{} +{}", name, v) }
                else { format!("{} {}", name, v) }
            })
            .collect();

        if !significant.is_empty() {
            self.log.push(ReckoningLogEntry {
                text: format!("  {}", significant.join(" | ")),
                style: Style::default().fg(Color::Rgb(180, 180, 140)),
            });
        }

        // Eli's defining act gets special treatment
        if action.action_type == ReckoningActionType::EliDefiningAct {
            self.log.push(ReckoningLogEntry {
                text: "  ** The room goes silent. **".to_string(),
                style: Style::default().fg(Color::Rgb(200, 180, 140)).add_modifier(Modifier::BOLD),
            });
        }

        while self.log.len() > 8 { self.log.remove(0); }
    }
}

/// A reckoning action menu item.
#[derive(Debug, Clone)]
pub struct ReckoningMenuItem {
    pub actor: String,
    pub label: String,
    pub action_type: ReckoningActionType,
    pub bar_effects: String,
    pub is_eli_act: bool,
    pub available: bool,
    pub lock_reason: Option<String>,
}

/// Render the public reckoning screen.
pub fn render_reckoning(
    frame: &mut Frame,
    area: Rect,
    state: &ReckoningState,
    ui: &ReckoningUi,
    actions: &[ReckoningMenuItem],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // title + phase
            Constraint::Length(7),  // five bars
            Constraint::Length(1),  // separator
            Constraint::Min(3),    // log
            Constraint::Length(1),  // separator
            Constraint::Min(5),    // action menu
        ])
        .split(area);

    render_reckoning_title(frame, chunks[0], state);
    render_five_bars(frame, chunks[1], state);
    render_separator(frame, chunks[2]);
    render_reckoning_log(frame, chunks[3], ui);
    render_separator(frame, chunks[4]);
    render_reckoning_actions(frame, chunks[5], actions, ui, state);
}

fn render_reckoning_title(frame: &mut Frame, area: Rect, state: &ReckoningState) {
    let phase_str = match state.phase {
        ReckoningPhase::Opening => "OPENING",
        ReckoningPhase::Presentation => "PRESENTATION",
        ReckoningPhase::Counterstrike => "COUNTERSTRIKE",
        ReckoningPhase::EliAct => "ELI'S ACT",
        ReckoningPhase::Verdict => "VERDICT",
    };
    let phase_color = match state.phase {
        ReckoningPhase::Opening => Color::White,
        ReckoningPhase::Presentation => Color::Rgb(200, 180, 140),
        ReckoningPhase::Counterstrike => Color::Red,
        ReckoningPhase::EliAct => Color::Rgb(200, 180, 140),
        ReckoningPhase::Verdict => Color::Green,
    };

    let line = Line::from(vec![
        Span::styled(
            " PUBLIC RECKONING \u{2014} Deadwater Trial",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            format!("Phase: {}", phase_str),
            Style::default().fg(phase_color).add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(vec![line, Line::from("")]), area);
}

fn render_five_bars(frame: &mut Frame, area: Rect, state: &ReckoningState) {
    let bars = [
        ("Room Credibility   ", state.room_credibility),
        ("Crowd Nerve        ", state.crowd_nerve),
        ("Witness Integrity  ", state.witness_integrity),
        ("Evidence Continuity", state.evidence_continuity),
        ("Procedural Control ", state.procedural_control),
    ];

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" The Room ", Style::default().fg(Color::Rgb(200, 180, 140))));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = bars.iter().map(|(label, value)| {
        let color = reckoning_bar_color(*value);
        let bar = crate::ui::screens::combat::bar_chars(*value, 100, 20);
        let danger = *value <= 15;
        let mark = if danger { " \u{26a0}" } else { "" };

        Line::from(vec![
            Span::styled(format!("  {}", label), Style::default().fg(Color::White)),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(
                format!("  {}/100{}", value, mark),
                Style::default().fg(color),
            ),
        ])
    }).collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Reckoning bar colors — truth-themed, not combat-themed.
fn reckoning_bar_color(value: i32) -> Color {
    if value >= 60 {
        Color::Rgb(140, 180, 140)  // steady green — truth holding
    } else if value >= 30 {
        Color::Rgb(200, 180, 100)  // amber — under pressure
    } else if value > 10 {
        Color::Rgb(200, 100, 80)   // danger red — close to collapse
    } else {
        Color::Rgb(180, 60, 60)    // critical — bar near failure
    }
}

fn render_reckoning_log(frame: &mut Frame, area: Rect, ui: &ReckoningUi) {
    let max = area.height as usize;
    let start = ui.log.len().saturating_sub(max);
    let lines: Vec<Line> = ui.log[start..].iter().map(|e| {
        Line::from(Span::styled(e.text.clone(), e.style))
    }).collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_reckoning_actions(
    frame: &mut Frame,
    area: Rect,
    actions: &[ReckoningMenuItem],
    ui: &ReckoningUi,
    state: &ReckoningState,
) {
    // Reckoning menu language — testimony and timing, not combat
    let title_text = match state.phase {
        ReckoningPhase::Opening => "  The room listens.  ",
        ReckoningPhase::Presentation => "  Present your case.  ",
        ReckoningPhase::Counterstrike => "  They're pushing back.  ",
        ReckoningPhase::EliAct => "  One choice remains.  ",
        ReckoningPhase::Verdict => "  The room decides.  ",
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(200, 180, 140)))
        .title(Span::styled(title_text, Style::default().fg(Color::Rgb(200, 180, 140))));

    let items: Vec<ListItem> = actions.iter().enumerate().map(|(i, action)| {
        let selected = i == ui.action_cursor;
        let prefix = if selected { " > " } else { "   " };

        if !action.available {
            let reason = action.lock_reason.as_deref().unwrap_or("[Not yet]");
            return ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}{:<10}\u{2014} {}", prefix, action.actor.to_uppercase(), action.label),
                    theme::locked_style(),
                ),
                Span::styled(format!("  {}", reason), theme::lock_reason_style()),
            ]));
        }

        // Eli's defining act gets special rendering
        if action.is_eli_act {
            let style = if selected {
                Style::default().fg(Color::Rgb(220, 200, 160)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(200, 180, 140))
            };
            return ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}{:<10}\u{2014} {}", prefix, action.actor.to_uppercase(), action.label),
                    style,
                ),
                Span::styled(
                    "  ** DEFINING ACT **",
                    Style::default().fg(Color::Rgb(220, 200, 160)).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        let style = if selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        ListItem::new(Line::from(vec![
            Span::styled(
                format!("{}{:<10}\u{2014} {:<28}", prefix, action.actor.to_uppercase(), action.label),
                style,
            ),
            Span::styled(action.bar_effects.clone(), theme::dim_style()),
        ]))
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
