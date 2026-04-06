//! Save/Load screen — slot picker.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::ui::theme;

/// A save slot entry for display.
#[derive(Debug, Clone)]
pub struct SaveSlotInfo {
    pub name: String,
    pub label: String,
    pub exists: bool,
}

/// Render the save/load screen.
pub fn render_save_load(
    frame: &mut Frame,
    area: Rect,
    mode: SaveLoadMode,
    slots: &[SaveSlotInfo],
    cursor: usize,
    delete_confirming: Option<usize>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    // Title
    let title = match mode {
        SaveLoadMode::Save => "  SAVE GAME",
        SaveLoadMode::Load => "  LOAD GAME",
    };
    let title_para = Paragraph::new(Line::from(Span::styled(
        title,
        theme::title_style(),
    )));
    frame.render_widget(title_para, chunks[0]);

    // Slot list
    let items: Vec<ListItem> = slots
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            let selected = i == cursor;
            let prefix = if selected { " > " } else { "   " };

            if slot.exists {
                let style = if selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(200, 200, 200))
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{}[{}] {}", prefix, slot.name, slot.label),
                    style,
                )))
            } else {
                let style = if selected {
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)
                } else {
                    theme::dim_style()
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{}[{}] — empty —", prefix, slot.name),
                    style,
                )))
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(list, chunks[1]);

    // Hint / delete confirmation
    let hint_text = if let Some(idx) = delete_confirming {
        let slot_name = slots.get(idx).map(|s| s.name.as_str()).unwrap_or("?");
        format!("  Delete save [{}]? (Y/N)", slot_name)
    } else {
        "  [Enter] Select   [D] Delete   [Esc] Back".to_string()
    };
    let hint_style = if delete_confirming.is_some() {
        Style::default().fg(Color::Yellow)
    } else {
        theme::dim_style()
    };
    let hint = Paragraph::new(Line::from(Span::styled(hint_text, hint_style)));
    frame.render_widget(hint, chunks[2]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveLoadMode {
    Save,
    Load,
}
