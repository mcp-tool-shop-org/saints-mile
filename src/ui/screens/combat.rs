//! Combat screen — turn-based party battle.
//!
//! The screen answers instantly: whose turn, who's near breaking,
//! what am I protecting, what changed this round.
//! Not a generic RPG battle screen — morally loaded, objective-driven,
//! biography-aware, text-first but not vague.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Table, Row, Cell};

use crate::combat::engine::*;
use crate::combat::types::*;
use crate::types::AgePhase;
use crate::ui::theme;

/// Combat UI state tracked by the App.
#[derive(Debug)]
pub struct CombatUi {
    pub action_cursor: usize,
    pub target_cursor: usize,
    /// The combat log — last N feedback lines.
    pub log: Vec<CombatLogEntry>,
    /// Whether we're showing the standoff result before round 1.
    pub showing_standoff_result: bool,
    /// The posture that was chosen (for result display).
    pub standoff_posture: Option<StandoffPosture>,
}

#[derive(Debug, Clone)]
pub struct CombatLogEntry {
    pub text: String,
    pub style: Style,
}

impl CombatUi {
    pub fn new() -> Self {
        Self {
            action_cursor: 0,
            target_cursor: 0,
            log: Vec::new(),
            showing_standoff_result: false,
            standoff_posture: None,
        }
    }

    /// Push an ActionResult into the combat log.
    pub fn push_result(&mut self, result: &ActionResult) {
        // Main action line
        self.log.push(CombatLogEntry {
            text: format!("  {}", result.action_description),
            style: Style::default().fg(Color::White),
        });

        // Damage events
        for dmg in &result.damage_dealt {
            let style = if dmg.was_critical {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let mut text = format!("  {} takes {} damage.", dmg.target, dmg.amount);
            if dmg.was_critical {
                text.push_str(" CRITICAL.");
            }
            if dmg.target_down {
                self.log.push(CombatLogEntry { text, style });
                self.log.push(CombatLogEntry {
                    text: format!("  {} is DOWN.", dmg.target),
                    style: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                });
                continue;
            }
            self.log.push(CombatLogEntry { text, style });
        }

        // Nerve damage events
        for nerve in &result.nerve_damage {
            if nerve.target_panicked {
                self.log.push(CombatLogEntry {
                    text: format!("  {} PANICS.", nerve.target),
                    style: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                });
            } else if nerve.amount > 0 {
                self.log.push(CombatLogEntry {
                    text: format!("  {}'s nerve shakes. (\u{2212}{})", nerve.target, nerve.amount),
                    style: Style::default().fg(Color::Yellow),
                });
            }
        }

        // Healing events
        for heal in &result.healing {
            self.log.push(CombatLogEntry {
                text: format!("  {} recovers {} HP.", heal.target, heal.amount),
                style: Style::default().fg(Color::Green),
            });
        }

        // Skill unlocks — biography moment
        for (char_id, skill_id) in &result.skill_unlocks {
            self.log.push(CombatLogEntry {
                text: format!("  ** {} learns {}. **", char_id, skill_id),
                style: Style::default().fg(Color::Rgb(200, 180, 140)).add_modifier(Modifier::BOLD),
            });
        }

        // Keep only last 8 entries
        while self.log.len() > 8 {
            self.log.remove(0);
        }
    }
}

/// Render the full combat screen.
pub fn render_combat(
    frame: &mut Frame,
    area: Rect,
    encounter: &EncounterState,
    ui: &CombatUi,
    age_phase: AgePhase,
    actions: &[CombatMenuItem],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),          // turn order bar
            Constraint::Length(6),          // enemy strip
            Constraint::Length(1),          // separator
            Constraint::Min(3),            // combat log
            Constraint::Length(1),          // separator
            Constraint::Length(4),          // party strip
            Constraint::Length(1),          // separator
            Constraint::Min(5),            // action menu + objectives
        ])
        .split(area);

    render_turn_bar(frame, chunks[0], encounter);
    render_enemy_strip(frame, chunks[1], encounter);
    render_separator(frame, chunks[2]);
    render_combat_log(frame, chunks[3], ui);
    render_separator(frame, chunks[4]);
    render_party_strip(frame, chunks[5], encounter);
    render_separator(frame, chunks[6]);
    render_action_menu(frame, chunks[7], encounter, ui, age_phase, actions);
}

// ─── Turn Order Bar ───────────────────────────────────────────────

