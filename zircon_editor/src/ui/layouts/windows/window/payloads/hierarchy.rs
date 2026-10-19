use super::super::pane_payload::{HierarchyPaneNodePayload, HierarchyPanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let entries = super::super::hierarchy_projection::projected_hierarchy_entries(context.chrome);
    PanePayload::HierarchyV1(HierarchyPanePayload {
        nodes: entries
            .iter()
            .map(|entry| HierarchyPaneNodePayload {
                node_id: entry.entity,
                name: entry.display_name.clone(),
                depth: entry.depth as u32,
                selected: entries.is_selected(entry.entity),
            })
            .collect(),
    })
}
