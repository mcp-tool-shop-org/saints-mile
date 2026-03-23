//! Theme constants — colors, age-phase styles, pacing/emotion maps.
//!
//! Visual language: spare, typographic, atmospheric. A western, not a fantasy game.
//! Emotion tints the speaker name only. Body text stays white/light gray always.

use ratatui::style::{Color, Modifier, Style};
use crate::types::AgePhase;
use crate::scene::types::{EmotionTag, PacingTag};

// ─── Age-Phase Identity ───────────────────────────────────────────

/// Border accent color for the current age phase.
pub fn age_accent(phase: AgePhase) -> Color {
    match phase {
        AgePhase::Youth => Color::Rgb(180, 200, 160),    // sage green
        AgePhase::YoungMan => Color::Rgb(200, 180, 120), // brass
        AgePhase::Adult => Color::Rgb(160, 140, 120),    // leather
        AgePhase::Older => Color::Rgb(140, 140, 150),    // iron gray
    }
}

/// Menu prompt changes by age — the command menu carries biography.
pub fn age_menu_title(phase: AgePhase) -> &'static str {
    match phase {
        AgePhase::Youth => "What do you do?",
        AgePhase::YoungMan => "Your move.",
        AgePhase::Adult => "Orders.",
        AgePhase::Older => "One more time.",
    }
}

/// How Galen's name renders by age.
pub fn age_name_style(phase: AgePhase) -> Style {
    match phase {
        AgePhase::Youth => Style::default().fg(Color::White),
        AgePhase::YoungMan => Style::default().fg(Color::White),
        AgePhase::Adult => Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        AgePhase::Older => Style::default().fg(Color::White)
            .add_modifier(Modifier::BOLD | Modifier::ITALIC),
    }
}

/// Age phase label for the top bar.
pub fn age_label(phase: AgePhase) -> &'static str {
    match phase {
        AgePhase::Youth => "Youth, Age 19",
        AgePhase::YoungMan => "Young Man, Age 24",
        AgePhase::Adult => "Adult, Age 34",
        AgePhase::Older => "Older, Age 50+",
    }
}

// ─── Emotion Colors ───────────────────────────────────────────────

/// Speaker name color based on emotion tag.
pub fn emotion_color(tag: Option<EmotionTag>) -> Color {
    match tag {
        None | Some(EmotionTag::Neutral) => Color::White,
        Some(EmotionTag::Warm) => Color::Rgb(210, 180, 140),   // tan
        Some(EmotionTag::Tense) => Color::Rgb(180, 80, 60),    // dusty red
        Some(EmotionTag::Bitter) => Color::Rgb(140, 140, 140),  // ash gray
        Some(EmotionTag::Dry) => Color::Rgb(180, 160, 120),    // dry grass
        Some(EmotionTag::Grief) => Color::Rgb(100, 110, 130),  // blue-gray
        Some(EmotionTag::Quiet) => Color::DarkGray,
    }
}

// ─── Pacing Rhythm ────────────────────────────────────────────────

/// Text reveal delay in milliseconds per character.
pub fn pacing_reveal_ms(tag: PacingTag) -> u64 {
    match tag {
        PacingTag::Exploration => 30,
        PacingTag::Pressure => 15,
        PacingTag::Intimate => 50,
        PacingTag::Crisis => 0, // instant
    }
}

/// Border style hint for pacing. Returns (use_border, double_border).
pub fn pacing_border(tag: PacingTag) -> PacingBorder {
    match tag {
        PacingTag::Exploration => PacingBorder::Plain,
        PacingTag::Pressure => PacingBorder::Double,
        PacingTag::Intimate => PacingBorder::None,
        PacingTag::Crisis => PacingBorder::Thick,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacingBorder {
    None,
    Plain,
    Double,
    Thick,
}

// ─── Gauge Colors ─────────────────────────────────────────────────

/// Color for a gauge bar based on fill percentage.
pub fn gauge_color(current: i32, max: i32) -> Color {
    if max <= 0 {
        return Color::DarkGray;
    }
    let pct = (current as f64 / max as f64 * 100.0) as i32;
    if pct > 60 {
        Color::Green
    } else if pct > 30 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Ammo display color — white above 3, red at 3 or below.
pub fn ammo_color(current: i32) -> Color {
    if current > 3 { Color::White } else { Color::Red }
}

// ─── Standard Styles ──────────────────────────────────────────────

/// Locked choice text style.
pub fn locked_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Lock reason text style.
pub fn lock_reason_style() -> Style {
    Style::default().fg(Color::Rgb(180, 80, 60)) // dusty red
}

/// Narrator text style (no speaker tag).
pub fn narrator_style() -> Style {
    Style::default().fg(Color::Rgb(200, 200, 200))
}

/// Standard body text.
pub fn body_style() -> Style {
    Style::default().fg(Color::Rgb(220, 220, 220))
}

/// Dim text for secondary information.
pub fn dim_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Memory echo style — italic, warm dim.
pub fn echo_style() -> Style {
    Style::default()
        .fg(Color::Rgb(180, 160, 120))
        .add_modifier(Modifier::ITALIC)
}

/// Echo border character style.
pub fn echo_border_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Title screen style.
pub fn title_style() -> Style {
    Style::default()
        .fg(Color::Rgb(200, 180, 140))
        .add_modifier(Modifier::BOLD)
}