fn render_turn_bar(frame: &mut Frame, area: Rect, encounter: &EncounterState) {
    let round_label = format!(" COMBAT \u{2014} Round {} ", encounter.round);

    let mut spans: Vec<Span> = vec![
        Span::styled(round_label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
    ];

    for (i, entry) in encounter.turn_queue.iter().enumerate() {
        let name = short_name(&entry.combatant_id, encounter);
        let is_current = i == encounter.current_turn;
        let is_past = i < encounter.current_turn;

        let style = if is_current {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else if is_past {
            theme::dim_style()
        } else {
            Style::default().fg(Color::Rgb(180, 180, 180))
        };

        if is_current {
            spans.push(Span::styled(format!("[{}]", name), style));
        } else {
            spans.push(Span::styled(name, style));
        }

        if i < encounter.turn_queue.len() - 1 {
            spans.push(Span::styled(" > ", theme::dim_style()));
        }
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ─── Enemy Strip ──────────────────────────────────────────────────

fn render_enemy_strip(frame: &mut Frame, area: Rect, encounter: &EncounterState) {
    let active: Vec<&LiveCombatant> = encounter.enemies.iter()
        .filter(|e| !e.down)
        .collect();

    if active.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "  All enemies neutralized.",
            Style::default().fg(Color::Green),
        )));
        frame.render_widget(msg, area);
        return;
    }

    let constraints: Vec<Constraint> = active.iter()
        .map(|_| Constraint::Ratio(1, active.len() as u32))
        .collect();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, enemy) in active.iter().enumerate() {
        render_combatant_card(frame, cols[i], enemy, false);
    }
}

// ─── Party Strip ──────────────────────────────────────────────────

fn render_party_strip(frame: &mut Frame, area: Rect, encounter: &EncounterState) {
    // Always 4 slots
    let constraints = vec![Constraint::Ratio(1, 4); 4];
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, slot) in encounter.party.iter().enumerate() {
        if let Some(member) = slot {
            render_combatant_card(frame, cols[i], member, true);
        } else {
            // Empty slot — subtle indication
            let empty = Paragraph::new(Line::from(Span::styled(
                "  \u{2500}\u{2500}\u{2500}",
                Style::default().fg(Color::Rgb(40, 40, 40)),
            )));
            frame.render_widget(empty, cols[i]);
        }
    }
}

// ─── Combatant Card ───────────────────────────────────────────────

