//! Crowd pressure screen — rising room temperature.
//!
//! The crowd is not enemies with a shared HP bar. It is a room-state
//! the party manages through different vectors. Victory is containment.
//! The emotional grammar is heat — momentum, surge, breaking point.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};

use crate::combat::crowd::*;
use crate::ui::theme;

/// Crowd-specific UI state.
#[derive(Debug)]
pub struct CrowdUi {
    pub action_cursor: usize,
    pub target_cursor: usize,
    pub log: Vec<CrowdLogEntry>,
}

#[derive(Debug, Clone)]
pub struct CrowdLogEntry {
    pub text: String,
    pub style: Style,
}

impl CrowdUi {
    pub fn new() -> Self {
        Self { action_cursor: 0, target_cursor: 0, log: Vec::new() }
    }

    pub fn push_result(&mut self, result: &CrowdActionResult) {
        self.log.push(CrowdLogEntry {
            text: format!("  {}", result.description),
            style: Style::default().fg(Color::White),
        });

        if let Some(ref name) = result.ringleader_broken {
            self.log.push(CrowdLogEntry {
                text: format!("  {} BREAKS. The crowd feels it.", name),
                style: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            });
        }

        if result.surge_delayed {
            self.log.push(CrowdLogEntry {
                text: "  The surge is delayed. The line holds.".to_string(),
                style: Style::default().fg(Color::Yellow),
            });
        }

        while self.log.len() > 6 { self.log.remove(0); }
    }
}

/// Render the crowd pressure screen.
pub fn render_crowd(
    frame: &mut Frame,
    area: Rect,
    crowd: &CrowdState,
    ui: &CrowdUi,
    actions: &[CrowdMenuItem],
) {
    let ringleader_count = crowd.ringleaders.len() as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),              // title + phase
            Constraint::Length(4),              // nerve / momentum / countdown
            Constraint::Length(1),              // separator
            Constraint::Length(ringleader_count + 2), // ringleaders
            Constraint::Length(1),              // separator
            Constraint::Min(2),                // log
            Constraint::Length(1),              // separator
            Constraint::Min(5),                // action menu
        ])
        .split(area);

    render_crowd_title(frame, chunks[0], crowd);
    render_crowd_dashboard(frame, chunks[1], crowd);
    render_separator(frame, chunks[2]);
    render_ringleaders(frame, chunks[3], crowd, ui.target_cursor);
    render_separator(frame, chunks[4]);
    render_crowd_log(frame, chunks[5], ui);
    render_separator(frame, chunks[6]);
    render_crowd_actions(frame, chunks[7], actions, ui);
}

