// Panel widget rendering
use crate::models::panel::Panel;
use crate::models::selection::SelectionState;
use crate::app::PanelSide;
use crate::ui::theme;
use crate::ui::file_icons;
use crate::ui::column_layout::{ColumnLayout, Alignment};
use crate::ui::formatters;
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

    // Calculate column layout based on available width
    let content_width = area.width.saturating_sub(4); // Subtract borders and padding
    let layout = ColumnLayout::calculate(content_width, &panel.entries);

    // T409: Show "No results" message if filter is active but no entries
    let items: Vec<ListItem> = if panel.entries.is_empty() && panel.has_filter() {
        vec![ListItem::new(Line::from(Span::styled(
            format!(" Sin resultados para: {}", panel.get_filter().unwrap_or("")),
            Style::default().fg(theme::ERROR),
        )))]
    } else {
        // Build column header as first item
        let mut all_items = Vec::new();
        
        // Header row
        let header_line = build_header_row(&layout);
        all_items.push(ListItem::new(header_line));
        
        // Separator row
        let separator = "─".repeat(content_width as usize);
        all_items.push(ListItem::new(Line::from(Span::styled(
            separator,
            Style::default().fg(theme::INACTIVE_BORDER),
        ))));
        
        // Data rows
        for entry in &panel.entries {
            let mut style = theme::get_entry_style(&entry.entry_type);
            
            // T558: Show "*" prefix and alternate background for marked items
            let is_marked = selection_state.is_marked(panel_side, &entry.path);
            
            // T559: Add alternate background color for marked items
            if is_marked {
                style = style.bg(theme::MARKED_BG);
            }
            
            // Build columnar row
            let line = build_data_row(entry, &layout, is_marked, style);
            all_items.push(ListItem::new(line));
        }
        
        all_items
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
    // Offset cursor by 2 to account for header and separator rows
    let cursor_offset = if panel.entries.is_empty() { 0 } else { panel.cursor + 2 };
    state.select(Some(cursor_offset));

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

/// Build header row with column titles
fn build_header_row(layout: &ColumnLayout) -> Line<'static> {
    let mut spans = Vec::new();
    
    // Icon column (empty header)
    spans.push(Span::styled(
        formatters::pad_text("", layout.icon_width, Alignment::Left),
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw("  "));
    
    // Mark column (empty header)
    spans.push(Span::styled(
        formatters::pad_text("", layout.mark_width, Alignment::Left),
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw("  "));
    
    // Name column
    spans.push(Span::styled(
        formatters::pad_text("Name", layout.name_width, Alignment::Left),
        Style::default().add_modifier(Modifier::BOLD).fg(theme::ACTIVE_BORDER),
    ));
    spans.push(Span::raw("  "));
    
    // Extension column (if visible)
    if layout.show_extension {
        spans.push(Span::styled(
            formatters::pad_text("Ext", layout.ext_width, Alignment::Left),
            Style::default().add_modifier(Modifier::BOLD).fg(theme::ACTIVE_BORDER),
        ));
        spans.push(Span::raw("  "));
    }
    
    // Size column
    spans.push(Span::styled(
        formatters::pad_text("Size", layout.size_width, Alignment::Right),
        Style::default().add_modifier(Modifier::BOLD).fg(theme::ACTIVE_BORDER),
    ));
    spans.push(Span::raw("  "));
    
    // Modified column
    spans.push(Span::styled(
        formatters::pad_text("Modified", layout.modified_width, Alignment::Center),
        Style::default().add_modifier(Modifier::BOLD).fg(theme::ACTIVE_BORDER),
    ));
    
    // Created column (if visible)
    if layout.show_created {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            formatters::pad_text("Created", layout.created_width, Alignment::Center),
            Style::default().add_modifier(Modifier::BOLD).fg(theme::ACTIVE_BORDER),
        ));
    }
    
    // Permissions column (if visible)
    if layout.show_permissions {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            formatters::pad_text("Perms", layout.perms_width, Alignment::Center),
            Style::default().add_modifier(Modifier::BOLD).fg(theme::ACTIVE_BORDER),
        ));
    }
    
    Line::from(spans)
}

/// Build data row for a file entry with all columns
fn build_data_row(
    entry: &crate::models::file_entry::FileEntry,
    layout: &ColumnLayout,
    is_marked: bool,
    style: Style,
) -> Line<'static> {
    let mut spans = Vec::new();
    
    // Icon column
    let icon = file_icons::get_icon_for_entry(entry);
    spans.push(Span::styled(
        formatters::pad_text(&icon, layout.icon_width, Alignment::Left),
        style,
    ));
    spans.push(Span::raw("  "));
    
    // Mark column
    let mark = if is_marked { "*" } else { "" };
    spans.push(Span::styled(
        formatters::pad_text(mark, layout.mark_width, Alignment::Left),
        style,
    ));
    spans.push(Span::raw("  "));
    
    // Name column (truncate if too long)
    let name = &entry.name;
    spans.push(Span::styled(
        formatters::pad_text(name, layout.name_width, Alignment::Left),
        style,
    ));
    spans.push(Span::raw("  "));
    
    // Extension column (if visible)
    if layout.show_extension {
        let ext = formatters::format_extension(entry);
        spans.push(Span::styled(
            formatters::pad_text(&ext, layout.ext_width, Alignment::Left),
            style,
        ));
        spans.push(Span::raw("  "));
    }
    
    // Size column
    let size = formatters::format_size(entry);
    spans.push(Span::styled(
        formatters::pad_text(&size, layout.size_width, Alignment::Right),
        style,
    ));
    spans.push(Span::raw("  "));
    
    // Modified column
    let modified = formatters::format_date(Some(entry.modified));
    spans.push(Span::styled(
        formatters::pad_text(&modified, layout.modified_width, Alignment::Center),
        style,
    ));
    
    // Created column (if visible)
    if layout.show_created {
        spans.push(Span::raw("  "));
        let created = formatters::format_date(entry.created);
        spans.push(Span::styled(
            formatters::pad_text(&created, layout.created_width, Alignment::Center),
            style,
        ));
    }
    
    // Permissions column (if visible)
    if layout.show_permissions {
        spans.push(Span::raw("  "));
        let perms = formatters::format_permissions(entry);
        spans.push(Span::styled(
            formatters::pad_text(&perms, layout.perms_width, Alignment::Center),
            style,
        ));
    }
    
    Line::from(spans)
}
