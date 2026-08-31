use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, style::UiRgbaColor, surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use super::shared::{
    CollectionRowVisual, RowRenderState, icon_command, quad_command, row_label, text_command,
};

#[cfg(test)]
mod capacity_tests;

pub(super) fn list_row_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &RowRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let visual = CollectionRowVisual::resolve(metadata);
    let mut commands = Vec::with_capacity(3);
    if let Some(background) = background(&visual, state) {
        commands.push(quad_command(
            node_id,
            frame,
            clip_frame,
            z_index.saturating_add(1),
            background,
            border(&visual, state),
            border_width(&visual, state),
            visual.corner_radius,
            state,
            opacity,
        ));
    }
    let text_line_height = visual.line_height(visual.body_font_size);
    if let Some(label) = row_label(metadata) {
        let adornment_reserve = visual.inline_inset + visual.action_size + visual.compact_inset;
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + visual.inline_inset,
                frame.y + (frame.height - text_line_height).max(0.0) * 0.5,
                (frame.width - visual.inline_inset - adornment_reserve).max(1.0),
                text_line_height.min(frame.height).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(3),
            label.to_string(),
            text(&visual, state),
            visual.body_font_size,
            text_line_height,
            state,
            opacity,
        ));
    }
    let icon = if state.unavailable() {
        "diamond"
    } else if state.marked() {
        "check"
    } else {
        "chevron-right"
    };
    commands.push(icon_command(
        node_id,
        UiFrame::new(
            frame.x + frame.width - visual.inline_inset - visual.action_size,
            frame.y + (frame.height - visual.action_size).max(0.0) * 0.5,
            visual.action_size,
            visual.action_size,
        ),
        clip_frame,
        z_index.saturating_add(4),
        icon,
        adornment(&visual, state),
        state,
        opacity,
    ));
    commands
}

fn background(visual: &CollectionRowVisual, state: &RowRenderState) -> Option<UiRgbaColor> {
    if state.unavailable() {
        None
    } else if state.marked() && state.hot() {
        Some(visual.selected_hover_surface)
    } else if state.marked() {
        Some(visual.selected_surface)
    } else if state.pressed() {
        Some(visual.pressed_surface)
    } else if state.hot() {
        Some(visual.hover_surface)
    } else if state.focus_or_press() {
        Some(visual.focus_surface)
    } else {
        None
    }
}

fn border(visual: &CollectionRowVisual, state: &RowRenderState) -> Option<UiRgbaColor> {
    (!state.unavailable() && (state.focus_or_press() || state.marked()))
        .then_some(visual.focus_border)
}

fn border_width(visual: &CollectionRowVisual, state: &RowRenderState) -> f32 {
    if border(visual, state).is_some() {
        visual.border_width
    } else {
        0.0
    }
}

fn text(visual: &CollectionRowVisual, state: &RowRenderState) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if state.marked() {
        visual.text_selected
    } else {
        visual.text_primary
    }
}

fn adornment(visual: &CollectionRowVisual, state: &RowRenderState) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if state.marked() {
        visual.icon_selected
    } else {
        visual.icon_secondary
    }
}
