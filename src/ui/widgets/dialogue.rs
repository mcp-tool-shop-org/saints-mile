//! Dialogue box widget — scrolling dialogue with speaker tags and emotion styling.
//!
//! Speaker names styled by EmotionTag. Narrator lines untagged.
//! Lines revealed progressively via TextReveal.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::scene::runner::DisplayedLine;
use crate::ui::theme;
use crate::ui::text_reveal::TextReveal;

/// Render dialogue lines into styled ratatui Lines.
pub fn render_dialogue<'a>(
    lines: &'a [DisplayedLine],
    reveal: &TextReveal,
    area_height: u16,
) -> Paragraph<'a> {
    let mut output: Vec<Line<'a>> = Vec::new();
    let is_narrator = |speaker: &str| speaker == "narrator" || speaker == "environment";

    for (i, line) in lines.iter().enumerate() {
        let visible_chars = reveal.visible_chars(i);
        if visible_chars == 0 && !reveal.line_visible(i) {
            break; // Haven't reached this line yet
        }

        // Blank line between speakers (except first line)
        if i > 0 {
            output.push(Line::from(""));
        }

        if is_narrator(&line.speaker) {
            // Narrator: no speaker tag, body style
            let text = truncate_to_chars(&line.text, visible_chars);
            output.push(Line::from(Span::styled(
                format!("  {}", text),
                theme::narrator_style(),
            )));
        } else {
            // Character: speaker name in emotion color, then text
            let speaker_display = line.speaker.to_uppercase();
            let speaker_color = theme::emotion_color(line.emotion);

            output.push(Line::from(Span::styled(
                format!("  {}", speaker_display),
                Style::default().fg(speaker_color).add_modifier(Modifier::BOLD),
            )));

            let text = truncate_to_chars(&line.text, visible_chars);
            output.push(Line::from(Span::styled(
                format!("  {}", text),
                theme::body_style(),
            )));
        }
    }

    // Auto-scroll: if content exceeds area, show the bottom
    let total_lines = output.len() as u16;
    let scroll = if total_lines > area_height {
        (total_lines - area_height, 0)
    } else {
        (0, 0)
    };

    Paragraph::new(output).scroll(scroll)
}

/// Truncate a string to at most `max_chars` characters.
fn truncate_to_chars(s: &str, max_chars: usize) -> String {
    if max_chars >= s.len() {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}