fn render_crowd_title(frame: &mut Frame, area: Rect, crowd: &CrowdState) {
    let phase_str = match crowd.phase {
        CrowdPhase::Tense => "TENSE",
        CrowdPhase::Surging => "SURGING",
        CrowdPhase::Calming => "CALMING",
        CrowdPhase::Broken => "BROKEN",
        CrowdPhase::Dispersed => "DISPERSED",
    };
    let phase_color = match crowd.phase {
        CrowdPhase::Tense => Color::Yellow,
        CrowdPhase::Surging => Color::Red,
        CrowdPhase::Calming => Color::Green,
        CrowdPhase::Broken => Color::Magenta,
        CrowdPhase::Dispersed => Color::Rgb(100, 180, 100),
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" CROWD PRESSURE \u{2014} Turn {}", crowd.turn),
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

fn render_crowd_dashboard(frame: &mut Frame, area: Rect, crowd: &CrowdState) {
    let bar = crate::ui::screens::combat::bar_chars(crowd.collective_nerve, crowd.max_nerve, 24);
    let nerve_color = theme::gauge_color(crowd.collective_nerve, crowd.max_nerve);

    // Momentum direction arrows
    let momentum_display = if crowd.momentum > 0 {
        format!("\u{25ba}\u{25ba} +{} (calming)", crowd.momentum)
    } else if crowd.momentum < 0 {
        format!("\u{25c4}\u{25c4} {} (escalating)", crowd.momentum)
    } else {
        "-- 0 (stalled)".to_string()
    };
    let momentum_color = if crowd.momentum > 0 {
        Color::Green
    } else if crowd.momentum < 0 {
        Color::Red
    } else {
        Color::Yellow
    };

    // Surge countdown
    let surge_color = if crowd.surge_countdown <= 2 {
        Color::Red
    } else {
        Color::Yellow
    };
    let surge_bar = crate::ui::screens::combat::bar_chars(
        crowd.surge_countdown as i32,
        8, // approximate max
        8,
    );

    let lines = vec![
        Line::from(vec![
            Span::styled("  Collective Nerve  ", Style::default().fg(Color::White)),
            Span::styled(bar, Style::default().fg(nerve_color)),
            Span::styled(
                format!("  {}/{}", crowd.collective_nerve, crowd.max_nerve),
                Style::default().fg(nerve_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Momentum          ", Style::default().fg(Color::White)),
            Span::styled(momentum_display, Style::default().fg(momentum_color)),
        ]),
        Line::from(vec![
            Span::styled("  Surge Countdown   ", Style::default().fg(Color::White)),
            Span::styled(surge_bar, Style::default().fg(surge_color)),
            Span::styled(
                format!("  {} turns", crowd.surge_countdown),
                Style::default().fg(surge_color),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_ringleaders(frame: &mut Frame, area: Rect, crowd: &CrowdState, target_cursor: usize) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Ringleaders ", theme::dim_style()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = crowd.ringleaders.iter().enumerate().map(|(i, rl)| {
        let is_target = i == target_cursor;
        let prefix = if is_target { " \u{25b6} " } else { "   " };

        if rl.broken {
            Line::from(vec![
                Span::styled(prefix, theme::dim_style()),
                Span::styled(
                    format!("{:<14} BROKEN", rl.name),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                ),
            ])
        } else {
            let nerve_bar = crate::ui::screens::combat::bar_chars(rl.nerve, 20, 6);
            let nerve_color = theme::gauge_color(rl.nerve, 20);

            Line::from(vec![
                Span::styled(prefix, if is_target { Style::default().fg(Color::Yellow) } else { Style::default() }),
                Span::styled(format!("{:<14} ", rl.name), Style::default().fg(Color::White)),
                Span::styled("NRV ", Style::default().fg(nerve_color)),
                Span::styled(nerve_bar, Style::default().fg(nerve_color)),
                Span::styled(
                    format!("   influence: {}", rl.influence),
                    theme::dim_style(),
                ),
            ])
        }
    }).collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_crowd_log(frame: &mut Frame, area: Rect, ui: &CrowdUi) {
    let max = area.height as usize;
    let start = ui.log.len().saturating_sub(max);
    let lines: Vec<Line> = ui.log[start..].iter().map(|e| {
        Line::from(Span::styled(e.text.clone(), e.style))
    }).collect();
    frame.render_widget(Paragraph::new(lines), area);
}

/// A crowd action menu item.
#[derive(Debug, Clone)]
pub struct CrowdMenuItem {
    pub actor: String,
    pub label: String,
    pub action_type: CrowdActionType,
    pub effect_hint: String,
    pub needs_target: bool,
}

fn render_crowd_actions(
    frame: &mut Frame,
    area: Rect,
    actions: &[CrowdMenuItem],
    ui: &CrowdUi,
) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(180, 80, 60))) // heat accent
        .title(Span::styled("  The room is waiting.  ", Style::default().fg(Color::Rgb(180, 80, 60))));

    let items: Vec<ListItem> = actions.iter().enumerate().map(|(i, action)| {
        let selected = i == ui.action_cursor;
        let prefix = if selected { " > " } else { "   " };

        let style = if selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        ListItem::new(Line::from(vec![
            Span::styled(
                format!("{}{:<10}\u{2014} {:<24}",
                    prefix,
                    action.actor.to_uppercase(),
                    action.label,
                ),
                style,
            ),
            Span::styled(action.effect_hint.clone(), theme::dim_style()),
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
