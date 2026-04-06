//! Error screen — displays save/load failure messages.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::ui::theme;

/// Render the error screen.
pub fn render_error(frame: &mut Frame, area: Rect, message: &str) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "  ERROR",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", message),
            Style::default().fg(Color::Rgb(200, 180, 160)),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "  [Enter] or [Esc] to dismiss",
            theme::dim_style(),
        )),
    ];

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}
