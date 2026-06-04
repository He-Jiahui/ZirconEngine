use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::shared::{
    color_attribute, icon_command, quad_command, row_label, text_command, RowRenderState, ACCENT,
    FONT_SIZE, SURFACE_HOVER, SURFACE_PRESSED, SURFACE_SELECTED, TEXT, TEXT_DISABLED, TEXT_MUTED,
};

const TEXT_INSET_X: f32 = 14.0;
const TEXT_INSET_Y: f32 = 6.0;
const ADORNMENT_SIZE: f32 = 13.0;
const ADORNMENT_RIGHT: f32 = 12.0;
const ADORNMENT_RESERVE: f32 = 26.0;
const RADIUS: f32 = 4.0;

pub(super) fn list_row_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &RowRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = Vec::new();
    if let Some(background) = background(metadata, state) {
        commands.push(quad_command(
            node_id,
            frame,
            clip_frame,
            z_index.saturating_add(1),
            background,
            border(metadata, state),
            border_width(state),
            RADIUS,
            state,
            opacity,
        ));
    }
    if let Some(label) = row_label(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + TEXT_INSET_X,
                frame.y + TEXT_INSET_Y,
                (frame.width - TEXT_INSET_X - ADORNMENT_RESERVE).max(1.0),
                (frame.height - TEXT_INSET_Y * 2.0).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(3),
            label,
            text(metadata, state),
            FONT_SIZE,
            state,
            opacity,
        ));
    }
    let icon = if state.disabled() {
        "diamond"
    } else if state.marked() {
        "check"
    } else {
        "chevron-right"
    };
    commands.push(icon_command(
        node_id,
        UiFrame::new(
            frame.x + frame.width - ADORNMENT_RIGHT - ADORNMENT_SIZE,
            frame.y + (frame.height - ADORNMENT_SIZE).max(0.0) * 0.5,
            ADORNMENT_SIZE,
            ADORNMENT_SIZE,
        ),
        clip_frame,
        z_index.saturating_add(4),
        icon,
        adornment(metadata, state),
        state,
        opacity,
    ));
    commands
}

fn background<'a>(metadata: &'a UiTemplateNodeMetadata, state: &RowRenderState) -> Option<&'a str> {
    if state.disabled() {
        None
    } else if state.marked() {
        Some(color_attribute(metadata, "background_color").unwrap_or(SURFACE_SELECTED))
    } else if state.pressed() {
        Some(SURFACE_PRESSED)
    } else if state.hot() {
        Some(SURFACE_HOVER)
    } else {
        None
    }
}

fn border<'a>(metadata: &'a UiTemplateNodeMetadata, state: &RowRenderState) -> Option<&'a str> {
    (state.focus_or_press() || state.marked())
        .then(|| color_attribute(metadata, "focus_border_color").unwrap_or(ACCENT))
}

fn border_width(state: &RowRenderState) -> f32 {
    if state.focus_or_press() || state.marked() {
        1.0
    } else {
        0.0
    }
}

fn text<'a>(metadata: &'a UiTemplateNodeMetadata, state: &RowRenderState) -> &'a str {
    if state.disabled() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(if state.marked() {
            TEXT
        } else {
            TEXT_MUTED
        })
    }
}

fn adornment<'a>(metadata: &'a UiTemplateNodeMetadata, state: &RowRenderState) -> &'a str {
    if state.disabled() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "icon_color").unwrap_or(if state.marked() {
            ACCENT
        } else {
            TEXT_MUTED
        })
    }
}
