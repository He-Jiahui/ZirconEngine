use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{icon_command, surface_command, text_command},
    metadata::{button_label, icon_name},
    state::ButtonRenderState,
    style::{ButtonVisual, foreground_color},
};

pub(super) fn button_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &ButtonRenderState,
    visual: &ButtonVisual,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = vec![surface_command(
        node_id, frame, clip_frame, z_index, state, visual, opacity,
    )];
    let icon = icon_name(metadata);
    let label = button_label(metadata);
    let text_width = (frame.width
        - visual.padding_left
        - visual.padding_right
        - icon
            .as_ref()
            .map(|_| visual.icon_size + visual.spacing)
            .unwrap_or(0.0))
    .max(visual.min_frame_extent);
    let mut cursor_x = frame.x + visual.padding_left;
    if let Some(icon) = icon {
        let icon_frame = UiFrame::new(
            cursor_x,
            frame.y + (frame.height - visual.icon_size).max(0.0) * 0.5,
            visual.icon_size,
            visual.icon_size,
        );
        commands.push(icon_command(
            node_id,
            icon_frame,
            clip_frame,
            z_index.saturating_add(3),
            icon,
            foreground_color(state, visual),
            state,
            opacity,
        ));
        cursor_x += visual.icon_size + visual.spacing;
    }
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                cursor_x,
                frame.y + (frame.height - visual.line_height).max(0.0) * 0.5,
                text_width,
                visual.line_height.min(frame.height),
            ),
            clip_frame,
            z_index.saturating_add(4),
            label,
            foreground_color(state, visual),
            visual.font_size,
            visual.line_height,
            state,
            opacity,
        ));
    }
    commands
}
