use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommand, UiResolvedStyle, UiTextAlign},
    tree::UiTemplateNodeMetadata,
};

use super::shared::{
    bool_attribute, collection_window, component_matches, quad, string_array, text, text_aligned,
    text_attribute, text_color, ACCENT, BORDER, ERROR, INFO, SUCCESS, SURFACE_INSET,
    SURFACE_SELECTED, TEXT_MUTED, TEXT_SECONDARY, WARNING,
};

// These values are local flow spacing tokens. They are never screen coordinates.
const FLOW_INSET: f32 = 8.0;
const FLOW_GAP: f32 = 4.0;

pub(super) fn supports(metadata: &UiTemplateNodeMetadata) -> bool {
    component_matches(
        metadata,
        &["TreeView", "mui-x-tree-view", "DataGrid", "mui-x-data-grid"],
    )
}

pub(super) fn render(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    if component_matches(metadata, &["TreeView", "mui-x-tree-view"]) {
        tree_view(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        )
    } else {
        data_grid(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        )
    }
}

fn tree_view(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let content = flow_content_frame(frame);
    let title = text_attribute(metadata, &["text", "title"]);
    let title_height = title
        .as_ref()
        .map(|_| flow_line_height(base_style, 11.0))
        .unwrap_or_default();
    let rows_top = content.y + title_height + title.as_ref().map_or(0.0, |_| FLOW_GAP);
    let row_baseline = flow_line_height(base_style, 10.0) + FLOW_GAP * 1.5;
    let rows_available = (content.bottom() - rows_top).max(0.0);
    let physical_capacity = (rows_available > 0.0)
        .then(|| {
            ((rows_available + FLOW_GAP) / (row_baseline + FLOW_GAP))
                .floor()
                .max(1.0) as usize
        })
        .unwrap_or_default();
    let items = collection_window(metadata, "collection_items", physical_capacity);
    let row_height = (!items.is_empty()).then_some(row_baseline);
    let mut commands = Vec::new();
    if let Some(title) = title {
        commands.push(text(
            node_id,
            UiFrame::new(content.x, content.y, content.width, title_height),
            clip_frame,
            z_index.saturating_add(1),
            title,
            text_color(base_style),
            11.0,
            base_style,
            opacity,
        ));
    }
    if items.is_empty() {
        if let Some(empty) = text_attribute(metadata, &["empty_text", "value_text"]) {
            commands.push(text(
                node_id,
                UiFrame::new(content.x, rows_top, content.width, rows_available.max(1.0)),
                clip_frame,
                z_index.saturating_add(2),
                empty,
                TEXT_MUTED,
                10.0,
                base_style,
                opacity,
            ));
        }
        return commands;
    }
    let selected_id = text_attribute(metadata, &["selected_id", "selectedId"]);
    let parent_selected = bool_attribute(metadata, "selected").unwrap_or(false);
    for (index, item) in items.into_iter().enumerate() {
        let item = tree_item(&item);
        let row_height = row_height.expect("nonempty item list has a row height");
        let y = rows_top + index as f32 * (row_height + FLOW_GAP);
        let row = UiFrame::new(content.x, y, content.width, row_height);
        let selected = item.state == "selected"
            || selected_id.as_deref().is_some_and(|id| id == item.label)
            || (parent_selected && index == 0);
        if selected {
            commands.push(quad(
                node_id,
                row,
                clip_frame,
                z_index.saturating_add(2 + index as i32 * 3),
                SURFACE_SELECTED,
                Some(INFO.to_string()),
                1.0,
                4.0,
                base_style,
                opacity,
            ));
        }
        let indent = FLOW_GAP + item.depth as f32 * FLOW_GAP * 3.0;
        let marker_extent = (row.height * 0.32).max(FLOW_GAP);
        commands.push(quad(
            node_id,
            UiFrame::new(
                row.x + indent,
                row.y + (row.height - marker_extent) * 0.5,
                marker_extent,
                marker_extent,
            ),
            clip_frame,
            z_index.saturating_add(3 + index as i32 * 3),
            if item.state == "expanded" {
                ACCENT
            } else {
                TEXT_MUTED
            },
            None,
            0.0,
            FLOW_GAP * 0.5,
            base_style,
            opacity,
        ));
        commands.push(text(
            node_id,
            UiFrame::new(
                row.x + indent + FLOW_GAP * 3.25,
                row.y + FLOW_GAP * 0.75,
                (row.right() - row.x - indent - FLOW_GAP * 4.5).max(1.0),
                (row.height - FLOW_GAP * 1.5).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(4 + index as i32 * 3),
            item.label,
            if selected {
                text_color(base_style)
            } else {
                TEXT_SECONDARY.to_string()
            },
            10.0,
            base_style,
            opacity,
        ));
    }
    commands
}

fn data_grid(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let title = text_attribute(metadata, &["text", "title"]);
    let headers = string_array(metadata, "options");
    let content = flow_content_frame(frame);
    let title_height = title
        .as_ref()
        .map(|_| flow_line_height(base_style, 11.0))
        .unwrap_or_default();
    let header_top = content.y + title_height + title.as_ref().map_or(0.0, |_| FLOW_GAP);
    let header_height = flow_line_height(base_style, 9.0) + FLOW_GAP * 1.5;
    let row_baseline = flow_line_height(base_style, 9.0) + FLOW_GAP * 1.5;
    let header = UiFrame::new(content.x, header_top, content.width, header_height);
    let rows_top = header.bottom() + FLOW_GAP;
    let rows_available = (content.bottom() - rows_top).max(0.0);
    let physical_capacity = (rows_available > 0.0)
        .then(|| {
            ((rows_available + FLOW_GAP) / (row_baseline + FLOW_GAP))
                .floor()
                .max(1.0) as usize
        })
        .unwrap_or_default();
    let items = collection_window(metadata, "collection_items", physical_capacity);
    let row_height = (!items.is_empty()).then_some(row_baseline);
    let mut commands = Vec::new();
    if let Some(title) = title {
        commands.push(text(
            node_id,
            UiFrame::new(content.x, content.y, content.width, title_height),
            clip_frame,
            z_index.saturating_add(1),
            title,
            text_color(base_style),
            11.0,
            base_style,
            opacity,
        ));
    }
    commands.push(quad(
        node_id,
        header,
        clip_frame,
        z_index.saturating_add(2),
        SURFACE_INSET,
        Some(BORDER.to_string()),
        1.0,
        3.0,
        base_style,
        opacity,
    ));
    for (index, label) in headers.iter().take(4).enumerate() {
        commands.push(text_aligned(
            node_id,
            cell_frame(header, index, headers.len()),
            clip_frame,
            z_index.saturating_add(3 + index as i32),
            label.clone(),
            TEXT_MUTED,
            9.0,
            column_alignment(metadata, label, index),
            base_style,
            opacity,
        ));
    }
    if items.is_empty() {
        if let Some(empty) = text_attribute(metadata, &["empty_text", "value_text"]) {
            commands.push(text(
                node_id,
                UiFrame::new(content.x, rows_top, content.width, rows_available.max(1.0)),
                clip_frame,
                z_index.saturating_add(8),
                empty,
                TEXT_MUTED,
                10.0,
                base_style,
                opacity,
            ));
        }
        return commands;
    }
    let selected_id = text_attribute(metadata, &["selected_row_id", "selectedRowId"]);
    let parent_selected = bool_attribute(metadata, "selected").unwrap_or(false);
    for (index, item) in items.into_iter().enumerate() {
        let row_data = grid_row(&item);
        let row_height = row_height.expect("nonempty item list has a row height");
        let y = rows_top + index as f32 * (row_height + FLOW_GAP);
        let row = UiFrame::new(content.x, y, content.width, row_height);
        let selected = row_data.state == "selected"
            || selected_id
                .as_deref()
                .is_some_and(|id| row_data.cells.first().is_some_and(|cell| cell == id))
            || (parent_selected && index == 0);
        commands.push(quad(
            node_id,
            row,
            clip_frame,
            z_index.saturating_add(10 + index as i32 * 6),
            if selected {
                SURFACE_SELECTED
            } else {
                SURFACE_INSET
            },
            Some(if selected { INFO } else { BORDER }.to_string()),
            1.0,
            3.0,
            base_style,
            opacity,
        ));
        commands.push(quad(
            node_id,
            UiFrame::new(row.x, row.y, FLOW_GAP * 0.75, row.height),
            clip_frame,
            z_index.saturating_add(11 + index as i32 * 6),
            state_color(&row_data.state),
            None,
            0.0,
            FLOW_GAP * 0.25,
            base_style,
            opacity,
        ));
        for (cell_index, value) in row_data.cells.into_iter().take(4).enumerate() {
            let label = headers
                .get(cell_index)
                .map(String::as_str)
                .unwrap_or_default();
            commands.push(text_aligned(
                node_id,
                cell_frame(row, cell_index, headers.len()),
                clip_frame,
                z_index.saturating_add(12 + index as i32 * 6 + cell_index as i32),
                value,
                if cell_index == 3 {
                    state_color(&row_data.state)
                } else {
                    TEXT_SECONDARY
                },
                9.0,
                column_alignment(metadata, label, cell_index),
                base_style,
                opacity,
            ));
        }
    }
    commands
}

struct TreeItem {
    state: String,
    depth: usize,
    label: String,
}

fn tree_item(raw: &str) -> TreeItem {
    let parts = raw.split('|').map(str::trim).collect::<Vec<_>>();
    TreeItem {
        state: parts
            .first()
            .copied()
            .unwrap_or("normal")
            .to_ascii_lowercase(),
        depth: parts
            .get(1)
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0),
        label: parts.get(2).copied().unwrap_or(raw).to_string(),
    }
}

