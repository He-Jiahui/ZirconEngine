mod commands;
mod content;
mod metadata;
mod metrics;
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
    content::chrome_commands, metadata::chrome_kind, metrics::ChromeMetrics,
    state::ChromeRenderState,
};

pub(super) fn chrome_suppresses_owner_surface(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| chrome_kind(metadata).is_some())
}

pub(super) fn chrome_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| chrome_kind(metadata).is_some())
}

pub(super) fn chrome_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| chrome_kind(metadata).is_some())
}

pub(super) fn chrome_render_commands(
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
    let Some(kind) = chrome_kind(metadata) else {
        return Vec::new();
    };
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = ChromeRenderState::resolve(metadata, state_flags, component_state);
    let metrics = ChromeMetrics::resolve(metadata);
    chrome_commands(
        node_id, metadata, kind, &state, metrics, frame, clip_frame, z_index, opacity,
    )
}
