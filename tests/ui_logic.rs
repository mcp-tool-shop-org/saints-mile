//! Tests for non-rendering UI logic — text reveal timing, theme pure functions.
//!
//! Skip anything requiring a terminal/TUI runtime.
//! Focus on testable logic only.

use saints_mile::scene::types::PacingTag;
use saints_mile::ui::text_reveal::TextReveal;
use saints_mile::ui::theme;
use saints_mile::types::AgePhase;

// ─── Text Reveal ──────────────────────────────────────────────────

/// Crisis pacing produces instant reveal (zero delay).
#[test]
fn crisis_pacing_instant_reveal() {
    let lines = vec![10, 20, 15];
    let reveal = TextReveal::new(&lines, PacingTag::Crisis);
    assert!(reveal.all_complete, "crisis pacing should complete instantly");
    assert_eq!(reveal.lines_complete, 3);
}

/// Exploration pacing does not instantly reveal.
#[test]
fn exploration_pacing_is_gradual() {
    let lines = vec![10, 20];
    let reveal = TextReveal::new(&lines, PacingTag::Exploration);
    assert!(!reveal.all_complete, "exploration pacing should not complete instantly");
    assert_eq!(reveal.lines_complete, 0);
    assert_eq!(reveal.char_index, 0);
}

/// Intimate pacing is slower than exploration.
#[test]
fn intimate_pacing_is_slower() {
    let intimate = TextReveal::new(&[10], PacingTag::Intimate);
    let exploration = TextReveal::new(&[10], PacingTag::Exploration);
    assert!(intimate.rate > exploration.rate,
        "intimate reveal rate {:?} should be slower than exploration {:?}",
        intimate.rate, exploration.rate);
}

/// Empty lines produce completed reveal.
#[test]
fn empty_lines_complete_immediately() {
    let reveal = TextReveal::new(&[], PacingTag::Exploration);
    assert!(reveal.all_complete);
    assert_eq!(reveal.total_lines, 0);
}

/// visible_chars returns correct values per line index.
#[test]
fn visible_chars_returns_correct_values() {
    let lines = vec![10, 20, 15];
    let reveal = TextReveal::new(&lines, PacingTag::Exploration);

    // Line 0 is current — shows char_index (0)
    assert_eq!(reveal.visible_chars(0), 0);
    // Line 1 is future — shows 0
    assert_eq!(reveal.visible_chars(1), 0);
    // Line 2 is future — shows 0
    assert_eq!(reveal.visible_chars(2), 0);

    // Crisis: all lines fully visible
    let crisis = TextReveal::new(&lines, PacingTag::Crisis);
    assert_eq!(crisis.visible_chars(0), usize::MAX);
    assert_eq!(crisis.visible_chars(1), usize::MAX);
    assert_eq!(crisis.visible_chars(2), usize::MAX);
}

/// skip_line completes the current line first, then advances.
#[test]
fn skip_line_completes_then_advances() {
    let lines = vec![10, 20];
    let mut reveal = TextReveal::new(&lines, PacingTag::Exploration);

    // First skip: complete line 0
    assert!(reveal.skip_line(&lines));
    assert_eq!(reveal.char_index, 10); // line 0 length

    // Second skip: advance to line 1
    assert!(reveal.skip_line(&lines));
    assert_eq!(reveal.lines_complete, 1);
    assert_eq!(reveal.char_index, 0);
    assert!(!reveal.all_complete);

    // Third skip: complete line 1
    assert!(reveal.skip_line(&lines));
    assert_eq!(reveal.char_index, 20);

    // Fourth skip: all complete
    assert!(reveal.skip_line(&lines));
    assert!(reveal.all_complete);
}

/// complete_all finishes everything instantly.
#[test]
fn complete_all_finishes_instantly() {
    let lines = vec![10, 20, 30];
    let mut reveal = TextReveal::new(&lines, PacingTag::Intimate);
    assert!(!reveal.all_complete);

    reveal.complete_all();
    assert!(reveal.all_complete);
    assert_eq!(reveal.lines_complete, 3);
}

