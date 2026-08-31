use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{icon_command, surface_command},
    metadata::icon_name,
    state::ButtonRenderState,
    style::{ButtonVisual, icon_button_foreground},
};

pub(super) fn icon_button_commands(
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
    let Some(icon) = icon_name(metadata) else {
        return commands;
    };
    commands.push(icon_command(
        node_id,
        UiFrame::new(
            frame.x + (frame.width - visual.icon_button_size).max(0.0) * 0.5,
            frame.y + (frame.height - visual.icon_button_size).max(0.0) * 0.5,
            visual.icon_button_size,
            visual.icon_button_size,
        ),
        clip_frame,
        z_index.saturating_add(3),
        icon,
        icon_button_foreground(state, visual),
        state,
        opacity,
    ));
    commands
}
