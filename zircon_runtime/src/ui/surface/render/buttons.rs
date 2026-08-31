mod button;
mod commands;
mod icon_button;
mod metadata;
mod state;
mod style;

use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use self::{
    button::button_commands,
    icon_button::icon_button_commands,
    metadata::{is_button_component, is_icon_button},
    state::ButtonRenderState,
    style::ButtonVisual,
};

pub(super) fn button_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_button_component)
}

pub(super) fn button_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_button_component)
}

pub(super) fn button_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    state_flags: &UiStateFlags,
    component_state: Option<&UiComponentState>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if !is_button_component(metadata) {
        return Vec::new();
    }

    let visual = ButtonVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let state = ButtonRenderState::resolve(metadata, state_flags, component_state);
    if is_icon_button(metadata) {
        icon_button_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        )
    } else {
        button_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        )
    }
}
