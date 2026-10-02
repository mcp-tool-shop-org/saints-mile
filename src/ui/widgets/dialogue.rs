//! Dialogue box widget — scrolling dialogue with speaker tags and emotion styling.
//!
//! Speaker names styled by EmotionTag. Narrator lines untagged.
//! Lines revealed progressively via TextReveal.
//!
//! Prose is word-wrapped HERE, to the width it will be drawn at, rather than by
//! ratatui's `Wrap`. Two reasons. The auto-scroll has to count the rows the text
//! really occupies, and only wrapping it ourselves gives that number; with `Wrap`
//! the paragraph would still count logical lines and scroll the newest text off
//! the bottom. And the reveal walks characters across rows that were wrapped from
//! the FULL line, so a word never starts on one row and jumps to the next as it
//! types out.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::scene::runner::DisplayedLine;
use crate::ui::theme;
use crate::ui::text_reveal::TextReveal;

/// Left margin every prose row carries, wrapped continuation rows included.
const MARGIN: &str = "  ";

/// Render dialogue lines into styled ratatui Lines, wrapped to `area_width` and
/// scrolled so the newest text sits at the bottom of `area_height` rows.
///
/// Pass the area the paragraph is drawn into (inside any border), not the outer
/// area: both numbers are row and column counts of the text itself.
pub fn render_dialogue<'a>(
    lines: &'a [DisplayedLine],
    reveal: &TextReveal,
    area_width: u16,
    area_height: u16,
) -> Paragraph<'a> {
    let mut output: Vec<Line<'a>> = Vec::new();
    let is_narrator = |speaker: &str| speaker == "narrator" || speaker == "environment";
    let text_width = (area_width as usize).saturating_sub(MARGIN.len()).max(1);

    for (i, line) in lines.iter().enumerate() {
        let visible_chars = reveal.visible_chars(i);
        if visible_chars == 0 && !reveal.line_visible(i) {
            break; // Haven't reached this line yet
        }

        // Blank line between speakers (except first line)
        if i > 0 {
            output.push(Line::from(""));
        }

        let style = if is_narrator(&line.speaker) {
            // Narrator: no speaker tag, body style
            theme::narrator_style()
        } else {
            // Character: speaker name in emotion color, then text
            let speaker_color = theme::emotion_color(line.emotion);
            output.push(Line::from(Span::styled(
                format!("{MARGIN}{}", line.speaker.to_uppercase()),
                Style::default().fg(speaker_color).add_modifier(Modifier::BOLD),
            )));
            theme::body_style()
        };

        for row in revealed_rows(&line.text, text_width, visible_chars) {
            output.push(Line::from(Span::styled(format!("{MARGIN}{row}"), style)));
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

/// Word-wrap `text` to `width` columns, returning each row as a char range
/// `[start, end)` into the text. Spaces at a break are dropped; a word longer
/// than the row is split. Widths are counted in chars, which is exact for this
/// game's prose (Latin script and punctuation, no double-width glyphs).
fn wrap_ranges(text: &str, width: usize) -> Vec<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();
    let width = width.max(1);
    let mut rows = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        if chars.len() - start <= width {
            rows.push((start, chars.len()));
            break;
        }
        // Break at the last space that keeps the row within width.
        let limit = start + width;
        match (start..=limit).rev().find(|&j| chars[j] == ' ') {
            Some(space) if space > start => {
                rows.push((start, space));
                start = space;
            }
            _ => {
                rows.push((start, limit));
                start = limit;
            }
        }
        while start < chars.len() && chars[start] == ' ' {
            start += 1;
        }
    }
    if rows.is_empty() {
        rows.push((0, 0)); // an empty line still takes a row
    }
    rows
}

