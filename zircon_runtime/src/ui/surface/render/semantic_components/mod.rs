mod agent_workflow;
mod data_surfaces;
mod shared;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommand, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

pub(super) fn semantic_component_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    agent_workflow::supports(metadata) || data_surfaces::supports(metadata)
}

pub(super) fn semantic_component_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if !shared::valid_frame(frame) {
        return Vec::new();
    }
    if agent_workflow::supports(metadata) {
        return agent_workflow::render(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        );
    }
    if data_surfaces::supports(metadata) {
        return data_surfaces::render(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        );
    }
    Vec::new()
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
