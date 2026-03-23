//! Text reveal engine — character-by-character reveal driven by PacingTag.
//!
//! This is how presentation rhythm becomes gameplay.
//! Intimate scenes breathe. Crisis scenes hit.

use std::time::{Duration, Instant};
use crate::scene::types::PacingTag;
use super::theme;

/// Tracks the progressive reveal of dialogue lines.
#[derive(Debug)]
pub struct TextReveal {
    /// Total number of lines to reveal.
    pub total_lines: usize,
    /// How many lines are fully revealed.
    pub lines_complete: usize,
    /// How many characters of the current line are visible.
    pub char_index: usize,
    /// Length of the current line in characters.
    pub current_line_len: usize,
    /// Time between character reveals.
    pub rate: Duration,
    /// When we last advanced a character.
    pub last_tick: Instant,
    /// All lines fully revealed, player can now choose.
    pub all_complete: bool,
}

impl TextReveal {
    /// Create a new text reveal for a set of lines with a pacing tag.
    pub fn new(line_lengths: &[usize], pacing: PacingTag) -> Self {
        let rate_ms = theme::pacing_reveal_ms(pacing);
        let total = line_lengths.len();
        let first_len = line_lengths.first().copied().unwrap_or(0);
        let all_complete = total == 0 || rate_ms == 0;

        Self {
            total_lines: total,
            lines_complete: if all_complete { total } else { 0 },
            char_index: if all_complete { first_len } else { 0 },
            current_line_len: first_len,
            rate: Duration::from_millis(rate_ms),
            last_tick: Instant::now(),
            all_complete,
        }
    }

    /// Advance the reveal by elapsed time. Call every frame.
    pub fn tick(&mut self, line_lengths: &[usize]) {
        if self.all_complete {
            return;
        }

        // Crisis pacing = instant reveal
        if self.rate.is_zero() {
            self.lines_complete = self.total_lines;
            self.all_complete = true;
            return;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);

        if elapsed >= self.rate {
            let chars_to_advance = (elapsed.as_millis() / self.rate.as_millis().max(1)) as usize;
            for _ in 0..chars_to_advance.max(1) {
                if self.lines_complete >= self.total_lines {
                    self.all_complete = true;
                    break;
                }

                if self.char_index < self.current_line_len {
                    self.char_index += 1;
                } else {
                    // Line complete, move to next
                    self.lines_complete += 1;
                    if self.lines_complete >= self.total_lines {
                        self.all_complete = true;
                        break;
                    }
                    self.char_index = 0;
                    self.current_line_len = line_lengths
                        .get(self.lines_complete)
                        .copied()
                        .unwrap_or(0);
                }
            }
            self.last_tick = now;
        }
    }

    /// Skip to the end of the current line (player pressed Space/Enter mid-reveal).
    /// Returns true if there was something to skip.
    pub fn skip_line(&mut self, line_lengths: &[usize]) -> bool {
        if self.all_complete {
            return false;
        }

        if self.char_index < self.current_line_len {
            // Complete the current line
            self.char_index = self.current_line_len;
            true
        } else {
            // Move to next line
            self.lines_complete += 1;
            if self.lines_complete >= self.total_lines {
                self.all_complete = true;
            } else {
                self.char_index = 0;
                self.current_line_len = line_lengths
                    .get(self.lines_complete)
                    .copied()
                    .unwrap_or(0);
            }
            true
        }
    }

    /// Complete all lines instantly.
    pub fn complete_all(&mut self) {
        self.lines_complete = self.total_lines;
        self.all_complete = true;
    }

    /// How many characters of line `idx` should be shown?
    pub fn visible_chars(&self, line_idx: usize) -> usize {
        if line_idx < self.lines_complete {
            usize::MAX // fully visible
        } else if line_idx == self.lines_complete {
            self.char_index
        } else {
            0 // not yet reached
        }
    }

    /// Is line `idx` fully visible?
    pub fn line_visible(&self, line_idx: usize) -> bool {
        line_idx < self.lines_complete
            || (line_idx == self.lines_complete && self.all_complete)
    }
}
