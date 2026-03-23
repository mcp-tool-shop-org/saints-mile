//! Stat bar widget — reusable HP/nerve/ammo/pressure gauge.

use ratatui::prelude::*;
use ratatui::widgets::Gauge;

use crate::ui::theme;

/// Render a labeled gauge bar with threshold-based coloring.
pub fn stat_gauge<'a>(label: &'a str, current: i32, max: i32) -> Gauge<'a> {
    let ratio = if max > 0 {
        (current as f64 / max as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let color = theme::gauge_color(current, max);

    Gauge::default()
        .ratio(ratio)
        .label(format!("{} {}/{}", label, current, max))
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
}
