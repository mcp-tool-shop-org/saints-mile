//! Standoff screen — the first moment where the player reads tradeoff under pressure.
//!
//! This is a combat phase, not a menu popup. It must feel like the opening of a fight:
//! posture choice, enemy nerve visible, focus target selectable, stakes readable.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};

use crate::combat::engine::{EncounterState, EncounterPhase, LiveCombatant};
use crate::combat::types::StandoffPosture;
use crate::ui::theme;

/// Standoff UI state tracked by the App.
#[derive(Debug)]
pub struct StandoffUi {
    pub posture_cursor: usize,
    pub focus_cursor: usize,
    pub postures: Vec<StandoffPosture>,
    pub enemy_count: usize,
}

impl StandoffUi {
    pub fn new(postures: Vec<StandoffPosture>, enemy_count: usize) -> Self {
        Self {
            posture_cursor: 1, // default to SteadyHand (balanced)
            focus_cursor: 0,
            postures,
            enemy_count,
        }
    }

    pub fn selected_posture(&self) -> StandoffPosture {
        self.postures.get(self.posture_cursor).copied()
            .unwrap_or(StandoffPosture::SteadyHand)
    }
}

/// Render the standoff screen.
pub fn render_standoff(
    frame: &mut Frame,
    area: Rect,
    encounter: &EncounterState,
    ui: &StandoffUi,
    terrain_name: &str,
) {
    let enemy_row_height = 4u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),                        // title bar
            Constraint::Length(enemy_row_height + 2),     // enemy display
            Constraint::Length(1),                        // separator
            Constraint::Min(8),                          // posture menu + focus
        ])
        .split(area);

    // ─── Title bar ────────────────────────────────────────────
    render_standoff_title(frame, chunks[0], terrain_name);

    // ─── Enemy nerve display ──────────────────────────────────
    render_enemy_readout(frame, chunks[1], &encounter.enemies, ui.focus_cursor);

    // ─── Separator ────────────────────────────────────────────
    let sep = Paragraph::new(Line::from(Span::styled(
        "\u{2500}".repeat(area.width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(sep, chunks[2]);

    // ─── Posture menu + focus selector ────────────────────────
    render_posture_menu(frame, chunks[3], ui, &encounter.enemies);
}

fn render_standoff_title(frame: &mut Frame, area: Rect, terrain_name: &str) {
    let left = " STANDOFF";
    let right = format!("{}  ", terrain_name);
    let padding = area.width.saturating_sub(left.len() as u16 + right.len() as u16);

    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(padding as usize)),
        Span::styled(right, Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(vec![line, Line::from("")]), area);
}

fn render_enemy_readout(
    frame: &mut Frame,
    area: Rect,
    enemies: &[LiveCombatant],
    focus_cursor: usize,
) {
    let active_enemies: Vec<&LiveCombatant> = enemies.iter()
        .filter(|e| !e.down)
        .collect();

    if active_enemies.is_empty() {
        return;
    }

    // Split horizontally for each enemy card
    let constraints: Vec<Constraint> = active_enemies.iter()
        .map(|_| Constraint::Ratio(1, active_enemies.len() as u32))
        .collect();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, enemy) in active_enemies.iter().enumerate() {
        let is_focused = i == focus_cursor;
        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let title = format!(" {} ", enemy.name);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(Color::White)));

        let inner = block.inner(cols[i]);
        frame.render_widget(block, cols[i]);

        // Nerve bar
        let nerve_ratio = if enemy.max_nerve > 0 {
            (enemy.nerve as f64 / enemy.max_nerve as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let nerve_color = if enemy.panicked {
            Color::Magenta
        } else {
            theme::gauge_color(enemy.nerve, enemy.max_nerve)
        };
        let nerve_label = if enemy.panicked {
            "Nerve  BROKE".to_string()
        } else {
            format!("Nerve  {}/{}", enemy.nerve, enemy.max_nerve)
        };

        if inner.height >= 1 {
            let gauge = Gauge::default()
                .ratio(nerve_ratio)
                .label(nerve_label)
                .gauge_style(Style::default().fg(nerve_color).bg(Color::DarkGray));
            frame.render_widget(gauge, Rect::new(inner.x, inner.y, inner.width, 1));
        }

        // Bluff level
        if inner.height >= 2 {
            let bluff_text = match enemy.bluff {
                0..=10 => "Bluff: None",
                11..=30 => "Bluff: Low",
                31..=60 => "Bluff: Moderate",
                _ => "Bluff: High",
            };
            let bluff_line = Paragraph::new(Line::from(Span::styled(
                format!(" {}", bluff_text),
                theme::dim_style(),
            )));
            frame.render_widget(bluff_line, Rect::new(inner.x, inner.y + 1, inner.width, 1));
        }
    }
}

fn render_posture_menu(
    frame: &mut Frame,
    area: Rect,
    ui: &StandoffUi,
    enemies: &[LiveCombatant],
) {
    let selected = ui.selected_posture();
    let show_focus = selected == StandoffPosture::Bait;

    let focus_height = if show_focus { 2u16 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                           // prompt
            Constraint::Min(3),                             // posture list
            Constraint::Length(focus_height),                // focus selector
            Constraint::Length(1),                           // hint
        ])
        .split(area);

    // Prompt
    let prompt = Paragraph::new(Line::from(Span::styled(
        "  Choose your posture:",
        Style::default().fg(Color::White),
    )));
    frame.render_widget(prompt, chunks[0]);

    // Posture items with tradeoff descriptions
    let posture_data = [
        (StandoffPosture::EarlyDraw,
         "EARLY DRAW",
         "Act first. Accuracy \u{2212}15. Heavy nerve damage to all enemies."),
        (StandoffPosture::SteadyHand,
         "STEADY HAND",
         "Balanced. No penalty. Moderate nerve pressure."),
        (StandoffPosture::Bait,
         "BAIT",
         "Provoke one target. Risk taking a hit. Focus target breaks faster."),
    ];

    let items: Vec<ListItem> = ui.postures.iter().enumerate().map(|(i, posture)| {
        let selected = i == ui.posture_cursor;
        let prefix = if selected { " > " } else { "   " };

        let (_, label, desc) = posture_data.iter()
            .find(|(p, _, _)| p == posture)
            .unwrap_or(&(StandoffPosture::SteadyHand, "???", ""));

        let label_style = if selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let desc_style = if selected {
            Style::default().fg(Color::Rgb(200, 200, 200))
        } else {
            theme::dim_style()
        };

        ListItem::new(Line::from(vec![
            Span::styled(format!("{}{:<14}", prefix, label), label_style),
            Span::styled(format!(" {}", desc), desc_style),
        ]))
    }).collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[1]);

    // Focus target selector (only when Bait is selected)
    if show_focus {
        let active_enemies: Vec<&LiveCombatant> = enemies.iter()
            .filter(|e| !e.down)
            .collect();

        let focus_name = active_enemies.get(ui.focus_cursor)
            .map(|e| e.name.as_str())
            .unwrap_or("none");

        let focus_line = Line::from(vec![
            Span::styled("  Focus: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("[{}]", focus_name),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled("                                 Tab to cycle", theme::dim_style()),
        ]);
        frame.render_widget(Paragraph::new(focus_line), chunks[2]);
    }

    // Hint line
    let hint = Paragraph::new(Line::from(Span::styled(
        "  [Enter] Commit posture   [Esc] Cancel",
        theme::dim_style(),
    )));
    frame.render_widget(hint, chunks[3]);
}

