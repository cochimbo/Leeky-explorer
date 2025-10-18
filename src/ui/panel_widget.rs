// Panel widget rendering
use crate::models::panel::Panel;
use crate::models::selection::SelectionState;
use crate::app::PanelSide;
use crate::ui::theme;
use crate::ui::file_icons;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

#[allow(clippy::too_many_arguments)]
pub fn render_panel(
    frame: &mut Frame,
    panel: &Panel,
    area: Rect,
    is_active: bool,
    search_mode: bool,
    search_pattern: &str,
    selection_state: &SelectionState,
    panel_side: PanelSide,
) {
    // T560: Add selection counter in header
    let selected_count = selection_state.count(panel_side);
    let title = if selected_count > 0 {
        format!(" {} ({} seleccionados) ", panel.current_path.display(), selected_count)
    } else {
        format!(" {} ", panel.current_path.display())
    };
    
    let border_style = if is_active {
        Style::default().fg(theme::ACTIVE_BORDER)
    } else {
        Style::default().fg(theme::INACTIVE_BORDER)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    // T409: Show "No results" message if filter is active but no entries
    let items: Vec<ListItem> = if panel.entries.is_empty() && panel.has_filter() {
        vec![ListItem::new(Line::from(Span::styled(
            format!(" Sin resultados para: {}", panel.get_filter().unwrap_or("")),
            Style::default().fg(theme::ERROR),
        )))]
    } else {
        panel
            .entries
            .iter()
            .map(|entry| {
                let mut style = theme::get_entry_style(&entry.entry_type);
                
                // T558: Show "*" prefix and alternate background for marked items
                let is_marked = selection_state.is_marked(panel_side, &entry.path);
                let prefix = if is_marked { "* " } else { "  " };
                
                // T559: Add alternate background color for marked items
                if is_marked {
                    style = style.bg(theme::MARKED_BG);
                }
                
                // T872-T873: Add emoji icon before filename
                let icon = file_icons::get_icon_for_entry(entry);
                let content = format!("{}{} {}", prefix, icon, entry);
                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect()
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme::HIGHLIGHT_BG)
                .fg(theme::HIGHLIGHT_FG)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(panel.cursor));

    frame.render_stateful_widget(list, area, &mut state);

    // T408: Render search bar at bottom of panel if active
    if is_active && search_mode {
        let search_text = format!(" Buscar: {}_ ", search_pattern);
        let search_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 1,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        let search_widget = Paragraph::new(Line::from(Span::styled(
            search_text,
            Style::default().fg(theme::WARNING),
        )));
        frame.render_widget(search_widget, search_area);
    } else if is_active && panel.has_filter() {
        // Show active filter when not in search mode
        if let Some(filter) = panel.get_filter() {
            let filter_text = format!(" Filtro: {} ", filter);
            let filter_area = Rect {
                x: area.x + 1,
                y: area.y + area.height - 1,
                width: area.width.saturating_sub(2),
                height: 1,
            };
            let filter_widget = Paragraph::new(Line::from(Span::styled(
                filter_text,
                Style::default().fg(theme::DIR_COLOR),
            )));
            frame.render_widget(filter_widget, filter_area);
        }
    }
}

