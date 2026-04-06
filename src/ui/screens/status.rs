//! Status screen — party state, skills, evidence, reputation.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::state::types::GameState;
use crate::ui::theme;

/// Render the status screen showing party, skills, memory objects, and reputation.
pub fn render_status(frame: &mut Frame, area: Rect, state: &GameState) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  STATUS",
        Style::default().fg(Color::Rgb(200, 180, 140)).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // --- Party members ---
    lines.push(Line::from(Span::styled(
        "  PARTY",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    for member in &state.party.members {
        let skills_text = if member.unlocked_skills.is_empty() {
            "no skills".to_string()
        } else {
            member.unlocked_skills.iter()
                .map(|s| s.0.replace('_', " "))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let injury_text = if member.injuries.is_empty() {
            String::new()
        } else {
            format!(" | injuries: {}", member.injuries.iter()
                .map(|i| i.0.replace('_', " "))
                .collect::<Vec<_>>()
                .join(", "))
        };
        let hand = match member.hand_state {
            crate::state::types::HandState::Healthy => "",
            crate::state::types::HandState::Damaged => " | hand: DAMAGED",
            crate::state::types::HandState::Adapted => " | hand: adapted",
        };
        lines.push(Line::from(Span::styled(
            format!("    {} — skills: {}{}{}",
                member.name, skills_text, injury_text, hand),
            Style::default().fg(Color::Rgb(200, 200, 200)),
        )));
    }
    lines.push(Line::from(""));

    // --- Resources ---
    lines.push(Line::from(Span::styled(
        "  RESOURCES",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("    Ammo: ", Style::default().fg(Color::Rgb(180, 180, 180))),
        Span::styled(
            format!("{}", state.resources.ammo),
            Style::default().fg(theme::ammo_color(state.resources.ammo)),
        ),
        Span::styled(
            format!("   Water: {}   Horse: {}",
                state.resources.water, state.resources.horse_stamina),
            Style::default().fg(Color::Rgb(180, 180, 180)),
        ),
    ]));
    lines.push(Line::from(""));

    // --- Memory objects ---
    if !state.memory_objects.is_empty() {
        lines.push(Line::from(Span::styled(
            "  MEMORY OBJECTS",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        for obj in &state.memory_objects {
            let state_label = if obj.state == "active" {
                String::new()
            } else {
                format!(" ({})", obj.state)
            };
            lines.push(Line::from(Span::styled(
                format!("    {}{}", obj.id.0.replace('_', " "), state_label),
                theme::echo_style(),
            )));
        }
        lines.push(Line::from(""));
    }

    // --- Reputation ---
    let rep_axes = [
        ("Rancher", crate::types::ReputationAxis::Rancher),
        ("Town Law", crate::types::ReputationAxis::TownLaw),
        ("Railroad", crate::types::ReputationAxis::Railroad),
    ];
    let has_rep = rep_axes.iter().any(|(_, axis)| state.reputation.get(*axis) != 0);
    if has_rep {
        lines.push(Line::from(Span::styled(
            "  REPUTATION",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        for (label, axis) in &rep_axes {
            let val = state.reputation.get(*axis);
            if val != 0 {
                let color = if val > 0 { Color::Green } else { Color::Red };
                lines.push(Line::from(vec![
                    Span::styled(format!("    {}: ", label), Style::default().fg(Color::Rgb(180, 180, 180))),
                    Span::styled(format!("{:+}", val), Style::default().fg(color)),
                ]));
            }
        }
        lines.push(Line::from(""));
    }

    // --- Evidence ---
    if !state.evidence.is_empty() {
        lines.push(Line::from(Span::styled(
            "  EVIDENCE",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        for item in &state.evidence {
            let integrity_color = if item.integrity >= 80 {
                Color::Green
            } else if item.integrity >= 40 {
                Color::Yellow
            } else {
                Color::Red
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {} ", item.id.0.replace('_', " ")),
                    Style::default().fg(Color::Rgb(200, 200, 200)),
                ),
                Span::styled(
                    format!("[integrity: {}%]", item.integrity),
                    Style::default().fg(integrity_color),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "  [Esc] or [Tab] to return",
        theme::dim_style(),
    )));

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}
