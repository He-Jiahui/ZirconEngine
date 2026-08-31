mod commands;
mod metadata;
mod segments;
mod state;
mod style;
mod tabs;

use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use metadata::{SegmentedControlKind, control_kind, is_segmented_or_tab};
use segments::segmented_commands;
use state::SegmentedRenderState;
use style::SegmentedVisual;
use tabs::tab_commands;

pub(super) fn segmented_control_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_segmented_or_tab)
}

pub(super) fn segmented_control_render_commands(
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
    let Some(kind) = control_kind(metadata) else {
        return Vec::new();
    };
    let visual = SegmentedVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let state = SegmentedRenderState::resolve(metadata, state_flags, component_state);
    match kind {
        SegmentedControlKind::SegmentedControl => segmented_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
        SegmentedControlKind::Tab => tab_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
    }
}
