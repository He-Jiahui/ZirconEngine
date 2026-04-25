use super::super::pane_payload::{InspectorPanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let inspector = context.chrome.inspector.as_ref();
    PanePayload::InspectorV1(InspectorPanePayload {
        node_id: inspector.map(|inspector| inspector.id).unwrap_or_default(),
        name: inspector
            .map(|inspector| inspector.name.clone())
            .unwrap_or_default(),
        parent: inspector
            .map(|inspector| inspector.parent.clone())
            .unwrap_or_default(),
        translation: inspector
            .map(|inspector| inspector.translation.clone())
            .unwrap_or_else(|| Default::default()),
        delete_enabled: inspector.is_some(),
    })
}
