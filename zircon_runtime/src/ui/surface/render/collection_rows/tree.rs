use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, style::UiRgbaColor, surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use super::shared::{
    icon_command, number_attribute, quad_command, row_label, string_attribute, text_command,
    CollectionRowVisual, RowRenderState,
};

pub(super) fn tree_row_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &RowRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let visual = CollectionRowVisual::resolve(metadata);
    let mut commands = Vec::new();
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
    for level in 0..depth(metadata) {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                frame.x
                    + visual.inline_inset
                    + visual.border_width
                    + level as f32 * visual.tree_indent,
                frame.y,
                visual.border_width.max(f32::EPSILON),
                frame.height,
            ),
            clip_frame,
            z_index.saturating_add(2),
            visual.separator,
            None,
            0.0,
            0.0,
            state,
            opacity * 0.78,
        ));
    }
    let disclosure = disclosure_rect(metadata, frame, &visual);
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
        secondary(&visual, state),
        state,
        opacity,
    ));
    let object = UiFrame::new(
        disclosure.x + disclosure.width + visual.compact_inset,
        disclosure.y + (disclosure.height - visual.action_size).max(0.0) * 0.5,
        visual.action_size,
        visual.action_size,
    );
    commands.push(icon_command(
        node_id,
        object,
        clip_frame,
        z_index.saturating_add(4),
        icon_name(metadata),
        icon_color(&visual, state),
        state,
        opacity,
    ));
    if let Some(label) = row_label(metadata) {
        let text_x = object.x + object.width + visual.compact_inset;
        let right_reserve = visual.inline_inset + visual.action_size * 2.0 + visual.action_gap;
        let text_line_height = visual.line_height(visual.body_font_size);
        commands.push(text_command(
            node_id,
            UiFrame::new(
                text_x,
                frame.y + (frame.height - text_line_height).max(0.0) * 0.5,
                (frame.x + frame.width - text_x - right_reserve).max(1.0),
                text_line_height.min(frame.height).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(5),
            label.to_string(),
            text(&visual, state),
            visual.body_font_size,
            text_line_height,
            state,
            opacity,
        ));
    }
    commands.push(icon_command(
        node_id,
        action_rect(frame, 1, &visual),
        clip_frame,
        z_index.saturating_add(6),
        "eye",
        action(&visual, state),
        state,
        opacity,
    ));
    commands.push(icon_command(
        node_id,
        action_rect(frame, 0, &visual),
        clip_frame,
        z_index.saturating_add(7),
        if state.marked() {
            "more-horizontal"
        } else {
            "lock"
        },
        action(&visual, state),
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

fn secondary(visual: &CollectionRowVisual, state: &RowRenderState) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if state.marked() {
        visual.icon_selected
    } else {
        visual.icon_secondary
    }
}

fn action(visual: &CollectionRowVisual, state: &RowRenderState) -> UiRgbaColor {
    secondary(visual, state)
}

fn icon_color(visual: &CollectionRowVisual, state: &RowRenderState) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if state.marked() {
        visual.icon_selected
    } else {
        visual.icon_secondary
    }
}

fn depth(metadata: &UiTemplateNodeMetadata) -> usize {
    number_attribute(metadata, "tree_depth")
        .or_else(|| number_attribute(metadata, "depth"))
        .unwrap_or(0.0)
        .max(0.0) as usize
}

fn disclosure_rect(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    visual: &CollectionRowVisual,
) -> UiFrame {
    let indent = number_attribute(metadata, "tree_indent_px")
        .filter(|indent| indent.is_finite() && *indent > 0.0)
        .unwrap_or_else(|| depth(metadata) as f32 * visual.tree_indent);
    UiFrame::new(
        frame.x + visual.inline_inset + indent,
        frame.y + (frame.height - visual.action_size).max(0.0) * 0.5,
        visual.action_size,
        visual.action_size,
    )
}

fn action_rect(frame: UiFrame, index_from_right: usize, visual: &CollectionRowVisual) -> UiFrame {
    let stride = visual.action_size + visual.action_gap;
    UiFrame::new(
        frame.x + frame.width
            - visual.inline_inset
            - visual.action_size
            - index_from_right as f32 * stride,
        frame.y + (frame.height - visual.action_size).max(0.0) * 0.5,
        visual.action_size,
        visual.action_size,
    )
}

fn icon_name(metadata: &UiTemplateNodeMetadata) -> &str {
    let label = row_label(metadata).unwrap_or_default();
    let control = metadata.control_id.as_deref().unwrap_or_default();
    if contains_ascii_case(label, "audio") || contains_ascii_case(control, "audio") {
        "volume-2"
    } else if contains_ascii_case(label, "player") || contains_ascii_case(control, "player") {
        "play"
    } else {
        string_attribute(metadata, "icon").unwrap_or("box")
    }
}

fn contains_ascii_case(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