struct GridRow {
    state: String,
    cells: Vec<String>,
}

fn grid_row(raw: &str) -> GridRow {
    let mut parts = raw.split('|').map(str::trim);
    let state = parts.next().unwrap_or("normal").to_ascii_lowercase();
    let cells = parts.take(4).map(str::to_string).collect();
    GridRow { state, cells }
}

fn cell_frame(row: UiFrame, index: usize, column_count: usize) -> UiFrame {
    const RATIOS: [f32; 4] = [0.38, 0.26, 0.18, 0.18];
    let inner_width = (row.width - FLOW_GAP * 4.0).max(1.0);
    let ratio_total = RATIOS
        .iter()
        .take(column_count.min(RATIOS.len()))
        .sum::<f32>();
    let ratio = RATIOS
        .get(index)
        .copied()
        .unwrap_or(1.0 / column_count.max(1) as f32);
    let scale = if ratio_total > 0.0 {
        1.0 / ratio_total
    } else {
        1.0
    };
    let x = row.x + FLOW_GAP * 2.0 + RATIOS.iter().take(index).sum::<f32>() * scale * inner_width;
    UiFrame::new(
        x,
        row.y + FLOW_GAP * 0.75,
        (ratio * scale * inner_width - FLOW_GAP).max(1.0),
        (row.height - FLOW_GAP * 1.5).max(1.0),
    )
}

