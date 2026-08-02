use super::super::pane_payload::{HierarchyPaneNodePayload, HierarchyPanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    PanePayload::HierarchyV1(HierarchyPanePayload {
        nodes: context
            .chrome
            .scene_entries
            .iter()
            .map(|entry| HierarchyPaneNodePayload {
                node_id: entry.entity,
                name: entry.display_name.clone(),
                depth: entry.depth as u32,
                selected: context.chrome.scene_entries.is_selected(entry.entity),
            })
            .collect(),
    })
}
