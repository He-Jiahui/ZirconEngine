use crate::ui::workbench::layout::WorkspaceTarget;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::pane_empty_state_model::PaneEmptyStateModel;

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentTabModel {
    pub workspace: WorkspaceTarget,
    pub workspace_path: Vec<usize>,
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub icon_key: String,
    pub content_kind: ViewContentKind,
    pub active: bool,
    pub closeable: bool,
    pub empty_state: Option<PaneEmptyStateModel>,
}
