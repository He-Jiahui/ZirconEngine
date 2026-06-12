use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::shared::{
    color_attribute, icon_command, line_height, number_attribute, quad_command, row_label,
    string_attribute, text_command, RowRenderState, ACCENT, FONT_SIZE, SURFACE_HOVER,
    SURFACE_PRESSED, SURFACE_SELECTED, TEXT_DISABLED, TEXT_MUTED, TEXT_SELECTED,
};

const BASE_INSET_X: f32 = 12.0;
const DISCLOSURE_SIZE: f32 = 12.0;
const ICON_SIZE: f32 = 14.0;
const TEXT_GAP: f32 = 7.0;
const ACTION_SIZE: f32 = 14.0;
const ACTION_GAP: f32 = 16.0;
const RIGHT_INSET: f32 = 12.0;
const GUIDE_STEP: f32 = 18.0;
const RADIUS: f32 = 5.0;
const GUIDE: &str = "#2a3740";

pub(super) fn tree_row_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &RowRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = Vec::new();
    if let Some(background) = background(state) {
        commands.push(quad_command(
            node_id,
            frame,
            clip_frame,
            z_index.saturating_add(1),
            background,
            border(state),
            border_width(state),
            RADIUS,
            state,
            opacity,
        ));
    }
    for level in 0..depth(metadata) {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                frame.x + BASE_INSET_X + 5.0 + level as f32 * GUIDE_STEP,
                frame.y - 1.0,
                1.0,
                frame.height + 2.0,
            ),
            clip_frame,
            z_index.saturating_add(2),
            GUIDE,
            None,
            0.0,
            0.0,
            state,
            opacity * 0.78,
        ));
    }
    let disclosure = disclosure_rect(metadata, frame);
    commands.push(icon_command(
        node_id,
        disclosure,
        clip_frame,
        z_index.saturating_add(3),
        if state.expanded() {
            "chevron-down"
        } else {
            "chevron-right"
        },
        secondary(state),
        state,
        opacity,
    ));
    let object = UiFrame::new(
        disclosure.x + disclosure.width + 4.0,
        disclosure.y + (disclosure.height - ICON_SIZE).max(0.0) * 0.5,
        ICON_SIZE,
        ICON_SIZE,
    );
    commands.push(icon_command(
        node_id,
        object,
        clip_frame,
        z_index.saturating_add(4),
        icon_name(metadata),
        icon_color(metadata, state),
        state,
        opacity,
    ));
    if let Some(label) = row_label(metadata) {
        let text_x = object.x + object.width + TEXT_GAP;
        let right_reserve = RIGHT_INSET + ACTION_SIZE * 2.0 + ACTION_GAP;
        commands.push(text_command(
            node_id,
            UiFrame::new(
                text_x,
                frame.y + (frame.height - line_height(FONT_SIZE)).max(0.0) * 0.5,
                (frame.x + frame.width - text_x - right_reserve).max(1.0),
                line_height(FONT_SIZE),
            ),
            clip_frame,
            z_index.saturating_add(5),
            label,
            text(state),
            FONT_SIZE,
            state,
            opacity,
        ));
    }
    commands.push(icon_command(
        node_id,
        action_rect(frame, 1),
        clip_frame,
        z_index.saturating_add(6),
        "eye",
        action(state),
        state,
        opacity,
    ));
    commands.push(icon_command(
        node_id,
        action_rect(frame, 0),
        clip_frame,
        z_index.saturating_add(7),
        if state.marked() {
            "more-horizontal"
        } else {
            "lock"
        },
        action(state),
        state,
        opacity,
    ));
    commands
}

fn background(state: &RowRenderState) -> Option<&'static str> {
    if state.unavailable() {
        None
    } else if state.marked() {
        Some(SURFACE_SELECTED)
    } else if state.pressed() {
        Some(SURFACE_PRESSED)
    } else if state.hot() {
        Some(SURFACE_HOVER)
    } else {
        None
    }
}

fn border(state: &RowRenderState) -> Option<&'static str> {
    (!state.unavailable() && (state.focus_or_press() || state.marked())).then_some(ACCENT)
}

fn border_width(state: &RowRenderState) -> f32 {
    if border(state).is_some() {
        1.0
    } else {
        0.0
    }
}

fn text(state: &RowRenderState) -> &'static str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if state.marked() {
        TEXT_SELECTED
    } else {
        "#a8b2b7"
    }
}

fn secondary(state: &RowRenderState) -> &'static str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if state.marked() {
        TEXT_SELECTED
    } else {
        TEXT_MUTED
    }
}

fn action(state: &RowRenderState) -> &'static str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if state.marked() {
        TEXT_SELECTED
    } else {
        "#9cadb6"
    }
}

fn icon_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &RowRenderState) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if state.marked() {
        TEXT_SELECTED
    } else {
        color_attribute(metadata, "icon_color").unwrap_or(TEXT_MUTED)
    }
}

fn depth(metadata: &UiTemplateNodeMetadata) -> usize {
    number_attribute(metadata, "tree_depth")
        .or_else(|| number_attribute(metadata, "depth"))
        .unwrap_or(0.0)
        .max(0.0) as usize
}

fn disclosure_rect(metadata: &UiTemplateNodeMetadata, frame: UiFrame) -> UiFrame {
    let indent = number_attribute(metadata, "tree_indent_px")
        .filter(|indent| indent.is_finite() && *indent > 0.0)
        .unwrap_or_else(|| depth(metadata) as f32 * GUIDE_STEP);
    UiFrame::new(
        frame.x + BASE_INSET_X + indent,
        frame.y + (frame.height - DISCLOSURE_SIZE).max(0.0) * 0.5,
        DISCLOSURE_SIZE,
        DISCLOSURE_SIZE,
    )
}

fn action_rect(frame: UiFrame, index_from_right: usize) -> UiFrame {
    let stride = ACTION_SIZE + ACTION_GAP;
    UiFrame::new(
        frame.x + frame.width - RIGHT_INSET - ACTION_SIZE - index_from_right as f32 * stride,
        frame.y + (frame.height - ACTION_SIZE).max(0.0) * 0.5,
        ACTION_SIZE,
        ACTION_SIZE,
    )
}

fn icon_name(metadata: &UiTemplateNodeMetadata) -> &str {
    let label = row_label(metadata).unwrap_or_default().to_ascii_lowercase();
    let control = metadata
        .control_id
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if label.contains("audio") || control.contains("audio") {
        "volume-2"
    } else if label.contains("player") || control.contains("player") {
        "play"
    } else {
        string_attribute(metadata, "icon").unwrap_or("box")
    }
}
