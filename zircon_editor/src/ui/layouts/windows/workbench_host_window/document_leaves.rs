use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::widgets::common::document_tab_data;
use crate::ui::workbench::layout::{DocumentNodeId, SplitAxis};
use crate::ui::workbench::model::{DocumentTabModel, DocumentWorkspaceModel};
use crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot;

#[derive(Clone)]
pub(crate) struct DocumentLeafSurfaceData {
    pub node_id: DocumentNodeId,
    pub relative_frame: FrameRect,
    pub tabs: crate::ui::retained_host::primitives::ModelRc<TabData>,
    pub pane: PaneData,
}

pub(super) fn project_document_leaves(
    model: &WorkbenchViewModel,
    mut project_pane: impl FnMut(&DocumentTabModel) -> PaneData,
) -> Vec<DocumentLeafSurfaceData> {
    let DocumentWorkspaceModel::Workbench { workspace, .. } = &model.document else {
        return Vec::new();
    };
    let mut leaves = Vec::new();
    visit(
        workspace,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        &mut Vec::new(),
        model,
        &mut project_pane,
        &mut leaves,
    );
    leaves
}

fn visit(
    node: &DocumentWorkspaceSnapshot,
    frame: FrameRect,
    path: &mut Vec<usize>,
    model: &WorkbenchViewModel,
    project_pane: &mut impl FnMut(&DocumentTabModel) -> PaneData,
    leaves: &mut Vec<DocumentLeafSurfaceData>,
) {
    match node {
        DocumentWorkspaceSnapshot::Split {
            axis,
            ratio,
            first,
            second,
            ..
        } => {
            let ratio = if ratio.is_finite() {
                ratio.clamp(0.1, 0.9)
            } else {
                0.5
            };
            let mut first_frame = frame.clone();
            let mut second_frame = frame;
            match axis {
                SplitAxis::Horizontal => {
                    first_frame.width *= ratio;
                    second_frame.x += first_frame.width;
                    second_frame.width -= first_frame.width;
                }
                SplitAxis::Vertical => {
                    first_frame.height *= ratio;
                    second_frame.y += first_frame.height;
                    second_frame.height -= first_frame.height;
                }
            }
            path.push(0);
            visit(first, first_frame, path, model, project_pane, leaves);
            path.pop();
            path.push(1);
            visit(second, second_frame, path, model, project_pane, leaves);
            path.pop();
        }
        DocumentWorkspaceSnapshot::Tabs {
            node_id,
            tabs,
            active_tab,
        } => {
            let leaf_tabs: Vec<_> = tabs
                .iter()
                .filter_map(|snapshot| {
                    model.document_tabs.iter().find(|tab| {
                        tab.instance_id == snapshot.instance_id && tab.workspace_path == *path
                    })
                })
                .collect();
            let selected = leaf_tabs
                .iter()
                .copied()
                .find(|tab| Some(&tab.instance_id) == active_tab.as_ref())
                .or_else(|| leaf_tabs.first().copied());
            leaves.push(DocumentLeafSurfaceData {
                node_id: *node_id,
                relative_frame: frame,
                tabs: model_rc(leaf_tabs.iter().map(|tab| document_tab_data(tab)).collect()),
                pane: selected.map(project_pane).unwrap_or_else(blank_pane),
            });
        }
    }
}

#[cfg(test)]
#[path = "tests/document_leaves.rs"]
mod tests;
