// Panel widget rendering
use crate::models::panel::Panel;
use crate::models::selection::SelectionState;
use crate::app::PanelSide;
use crate::ui::theme::{self, Theme};
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
    theme: &Theme, // US5: Pass theme for customization
) {
    // T560: Add selection counter in header
    let selected_count = selection_state.count(panel_side);
    let title = if selected_count > 0 {
        format!(" {} ({} seleccionados) ", panel.current_path.display(), selected_count)
    } else {
        format!(" {} ", panel.current_path.display())
    };
    
    let border_style = if is_active {
        Style::default().fg(theme.active_border)
    } else {
        Style::default().fg(theme.inactive_border)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .style(Style::default().bg(theme.panel_bg)); // US5: Use theme background


    // Calculate column layout based on available width
    let content_width = area.width.saturating_sub(4); // Subtract borders and padding
    let layout = ColumnLayout::calculate(content_width, &panel.entries);

    // Adjust area height if search bar is active to prevent overlap
    let list_area = if is_active && search_mode {
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(1), // Reserve 1 line for search bar
        }
    } else if is_active && panel.has_filter() {
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(1), // Reserve 1 line for filter display
        }
    } else {
        area
    };

    // T409: Show "No results" message if filter is active but no entries
    let items: Vec<ListItem> = if panel.entries.is_empty() && panel.has_filter() {
        vec![ListItem::new(Line::from(Span::styled(
            format!(" Sin resultados para: {}", panel.get_filter().unwrap_or("")),
            Style::default().fg(theme.error_color),
        )))]
    } else {
        // Build column header as first item
        let mut all_items = Vec::new();
        
        // Header row
        let header_line = build_header_row(&layout, theme);
        all_items.push(ListItem::new(header_line));
        
        // Separator row
        let separator = "─".repeat(content_width as usize);
        all_items.push(ListItem::new(Line::from(Span::styled(
            separator,
            Style::default().fg(theme.inactive_border),
        ))));
        
        // Data rows
        for (idx, entry) in panel.entries.iter().enumerate() {
            let mut style = theme.get_entry_style(&entry.entry_type);
            
            // T558: Show "*" prefix and alternate background for marked items
            let is_marked = selection_state.is_marked(panel_side, &entry.path);
            
            // T559: Add alternate background color for marked items
            if is_marked {
                style = style.bg(theme.marked_bg);
            }
            
            // Check if this is the selected item
            let is_selected = idx == panel.cursor;
            let name_scroll = if is_selected { panel.text_scroll_offset } else { 0 };
            let ext_scroll = if is_selected { panel.ext_scroll_offset } else { 0 };
            let size_scroll = if is_selected { panel.size_scroll_offset } else { 0 };
            let modified_scroll = if is_selected { panel.modified_scroll_offset } else { 0 };
            let created_scroll = if is_selected { panel.created_scroll_offset } else { 0 };
            let perms_scroll = if is_selected { panel.perms_scroll_offset } else { 0 };
            
            // Build columnar row
            let line = build_data_row(
                entry,
                &layout,
                is_marked,
                is_selected,
                name_scroll,
                ext_scroll,
                size_scroll,
                modified_scroll,
                created_scroll,
                perms_scroll,
                style,
            );
            all_items.push(ListItem::new(line));
        }
        
        all_items
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme.highlight_bg)
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.panel_bg).fg(theme.panel_fg)); // US5: Panel colors


    let mut state = ListState::default();
    // Offset cursor by 2 to account for header and separator rows
    let cursor_offset = if panel.entries.is_empty() { 0 } else { panel.cursor + 2 };
    state.select(Some(cursor_offset));

    frame.render_stateful_widget(list, list_area, &mut state);

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
                Style::default().fg(theme.info_color),
            )));
            frame.render_widget(filter_widget, filter_area);
        }
    }
}

