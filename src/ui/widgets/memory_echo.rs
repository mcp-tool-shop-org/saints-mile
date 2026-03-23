//! Memory echo widget — inline rendering for cross-chapter callbacks.
//!
//! Half-box characters, italic, dim warm color.
//! The biscuit cloth, the flask, the poster — when they echo, the player sees it.

use ratatui::prelude::*;

use crate::scene::types::{MemoryRef, MemoryCallbackType};
use crate::state::types::MemoryObject;
use crate::ui::theme;

/// Render memory echo lines to append to the dialogue area.
pub fn render_echoes<'a>(
    callbacks: &[MemoryRef],
    memory_objects: &[MemoryObject],
) -> Vec<Line<'a>> {
    if callbacks.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    lines.push(Line::from("")); // breathing room

    for callback in callbacks {
        let verb = match callback.callback_type {
            MemoryCallbackType::Echo => "You remember",
            MemoryCallbackType::Carry => "You still carry",
            MemoryCallbackType::Transform => "It has changed",
        };

        let object_name = humanize_id(&callback.object.0);

        // For Transform echoes, show the current state
        let detail = if matches!(callback.callback_type, MemoryCallbackType::Transform) {
            memory_objects
                .iter()
                .find(|o| o.id == callback.object)
                .map(|o| format!(" — now {}", o.state))
                .unwrap_or_default()
        } else {
            String::new()
        };

        let echo_text = format!("{} — the {}{}", verb, object_name, detail);

        lines.push(Line::from(vec![
            Span::styled("  \u{2554} ", theme::echo_border_style()), // ╔
            Span::styled(echo_text, theme::echo_style()),
        ]));
        lines.push(Line::from(Span::styled(
            "  \u{255a}",
            theme::echo_border_style(), // ╚
        )));
    }

    lines
}

/// Convert a memory object ID to a specific, evocative name.
/// Key objects get authored descriptions; others fall back to humanized IDs.
fn humanize_id(id: &str) -> String {
    match id {
        "wanted_poster" => "wanted poster from the relay post".to_string(),
        "biscuit_cloth" => "biscuit cloth Nella gave you".to_string(),
        "flask" => "dented flask from the convoy".to_string(),
        "sheriff_badge" => "sheriff's badge, tarnished".to_string(),
        "relay_manifest" => "scorched relay manifest".to_string(),
        "voss_letter" => "letter in Voss's handwriting".to_string(),
        "mission_bell_fragment" => "fragment of the mission bell".to_string(),
        "ledger_page" => "page torn from the Briar Line ledger".to_string(),
        _ => id.replace('_', " "),
    }
}
