//! Transmission Race screen — time getting weaponized.
//!
//! Chapter 9's split-party operational language. Dispatch sent/delayed/intercepted.
//! Line status. Witness route timing. False-message propagation.
//! Leaner than the other pressure screens, but legible.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell};

use crate::combat::split_party::*;
use crate::pressure::types::*;
use crate::ui::theme;

/// Transmission-specific UI state.
#[derive(Debug)]
pub struct TransmissionUi {
    pub assignment_cursor: usize,
    pub phase: TransmissionPhase,
    pub log: Vec<TransmissionLogEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransmissionPhase {
    /// Player is assigning teams to objectives.
    Assignment,
    /// Results are being revealed.
    Results,
    /// Summary — all results shown.
    Summary,
}

#[derive(Debug, Clone)]
pub struct TransmissionLogEntry {
    pub text: String,
    pub style: Style,
}

impl TransmissionUi {
    pub fn new() -> Self {
        Self {
            assignment_cursor: 0,
            phase: TransmissionPhase::Assignment,
            log: Vec::new(),
        }
    }
}

/// Render the transmission race — assignment phase.
pub fn render_transmission_assignment(
    frame: &mut Frame,
    area: Rect,
    channels: &[Channel],
    teams: &[TeamDisplayInfo],
    ui: &TransmissionUi,
    time_remaining: i32,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),              // title
            Constraint::Length(channels.len() as u16 + 2), // channel status
            Constraint::Length(1),              // separator
            Constraint::Min(5),                // team assignments
            Constraint::Length(2),              // hint
        ])
        .split(area);

    // Title
    let line = Line::from(vec![
        Span::styled(
            " TRANSMISSION RACE",
            Style::default().fg(Color::Rgb(140, 160, 200)).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("   Time remaining: {} hours", time_remaining),
            Style::default().fg(if time_remaining <= 3 { Color::Red } else { Color::Yellow }),
        ),
    ]);
    frame.render_widget(Paragraph::new(vec![line, Line::from("")]), chunks[0]);

    // Channel status
    render_channels(frame, chunks[1], channels);

    render_separator(frame, chunks[2]);

    // Team assignments
    render_team_assignments(frame, chunks[3], teams, ui);

    // Hint
    let hint = Paragraph::new(Line::from(Span::styled(
        "  [Enter] Confirm assignments   [Tab] Cycle member   [Up/Down] Select team",
        theme::dim_style(),
    )));
    frame.render_widget(hint, chunks[4]);
}

/// Render the transmission results.
pub fn render_transmission_results(
    frame: &mut Frame,
    area: Rect,
    operation: &SplitOperation,
    ui: &TransmissionUi,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // title
            Constraint::Min(5),    // results
            Constraint::Length(2),  // continue hint
        ])
        .split(area);

    let line = Line::from(Span::styled(
        " TRANSMISSION RACE \u{2014} Results",
        Style::default().fg(Color::Rgb(140, 160, 200)).add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(vec![line, Line::from("")]), chunks[0]);

    // Results for each team
    let mut lines: Vec<Line> = Vec::new();

    for (team, result) in operation.teams.iter().zip(operation.results.iter()) {
        let synergy_str = match team.synergy {
            TeamSynergy::Strong => "strong",
            TeamSynergy::Functional => "functional",
            TeamSynergy::Volatile => "volatile",
            TeamSynergy::Hostile => "hostile",
        };
        let synergy_color = match team.synergy {
            TeamSynergy::Strong => Color::Green,
            TeamSynergy::Functional => Color::White,
            TeamSynergy::Volatile => Color::Yellow,
            TeamSynergy::Hostile => Color::Red,
        };

        let outcome_icon = if result.success { "\u{2713}" } else { "\u{2717}" };
        let outcome_color = if result.success { Color::Green } else { Color::Red };

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", outcome_icon),
                Style::default().fg(outcome_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} ", team.name),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({})", synergy_str),
                Style::default().fg(synergy_color),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("    {}", team.objective),
            theme::dim_style(),
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", result.report),
            Style::default().fg(Color::Rgb(200, 200, 200)),
        )));
    }

    frame.render_widget(Paragraph::new(lines), chunks[1]);

    let hint = Paragraph::new(Line::from(Span::styled(
        "  [Enter] Continue",
        theme::dim_style(),
    )));
    frame.render_widget(hint, chunks[2]);
}

fn render_channels(frame: &mut Frame, area: Rect, channels: &[Channel]) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Lines ", Style::default().fg(Color::Rgb(140, 160, 200))));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = channels.iter().map(|ch| {
        let owner_str = match ch.controlled_by {
            ChannelOwner::Party => "OURS",
            ChannelOwner::Enemy => "THEIRS",
            ChannelOwner::Neutral => "OPEN",
        };
        let owner_color = match ch.controlled_by {
            ChannelOwner::Party => Color::Green,
            ChannelOwner::Enemy => Color::Red,
            ChannelOwner::Neutral => Color::Yellow,
        };

        let relay_str = ch.relay_points.join(" \u{2192} ");

        Line::from(vec![
            Span::styled(format!("  {:<16}", ch.name), Style::default().fg(Color::White)),
            Span::styled(format!("[{}]  ", owner_str), Style::default().fg(owner_color)),
            Span::styled(relay_str, theme::dim_style()),
        ])
    }).collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Info for displaying a team assignment.
#[derive(Debug, Clone)]
pub struct TeamDisplayInfo {
    pub team_name: String,
    pub objective: String,
    pub members: Vec<String>,
    pub synergy: TeamSynergy,
}

fn render_team_assignments(
    frame: &mut Frame,
    area: Rect,
    teams: &[TeamDisplayInfo],
    ui: &TransmissionUi,
) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(140, 160, 200)))
        .title(Span::styled("  Assign your people.  ", Style::default().fg(Color::Rgb(140, 160, 200))));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    for (i, team) in teams.iter().enumerate() {
        let selected = i == ui.assignment_cursor;
        let prefix = if selected { " > " } else { "   " };

        let synergy_color = match team.synergy {
            TeamSynergy::Strong => Color::Green,
            TeamSynergy::Functional => Color::White,
            TeamSynergy::Volatile => Color::Yellow,
            TeamSynergy::Hostile => Color::Red,
        };

        let style = if selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{}{}", prefix, team.team_name), style),
            Span::styled(
                format!("  [{}]", team.members.join(", ")),
                Style::default().fg(synergy_color),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("     {}", team.objective),
            theme::dim_style(),
        )));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new(Line::from(Span::styled(
        "\u{2500}".repeat(area.width as usize),
        Style::default().fg(Color::Rgb(60, 60, 60)),
    )));
    frame.render_widget(sep, area);
}
