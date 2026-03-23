//! Choice menu widget — selectable choices with lock reasons and age identity.
//!
//! The player sees what they COULD do and why they can't. Absence is a promise.
//! Menu title changes by age phase — the command menu carries biography.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, BorderType, List, ListItem};

use crate::types::AgePhase;
use crate::scene::runner::PresentedChoice;
use crate::ui::theme;

/// Render the choice menu.
pub fn render_choice_menu<'a>(
    choices: &'a [PresentedChoice],
    cursor: usize,
    age_phase: AgePhase,
    show_choices: bool,
) -> List<'a> {
    let accent = theme::age_accent(age_phase);
    let title = theme::age_menu_title(age_phase);

    let border_type = BorderType::Plain;

    let block = Block::default()
        .borders(Borders::TOP)
        .border_type(border_type)
        .border_style(Style::default().fg(accent))
        .title(Span::styled(
            format!("  {}  ", title),
            Style::default().fg(accent),
        ));

    if !show_choices {
        // Choices not yet visible (still revealing text)
        return List::new(Vec::<ListItem>::new()).block(block);
    }

    let items: Vec<ListItem> = choices
        .iter()
        .enumerate()
        .map(|(i, choice)| {
            let selected = i == cursor;
            let prefix = if selected { " > " } else { "   " };

            if choice.available {
                let style = if selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", prefix, choice.label),
                    style,
                )))
            } else {
                // Locked choice: gray label + dusty red lock reason
                let reason = choice.lock_reason.as_deref().unwrap_or("[Locked]");
                let label_span = Span::styled(
                    format!("{}{}", prefix, choice.label),
                    theme::locked_style(),
                );
                let padding = " ".repeat(2);
                let reason_span = Span::styled(
                    format!("{}{}", padding, reason),
                    theme::lock_reason_style(),
                );
                ListItem::new(Line::from(vec![label_span, reason_span]))
            }
        })
        .collect();

    List::new(items).block(block)
}