/// Build header row with column titles
fn build_header_row(layout: &ColumnLayout, theme: &Theme) -> Line<'static> {
    let mut spans = Vec::new();
    
    // Icon column (empty header)
    spans.push(Span::styled(
        formatters::pad_text("", layout.icon_width, Alignment::Left),
        Style::default().fg(theme.panel_fg).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw("  "));
    
    // Mark column (empty header)
    spans.push(Span::styled(
        formatters::pad_text("", layout.mark_width, Alignment::Left),
        Style::default().fg(theme.panel_fg).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw("  "));
    
    // Name column
    spans.push(Span::styled(
        formatters::pad_text("Name", layout.name_width, Alignment::Left),
        Style::default().fg(theme.panel_fg).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw("  "));
    
    // Extension column (if visible)
    if layout.show_extension {
        spans.push(Span::styled(
            formatters::pad_text("Ext", layout.ext_width, Alignment::Left),
            Style::default().fg(theme.panel_fg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw("  "));
    }
    
    // Size column
    spans.push(Span::styled(
        formatters::pad_text("Size", layout.size_width, Alignment::Right),
        Style::default().fg(theme.panel_fg).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw("  "));
    
    // Modified column
    spans.push(Span::styled(
        formatters::pad_text("Modified", layout.modified_width, Alignment::Center),
        Style::default().fg(theme.panel_fg).add_modifier(Modifier::BOLD),
    ));
    
    // Created column (if visible)
    if layout.show_created {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            formatters::pad_text("Created", layout.created_width, Alignment::Center),
            Style::default().fg(theme.panel_fg).add_modifier(Modifier::BOLD),
        ));
    }
    
    // Permissions column (if visible)
    if layout.show_permissions {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            formatters::pad_text("Perms", layout.perms_width, Alignment::Center),
            Style::default().fg(theme.panel_fg).add_modifier(Modifier::BOLD),
        ));
    }
    
    Line::from(spans)
}

/// Build data row for a file entry with all columns
fn build_data_row(
    entry: &crate::models::file_entry::FileEntry,
    layout: &ColumnLayout,
    is_marked: bool,
    is_selected: bool,
    name_scroll_offset: usize,
    ext_scroll_offset: usize,
    size_scroll_offset: usize,
    modified_scroll_offset: usize,
    created_scroll_offset: usize,
    perms_scroll_offset: usize,
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
    
    // Name column (truncate if too long, or scroll if selected)
    let name = if is_selected && name_scroll_offset > 0 {
        // Apply scroll offset for selected item
        let chars: Vec<char> = entry.name.chars().collect();
        if name_scroll_offset < chars.len() {
            chars[name_scroll_offset..].iter().collect()
        } else {
            entry.name.clone()
        }
    } else {
        entry.name.clone()
    };
    
    spans.push(Span::styled(
        formatters::pad_text(&name, layout.name_width, Alignment::Left),
        style,
    ));
    spans.push(Span::raw("  "));
    
    // Extension column (if visible, with scroll support)
    if layout.show_extension {
        let ext_full = formatters::format_extension(entry);
        let ext = if is_selected && ext_scroll_offset > 0 && !ext_full.is_empty() {
            // Apply scroll offset for selected item's extension
            let chars: Vec<char> = ext_full.chars().collect();
            if ext_scroll_offset < chars.len() {
                chars[ext_scroll_offset..].iter().collect()
            } else {
                ext_full.clone()
            }
        } else {
            ext_full
        };
        
        spans.push(Span::styled(
            formatters::pad_text(&ext, layout.ext_width, Alignment::Left),
            style,
        ));
        spans.push(Span::raw("  "));
    }
    
    // Size column (with scroll support)
    let size_full = formatters::format_size(entry);
    let size = if is_selected && size_scroll_offset > 0 && !size_full.is_empty() {
        let chars: Vec<char> = size_full.chars().collect();
        if size_scroll_offset < chars.len() {
            chars[size_scroll_offset..].iter().collect()
        } else {
            size_full.clone()
        }
    } else {
        size_full
    };
    spans.push(Span::styled(
        formatters::pad_text(&size, layout.size_width, Alignment::Right),
        style,
    ));
    spans.push(Span::raw("  "));
    
    // Modified column (with scroll support)
    let modified_full = formatters::format_date(Some(entry.modified));
    let modified = if is_selected && modified_scroll_offset > 0 && !modified_full.is_empty() {
        let chars: Vec<char> = modified_full.chars().collect();
        if modified_scroll_offset < chars.len() {
            chars[modified_scroll_offset..].iter().collect()
        } else {
            modified_full.clone()
        }
    } else {
        modified_full
    };
    spans.push(Span::styled(
        formatters::pad_text(&modified, layout.modified_width, Alignment::Center),
        style,
    ));
    
    // Created column (if visible, with scroll support)
    if layout.show_created {
        spans.push(Span::raw("  "));
        let created_full = formatters::format_date(entry.created);
        let created = if is_selected && created_scroll_offset > 0 && !created_full.is_empty() {
            let chars: Vec<char> = created_full.chars().collect();
            if created_scroll_offset < chars.len() {
                chars[created_scroll_offset..].iter().collect()
            } else {
                created_full.clone()
            }
        } else {
            created_full
        };
        spans.push(Span::styled(
            formatters::pad_text(&created, layout.created_width, Alignment::Center),
            style,
        ));
    }
    
    // Permissions column (if visible, with scroll support)
    if layout.show_permissions {
        spans.push(Span::raw("  "));
        let perms_full = formatters::format_permissions(entry);
        let perms = if is_selected && perms_scroll_offset > 0 && !perms_full.is_empty() {
            let chars: Vec<char> = perms_full.chars().collect();
            if perms_scroll_offset < chars.len() {
                chars[perms_scroll_offset..].iter().collect()
            } else {
                perms_full.clone()
            }
        } else {
            perms_full
        };
        spans.push(Span::styled(
            formatters::pad_text(&perms, layout.perms_width, Alignment::Center),
            style,
        ));
    }
    
    Line::from(spans)
}
