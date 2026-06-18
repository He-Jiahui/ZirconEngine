use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue, event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use super::shared::{
    bool_attribute, color_attribute, icon_command, quad_command, row_label, text_command,
    RowRenderState, ACCENT, SURFACE_DISABLED, SURFACE_HOVER, SURFACE_PRESSED, SURFACE_SELECTED,
    TABLE_FONT_SIZE, TEXT, TEXT_DISABLED, TEXT_MUTED,
};

const TEXT_INSET_X: f32 = 9.0;
const TEXT_INSET_Y: f32 = 4.0;
const ACTION_WIDTH: f32 = 24.0;
const ACTION_SIZE: f32 = 14.0;
const RADIUS: f32 = 3.0;
const COLUMN_RATIOS: [f32; 4] = [0.36, 0.27, 0.19, 0.18];
const ROW_BG: &str = "#0d1114";
const HEADER_BG: &str = "#0c1013";
const TAIL_BG: &str = "#0e1215";
const SEPARATOR: &str = "#1c2429";

pub(super) fn table_row_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &RowRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let cells = table_cells(metadata);
    if cells.is_empty() {
        return Vec::new();
    }
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        background(metadata, state),
        border(state),
        border_width(state),
        RADIUS,
        state,
        opacity,
    )];
    commands.push(quad_command(
        node_id,
        UiFrame::new(
            frame.x,
            frame.y + (frame.height - 1.0).max(0.0),
            frame.width,
            1.0,
        ),
        clip_frame,
        z_index.saturating_add(2),
        SEPARATOR,
        None,
        0.0,
        0.0,
        state,
        opacity,
    ));
    for (index, cell) in cells.iter().take(COLUMN_RATIOS.len()).enumerate() {
        commands.push(text_command(
            node_id,
            cell_rect(frame, index),
            clip_frame,
            z_index.saturating_add(4),
            cell.clone(),
            text(metadata, state, index),
            TABLE_FONT_SIZE,
            state,
            opacity,
        ));
    }
    commands.push(icon_command(
        node_id,
        UiFrame::new(
            frame.x + frame.width - ACTION_WIDTH + 7.0,
            frame.y + (frame.height - ACTION_SIZE).max(0.0) * 0.5,
            ACTION_SIZE,
            ACTION_SIZE,
        ),
        clip_frame,
        z_index.saturating_add(5),
        if is_header(metadata) {
            "settings"
        } else {
            "more-horizontal"
        },
        action(state),
        state,
        opacity,
    ));
    commands
}

fn background<'a>(metadata: &'a UiTemplateNodeMetadata, state: &RowRenderState) -> &'a str {
    if state.unavailable() {
        SURFACE_DISABLED
    } else if state.marked() {
        color_attribute(metadata, "background_color").unwrap_or(SURFACE_SELECTED)
    } else if state.pressed() {
        SURFACE_PRESSED
    } else if state.hot() {
        SURFACE_HOVER
    } else if is_header(metadata) {
        HEADER_BG
    } else if is_tail(metadata) {
        TAIL_BG
    } else {
        color_attribute(metadata, "background_color").unwrap_or(ROW_BG)
    }
}

fn border(state: &RowRenderState) -> Option<&'static str> {
    (!state.unavailable() && state.focus_or_press()).then_some(ACCENT)
}

fn border_width(state: &RowRenderState) -> f32 {
    if border(state).is_some() {
        1.0
    } else {
        0.0
    }
}

fn text<'a>(metadata: &'a UiTemplateNodeMetadata, state: &RowRenderState, index: usize) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if is_header(metadata) {
        "#aab5ba"
    } else if is_tail(metadata) && index == 3 {
        color_attribute(metadata, "value_color").unwrap_or("#aab5ba")
    } else if index >= 2 {
        TEXT_MUTED
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TEXT)
    }
}

fn action(state: &RowRenderState) -> &'static str {
    if state.unavailable() {
        TEXT_DISABLED
    } else {
        TEXT_MUTED
    }
}

fn table_cells(metadata: &UiTemplateNodeMetadata) -> Vec<String> {
    ["cells", "columns", "options"]
        .iter()
        .find_map(|key| metadata.attributes.get(*key).and_then(Value::as_array))
        .map(|values| {
            values
                .iter()
                .filter_map(value_text)
                .filter(|text| !text.trim().is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|cells| !cells.is_empty())
        .or_else(|| row_label(metadata).map(|label| split_row_label_table_text(label.as_str())))
        .unwrap_or_default()
}

fn value_text(value: &Value) -> Option<String> {
    match value {
        Value::Table(table) => ["label", "text", "value", "name", "title"]
            .iter()
            .find_map(|key| table.get(*key))
            .map(UiValue::from_toml)
            .map(|value| value.display_text()),
        value => Some(UiValue::from_toml(value).display_text()),
    }
}

fn split_row_label_table_text(text: &str) -> Vec<String> {
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    match tokens.as_slice() {
        [] => Vec::new(),
        [name, kind, size, size_unit, modified_value, modified_unit, ..] => vec![
            (*name).to_string(),
            (*kind).to_string(),
            format!("{size} {size_unit}"),
            format!("{modified_value} {modified_unit}"),
        ],
        [name, kind, size, modified, ..] => vec![
            (*name).to_string(),
            (*kind).to_string(),
            (*size).to_string(),
            (*modified).to_string(),
        ],
        _ => vec![text.trim().to_string()],
    }
}

fn cell_rect(frame: UiFrame, index: usize) -> UiFrame {
    let mut x = frame.x + TEXT_INSET_X;
    let available_width = (frame.width - TEXT_INSET_X * 2.0 - ACTION_WIDTH).max(1.0);
    for ratio in COLUMN_RATIOS.iter().take(index) {
        x += available_width * ratio;
    }
    UiFrame::new(
        x,
        frame.y + TEXT_INSET_Y,
        COLUMN_RATIOS
            .get(index)
            .map(|ratio| available_width * ratio)
            .unwrap_or(available_width)
            .max(1.0),
        (frame.height - TEXT_INSET_Y * 2.0).max(1.0),
    )
}

fn is_header(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.control_id.as_deref() == Some("WorkbenchTableHeader")
        || bool_attribute(metadata, "header").unwrap_or(false)
}

fn is_tail(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.control_id.as_deref() == Some("WorkbenchTableTail")
        || bool_attribute(metadata, "tail").unwrap_or(false)
}