/// The rows of `text` wrapped to `width`, showing only its first `visible` chars.
/// Rows past the reveal point are omitted, the row it is on is cut at that point.
fn revealed_rows(text: &str, width: usize, visible: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    for (start, end) in wrap_ranges(text, width) {
        if start >= visible && start > 0 {
            break;
        }
        out.push(chars[start..end.min(visible).max(start)].iter().collect());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::types::PacingTag;
    use ratatui::buffer::Buffer;

    const POSTER: &str = "Dusk on a dead stretch of trail called Saint's Mile. Your horse drinks from a nearly dry trough behind an abandoned relay post.";
    const POSTED: &str = "Nailed to the post is a fresh wanted poster with your name on it. Not as legend \u{2014} as current business. Somebody nearby cared enough to print and post it.";

    fn narrator(text: &str) -> DisplayedLine {
        DisplayedLine { speaker: "narrator".into(), text: text.into(), emotion: None, pacing: PacingTag::Exploration }
    }

    fn eli(text: &str) -> DisplayedLine {
        DisplayedLine { speaker: "eli".into(), text: text.into(), emotion: None, pacing: PacingTag::Exploration }
    }

    fn revealed(lines: &[DisplayedLine]) -> TextReveal {
        let lens: Vec<usize> = lines.iter().map(|l| l.text.chars().count()).collect();
        let mut r = TextReveal::new(&lens, PacingTag::Exploration);
        r.complete_all();
        r
    }

    /// Draw the dialogue into a buffer the size of the area and return its rows.
    fn draw(lines: &[DisplayedLine], reveal: &TextReveal, w: u16, h: u16) -> Vec<String> {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        render_dialogue(lines, reveal, w, h).render(area, &mut buf);
        (0..h)
            .map(|y| (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string())
            .collect()
    }

    fn joined(rows: &[String]) -> String {
        rows.iter().map(|r| r.trim()).filter(|r| !r.is_empty()).collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn the_end_of_a_long_line_survives_at_100_and_80_columns() {
        // The bug: unwrapped, row 1 ended "...a nearly dry trough be" at 100 columns.
        let lines = vec![narrator(POSTER), narrator(POSTED)];
        let reveal = revealed(&lines);
        for (w, h) in [(100u16, 24u16), (80, 20)] {
            let rows = draw(&lines, &reveal, w, h);
            let text = joined(&rows);
            assert!(text.contains("behind an abandoned relay post."), "{w} cols lost the end of line 1: {rows:#?}");
            assert!(text.contains("cared enough to print and post it."), "{w} cols lost the end of line 2: {rows:#?}");
            assert!(rows.iter().all(|r| r.chars().count() <= w as usize));
        }
    }

    #[test]
    fn wrapped_rows_keep_the_margin_and_break_between_words() {
        let rows = draw(&[narrator(POSTER)], &revealed(&[narrator(POSTER)]), 60, 10);
        let prose: Vec<&String> = rows.iter().filter(|r| !r.trim().is_empty()).collect();
        assert!(prose.len() >= 2, "a 130-char line at 60 columns takes more than one row");
        for r in &prose {
            assert!(r.starts_with("  ") && !r.starts_with("   "), "every row carries the 2-space margin: {r:?}");
        }
        // No word was cut in half: rejoining the rows gives back the sentence.
        assert_eq!(joined(&rows), POSTER);
    }

    #[test]
    fn auto_scroll_counts_wrapped_rows_so_the_newest_line_stays_on_screen() {
        // Six long lines at 50 columns overflow 12 rows. Counting logical lines,
        // the scroll was too small and the last line fell off the bottom.
        let mut lines: Vec<DisplayedLine> = (0..5).map(|_| narrator(POSTED)).collect();
        lines.push(eli("Then we ride for Morrow Crossing before the light goes."));
        let rows = draw(&lines, &revealed(&lines), 50, 12);
        assert!(joined(&rows).ends_with("before the light goes."), "{rows:#?}");
    }

    #[test]
    fn a_partly_revealed_line_types_into_its_final_rows_without_reflowing() {
        let full = revealed_rows(POSTER, 40, usize::MAX);
        for visible in [1, 17, 45, 90, POSTER.chars().count()] {
            let part = revealed_rows(POSTER, 40, visible);
            for (i, row) in part.iter().enumerate() {
                assert!(full[i].starts_with(row.as_str()), "at {visible} chars row {i} {row:?} is not a prefix of {:?}", full[i]);
            }
        }
    }

    #[test]
    fn a_word_longer_than_the_row_is_split_rather_than_lost() {
        let rows = revealed_rows("abcdefghijkl mn", 5, usize::MAX);
        assert_eq!(rows, vec!["abcde", "fghij", "kl mn"]);
    }

    #[test]
    fn an_empty_line_still_takes_a_row() {
        assert_eq!(revealed_rows("", 10, usize::MAX), vec![String::new()]);
    }
}