fn flow_content_frame(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x + FLOW_INSET,
        frame.y + FLOW_INSET,
        (frame.width - FLOW_INSET * 2.0).max(1.0),
        (frame.height - FLOW_INSET * 2.0).max(1.0),
    )
}

fn flow_line_height(base_style: &UiResolvedStyle, requested_font_size: f32) -> f32 {
    base_style.line_height.max(requested_font_size).max(1.0)
}

fn column_alignment(metadata: &UiTemplateNodeMetadata, label: &str, index: usize) -> UiTextAlign {
    string_array(metadata, "column_alignments")
        .get(index)
        .and_then(|value| parse_column_alignment(value))
        .unwrap_or_else(|| semantic_column_alignment(label))
}

fn parse_column_alignment(value: &str) -> Option<UiTextAlign> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" | "start" => Some(UiTextAlign::Left),
        "right" | "end" => Some(UiTextAlign::Right),
        "center" => Some(UiTextAlign::Center),
        _ => None,
    }
}

fn semantic_column_alignment(label: &str) -> UiTextAlign {
    let label = label.trim().to_ascii_lowercase();
    if [
        "count",
        "size",
        "amount",
        "cost",
        "price",
        "duration",
        "time",
        "date",
        "percent",
        "percentage",
        "tokens",
        "usage",
    ]
    .iter()
    .any(|value| label.contains(value))
    {
        UiTextAlign::Right
    } else if ["status", "state", "health", "result", "severity"]
        .iter()
        .any(|value| label.contains(value))
    {
        UiTextAlign::Center
    } else {
        UiTextAlign::Left
    }
}

fn state_color(state: &str) -> &'static str {
    match state {
        "success" | "ready" => SUCCESS,
        "warning" | "review" => WARNING,
        "failure" | "error" => ERROR,
        "selected" => INFO,
        _ => TEXT_MUTED,
    }
}

#[cfg(test)]
#[path = "tests/data_surfaces.rs"]
mod tests;
