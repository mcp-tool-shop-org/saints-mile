//! Title screen — Saint's Mile opening.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::ui::theme;

/// Render the title screen.
pub fn render_title(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Min(8),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    let title_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  SAINT'S MILE",
            theme::title_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  A frontier JRPG for the adults who loved those games first.",
            Style::default().fg(Color::Rgb(160, 150, 130)),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "  [N] New Game     [L] Load Game     [Q] Quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(paragraph, chunks[1]);
}
