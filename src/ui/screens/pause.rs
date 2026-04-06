//! Pause screen — resume, save, or return to title.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::ui::theme;
use crate::ui::PauseOption;

/// Render the pause screen.
pub fn render_pause(frame: &mut Frame, area: Rect, cursor: usize) {
    let options = PauseOption::all();
    let mut lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "  PAUSED",
            Style::default().fg(Color::Rgb(200, 180, 140)).add_modifier(Modifier::BOLD),
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
        "  [Esc] Resume",
        theme::dim_style(),
    )));

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}
