use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue, event_ui::UiNodeId, layout::UiFrame, style::UiRgbaColor,
    surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::shared::{
    bool_attribute, icon_command, quad_command, row_label, text_command, CollectionRowVisual,
    RowRenderState,
};

const COLUMN_RATIOS: [f32; 4] = [0.36, 0.27, 0.19, 0.18];

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
    let visual = CollectionRowVisual::resolve(metadata);
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        background(&visual, metadata, state),
        border(&visual, state),
        border_width(&visual, state),
        visual.corner_radius,
        state,
        opacity,
    )];
    commands.push(quad_command(
        node_id,
        UiFrame::new(
            frame.x,
            frame.y + (frame.height - visual.border_width).max(0.0),
            frame.width,
            visual.border_width.max(f32::EPSILON),
        ),
        clip_frame,
        z_index.saturating_add(2),
        visual.separator,
        None,
        0.0,
        0.0,
        state,
        opacity,
    ));
    let text_line_height = visual.line_height(visual.caption_font_size);
    for (index, cell) in cells.into_iter().take(COLUMN_RATIOS.len()).enumerate() {
        commands.push(text_command(
            node_id,
            cell_rect(frame, index, &visual, text_line_height),
            clip_frame,
            z_index.saturating_add(4),
            cell,
            text(&visual, metadata, state, index),
            visual.caption_font_size,
            text_line_height,
            state,
            opacity,
        ));
    }
    commands.push(icon_command(
        node_id,
        UiFrame::new(
            frame.x + frame.width - visual.inline_inset - visual.action_size,
            frame.y + (frame.height - visual.action_size).max(0.0) * 0.5,
            visual.action_size,
            visual.action_size,
        ),
        clip_frame,
        z_index.saturating_add(5),
        if is_header(metadata) {
            "settings"
        } else {
            "more-horizontal"
        },
        action(&visual, state),
        state,
        opacity,
    ));
    commands
}

fn background(
    visual: &CollectionRowVisual,
    metadata: &UiTemplateNodeMetadata,
    state: &RowRenderState,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_surface
    } else if state.marked() && state.hot() {
        visual.selected_hover_surface
    } else if state.marked() {
        visual.selected_surface
    } else if state.pressed() {
        visual.pressed_surface
    } else if state.hot() {
        visual.hover_surface
    } else if is_header(metadata) {
        visual.table_header_surface
    } else if is_tail(metadata) {
        visual.table_tail_surface
    } else {
        visual.table_surface
    }
}

fn border(visual: &CollectionRowVisual, state: &RowRenderState) -> Option<UiRgbaColor> {
    (!state.unavailable() && state.focus_or_press()).then_some(visual.focus_border)
}

fn border_width(visual: &CollectionRowVisual, state: &RowRenderState) -> f32 {
    if border(visual, state).is_some() {
        visual.border_width
    } else {
        0.0
    }
}

fn text(
    visual: &CollectionRowVisual,
    metadata: &UiTemplateNodeMetadata,
    state: &RowRenderState,
    index: usize,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if state.marked() {
        visual.text_selected
    } else if is_header(metadata) || (is_tail(metadata) && index == 3) || index >= 2 {
        visual.text_secondary
    } else {
        visual.text_primary
    }
}

fn action(visual: &CollectionRowVisual, state: &RowRenderState) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if state.marked() {
        visual.icon_selected
    } else {
        visual.icon_secondary
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
        .or_else(|| row_label(metadata).map(split_row_label_table_text))
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

fn cell_rect(
    frame: UiFrame,
    index: usize,
    visual: &CollectionRowVisual,
    text_line_height: f32,
) -> UiFrame {
    let mut x = frame.x + visual.inline_inset;
    let action_reserve = visual.inline_inset + visual.action_size + visual.compact_inset;
    let available_width = (frame.width - visual.inline_inset - action_reserve).max(1.0);
    for ratio in COLUMN_RATIOS.iter().take(index) {
        x += available_width * ratio;
    }
    UiFrame::new(
        x,
        frame.y + (frame.height - text_line_height).max(0.0) * 0.5,
        COLUMN_RATIOS
            .get(index)
            .map(|ratio| available_width * ratio)
            .unwrap_or(available_width)
            .max(1.0),
        text_line_height.min(frame.height).max(1.0),
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
