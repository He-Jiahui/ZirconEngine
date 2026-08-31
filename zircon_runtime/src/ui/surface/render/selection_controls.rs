mod checkbox;
mod commands;
mod geometry;
mod metadata;
mod radio;
mod state;
mod style;
mod toggle;

use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use checkbox::checkbox_commands;
use metadata::{SelectionControlKind, selection_control_kind};
use radio::radio_commands;
use state::SelectionRenderState;
use style::SelectionVisual;
use toggle::toggle_commands;

pub(super) fn selection_control_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.and_then(selection_control_kind).is_some()
}

pub(super) fn selection_control_render_commands(
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
    let Some(kind) = selection_control_kind(metadata) else {
        return Vec::new();
    };
    let visual = SelectionVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let state = SelectionRenderState::resolve(metadata, state_flags, component_state);
    match kind {
        SelectionControlKind::Checkbox => checkbox_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
        SelectionControlKind::Radio => radio_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
        SelectionControlKind::Toggle => toggle_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
    }
}
