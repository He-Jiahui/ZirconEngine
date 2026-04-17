use crate::snapshot::{DocumentWorkspaceSnapshot, ViewTabSnapshot};

pub(super) fn active_view_in_workspace(
    workspace: &DocumentWorkspaceSnapshot,
) -> Option<&ViewTabSnapshot> {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            active_view_in_workspace(first).or_else(|| active_view_in_workspace(second))
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => active_tab
            .as_ref()
            .and_then(|active_id| tabs.iter().find(|tab| &tab.instance_id == active_id))
            .or_else(|| tabs.first()),
    }
}