/// Render the standoff result — who broke, who hesitated, who got the jump.
pub fn render_standoff_result(
    frame: &mut Frame,
    area: Rect,
    encounter: &EncounterState,
    posture: StandoffPosture,
) {
    let result = match &encounter.standoff_result {
        Some(r) => r,
        None => return,
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Posture chosen
    let posture_name = match posture {
        StandoffPosture::EarlyDraw => "Early Draw",
        StandoffPosture::SteadyHand => "Steady Hand",
        StandoffPosture::Bait => "Bait",
    };
    lines.push(Line::from(Span::styled(
        format!("  {} \u{2014} {}", "Posture", posture_name),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Nerve damage results
    for (target, amount) in &result.nerve_damage {
        let enemy = encounter.enemies.iter().find(|e| e.id == *target);
        let name = enemy.map(|e| e.name.as_str()).unwrap_or(target);
        let broke = enemy.map(|e| e.panicked).unwrap_or(false);

        if broke {
            lines.push(Line::from(Span::styled(
                format!("  {} BREAKS. Nerve shattered.", name),
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            )));
        } else if *amount > 5 {
            lines.push(Line::from(Span::styled(
                format!("  {} hesitates. Nerve \u{2212}{}.", name, amount),
                Style::default().fg(Color::Yellow),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("  {} holds steady. Nerve \u{2212}{}.", name, amount),
                theme::dim_style(),
            )));
        }
    }

    // Initiative result
    lines.push(Line::from(""));
    if result.first_shot_accuracy < 0 {
        lines.push(Line::from(Span::styled(
            format!("  First shot accuracy: {}", result.first_shot_accuracy),
            Style::default().fg(Color::Red),
        )));
    } else if result.first_shot_accuracy > 0 {
        lines.push(Line::from(Span::styled(
            format!("  First shot accuracy: +{}", result.first_shot_accuracy),
            Style::default().fg(Color::Green),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Enter] Begin combat",
        theme::dim_style(),
    )));

    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(" Standoff Result ", Style::default().fg(Color::White))),
    );
    frame.render_widget(para, area);
}