fn render_combatant_card(frame: &mut Frame, area: Rect, c: &LiveCombatant, is_party: bool) {
    let name_style = if c.panicked {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else if c.down {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    };

    let mut lines: Vec<Line> = Vec::new();

    // Name line
    let mut name_spans = vec![Span::styled(format!(" {}", c.name), name_style)];
    if !c.wounds.is_empty() && is_party {
        // Show wound indicator
        let wound_names: Vec<&str> = c.wounds.iter().map(|w| w.name.as_str()).collect();
        name_spans.push(Span::styled(
            format!(" [{}]", wound_names.join(", ")),
            Style::default().fg(Color::Red),
        ));
    }
    lines.push(Line::from(name_spans));

    // HP bar
    let hp_color = if c.down { Color::Red } else { theme::gauge_color(c.hp, c.max_hp) };
    let hp_text = format!(" HP {}/{}  ", c.hp, c.max_hp);
    lines.push(Line::from(vec![
        Span::styled(" HP ", Style::default().fg(hp_color)),
        Span::styled(
            bar_chars(c.hp, c.max_hp, 10),
            Style::default().fg(hp_color),
        ),
        Span::styled(format!(" {}", c.hp), Style::default().fg(hp_color)),
    ]));

    // Nerve bar
    let nerve_display = if c.panicked {
        " PAN".to_string()
    } else {
        format!(" {}", c.nerve)
    };
    let nerve_color = if c.panicked { Color::Magenta } else { theme::gauge_color(c.nerve, c.max_nerve) };
    lines.push(Line::from(vec![
        Span::styled(" NRV", Style::default().fg(nerve_color)),
        Span::styled(
            bar_chars(c.nerve, c.max_nerve, 10),
            Style::default().fg(nerve_color),
        ),
        Span::styled(nerve_display, Style::default().fg(nerve_color)),
    ]));

    // Position + ammo (party only)
    if is_party {
        let pos = match c.position {
            PositionState::Open => "Open",
            PositionState::InCover => "In Cover",
            PositionState::Elevated => "Elevated",
            PositionState::FrontLine => "Front",
            PositionState::BackLine => "Back",
            PositionState::PartialCover => "Partial",
        };
        let ammo_color = theme::ammo_color(c.ammo);
        lines.push(Line::from(vec![
            Span::styled(format!(" [{}]", pos), theme::dim_style()),
            Span::styled(format!("  AMM {}", c.ammo), Style::default().fg(ammo_color)),
        ]));
    } else {
        // Enemy: position only
        let pos = match c.position {
            PositionState::Open => "Open",
            PositionState::InCover => "In Cover",
            _ => "",
        };
        if !pos.is_empty() {
            lines.push(Line::from(Span::styled(
                format!(" [{}]", pos),
                theme::dim_style(),
            )));
        }
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

/// Build a simple text gauge: ████░░░░
pub fn bar_chars(current: i32, max: i32, width: usize) -> String {
    if max <= 0 {
        return "\u{2591}".repeat(width);
    }
    let filled = ((current as f64 / max as f64) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

// ─── Combat Log ───────────────────────────────────────────────────

fn render_combat_log(frame: &mut Frame, area: Rect, ui: &CombatUi) {
    let max_lines = area.height as usize;
    let start = ui.log.len().saturating_sub(max_lines);

    let lines: Vec<Line> = ui.log[start..].iter().map(|entry| {
        Line::from(Span::styled(entry.text.clone(), entry.style))
    }).collect();

    frame.render_widget(Paragraph::new(lines), area);
}

// ─── Action Menu ──────────────────────────────────────────────────

/// A combat action the player can choose.
#[derive(Debug, Clone)]
pub struct CombatMenuItem {
    pub label: String,
    pub cost_text: String,
    pub line_label: String,
    pub available: bool,
    pub lock_reason: Option<String>,
}

pub fn render_action_menu(
    frame: &mut Frame,
    area: Rect,
    encounter: &EncounterState,
    ui: &CombatUi,
    age_phase: AgePhase,
    actions: &[CombatMenuItem],
) {
    // Split: action list on left, objectives on right
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Action menu
    let accent = theme::age_accent(age_phase);
    let title = theme::age_menu_title(age_phase);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(accent))
        .title(Span::styled(format!("  {}  ", title), Style::default().fg(accent)));

    let items: Vec<ListItem> = actions.iter().enumerate().map(|(i, action)| {
        let selected = i == ui.action_cursor;
        let prefix = if selected { " > " } else { "   " };

        if !action.available {
            let reason = action.lock_reason.as_deref().unwrap_or("[Locked]");
            return ListItem::new(Line::from(vec![
                Span::styled(format!("{}{}", prefix, action.label), theme::locked_style()),
                Span::styled(format!("  {}", reason), theme::lock_reason_style()),
            ]));
        }

        let label_style = if selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let cost_style = if selected {
            Style::default().fg(Color::Rgb(180, 180, 180))
        } else {
            theme::dim_style()
        };
        let line_style = theme::dim_style();

        ListItem::new(Line::from(vec![
            Span::styled(format!("{}{:<20}", prefix, action.label), label_style),
            Span::styled(format!("{:<16}", action.cost_text), cost_style),
            Span::styled(action.line_label.clone(), line_style),
        ]))
    }).collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, h_chunks[0]);

    // Objectives panel
    render_objectives(frame, h_chunks[1], &encounter.objectives);
}

fn render_objectives(frame: &mut Frame, area: Rect, objectives: &[LiveObjective]) {
    let block = Block::default()
        .borders(Borders::TOP | Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Objectives ", theme::dim_style()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = objectives.iter().map(|obj| {
        let (icon, color) = match obj.status {
            ObjectiveStatus::Active => (" [ ]", Color::White),
            ObjectiveStatus::Succeeded => (" [x]", Color::Green),
            ObjectiveStatus::Failed => (" [\u{2717}]", Color::Red),
        };
        let type_indicator = match obj.objective_type {
            ObjectiveType::Primary => "",
            ObjectiveType::Secondary => " (opt)",
        };

        Line::from(vec![
            Span::styled(icon, Style::default().fg(color)),
            Span::styled(
                format!(" {}{}", obj.label, type_indicator),
                Style::default().fg(color),
            ),
        ])
    }).collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new(Line::from(Span::styled(
        "\u{2500}".repeat(area.width as usize),
        Style::default().fg(Color::Rgb(60, 60, 60)),
    )));
    frame.render_widget(sep, area);
}

/// Get a short display name for a combatant.
fn short_name(id: &str, encounter: &EncounterState) -> String {
    // Check party
    for slot in &encounter.party {
        if let Some(member) = slot {
            if member.id == id {
                return member.name.split_whitespace().next()
                    .unwrap_or(&member.name).to_string();
            }
        }
    }
    // Check enemies
    for enemy in &encounter.enemies {
        if enemy.id == id {
            return enemy.name.clone();
        }
    }
    // Check NPCs
    for npc in &encounter.npc_allies {
        if npc.combatant.id == id {
            return npc.combatant.name.clone();
        }
    }
    id.to_string()
}