/// skip_line returns false when already complete.
#[test]
fn skip_line_returns_false_when_complete() {
    let lines = vec![5];
    let mut reveal = TextReveal::new(&lines, PacingTag::Crisis);
    assert!(reveal.all_complete);
    assert!(!reveal.skip_line(&lines));
}

/// line_visible returns correct state.
#[test]
fn line_visible_tracks_correctly() {
    let lines = vec![5, 10];
    let mut reveal = TextReveal::new(&lines, PacingTag::Exploration);

    assert!(!reveal.line_visible(0));
    assert!(!reveal.line_visible(1));

    reveal.complete_all();
    assert!(reveal.line_visible(0));
    assert!(reveal.line_visible(1));
}

// ─── Theme Pure Functions ─────────────────────────────────────────

/// Pacing reveal rates are ordered: crisis < pressure < exploration < intimate.
#[test]
fn pacing_reveal_rates_ordered() {
    let crisis = theme::pacing_reveal_ms(PacingTag::Crisis);
    let pressure = theme::pacing_reveal_ms(PacingTag::Pressure);
    let exploration = theme::pacing_reveal_ms(PacingTag::Exploration);
    let intimate = theme::pacing_reveal_ms(PacingTag::Intimate);

    assert_eq!(crisis, 0, "crisis should be instant");
    assert!(pressure < exploration, "pressure should be faster than exploration");
    assert!(exploration < intimate, "exploration should be faster than intimate");
}

/// Age menu titles change across all four phases.
#[test]
fn age_menu_titles_differ() {
    let titles: Vec<_> = [AgePhase::Youth, AgePhase::YoungMan, AgePhase::Adult, AgePhase::Older]
        .iter()
        .map(|p| theme::age_menu_title(*p))
        .collect();

    // All four should be unique
    for i in 0..titles.len() {
        for j in (i + 1)..titles.len() {
            assert_ne!(titles[i], titles[j],
                "menu titles should differ between age phases");
        }
    }
}

/// Age labels include descriptive text.
#[test]
fn age_labels_are_descriptive() {
    assert!(theme::age_label(AgePhase::Youth).contains("Youth"));
    assert!(theme::age_label(AgePhase::YoungMan).contains("Young Man"));
    assert!(theme::age_label(AgePhase::Adult).contains("Adult"));
    assert!(theme::age_label(AgePhase::Older).contains("Older"));
}

/// Gauge color transitions at correct thresholds.
#[test]
fn gauge_color_thresholds() {
    use ratatui::style::Color;

    // Full health = green
    assert_eq!(theme::gauge_color(100, 100), Color::Green);
    // Half health = yellow
    assert_eq!(theme::gauge_color(50, 100), Color::Yellow);
    // Low health = red
    assert_eq!(theme::gauge_color(20, 100), Color::Red);
    // Zero max = dark gray
    assert_eq!(theme::gauge_color(0, 0), Color::DarkGray);
}

/// Ammo color transitions at threshold.
#[test]
fn ammo_color_thresholds() {
    use ratatui::style::Color;

    assert_eq!(theme::ammo_color(10), Color::White);
    assert_eq!(theme::ammo_color(4), Color::White);
    assert_eq!(theme::ammo_color(3), Color::Red);
    assert_eq!(theme::ammo_color(1), Color::Red);
}

/// Pacing border varies by pacing tag.
#[test]
fn pacing_border_varies() {
    let exploration = theme::pacing_border(PacingTag::Exploration);
    let crisis = theme::pacing_border(PacingTag::Crisis);
    let intimate = theme::pacing_border(PacingTag::Intimate);

    assert_ne!(exploration, crisis, "exploration and crisis should have different borders");
    assert_ne!(intimate, crisis, "intimate and crisis should have different borders");
    assert_eq!(intimate, theme::PacingBorder::None, "intimate should have no border");
}
