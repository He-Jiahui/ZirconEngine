mod list;
mod shared;
mod table;
mod tree;

use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    surface::UiRenderCommand,
    tree::UiTemplateNodeMetadata,
};

use self::shared::{collection_row_kind, CollectionRowKind, RowRenderState};

pub(super) fn collection_row_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(|metadata| collection_row_kind(metadata).is_some())
}

pub(super) fn collection_row_suppresses_owner_image(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(|metadata| collection_row_kind(metadata).is_some())
}

pub(super) fn collection_row_render_commands(
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
    let Some(kind) = collection_row_kind(metadata) else {
        return Vec::new();
    };
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = RowRenderState::resolve(kind, metadata, state_flags, component_state);
    match kind {
        CollectionRowKind::List => list::list_row_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
        CollectionRowKind::Tree => tree::tree_row_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
        CollectionRowKind::Table => table::table_row_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
    }
}
