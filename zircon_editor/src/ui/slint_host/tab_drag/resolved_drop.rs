use crate::ui::workbench::layout::SplitAxis;
use crate::ui::workbench::layout::SplitPlacement;
use crate::ui::workbench::layout::TabInsertionAnchor;
use crate::ui::workbench::layout::WorkspaceTarget;
use crate::ui::workbench::view::ViewHost;

use super::group::WorkbenchDragTargetGroup;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedTabDrop {
    pub host: ViewHost,
    pub anchor: Option<TabInsertionAnchor>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedWorkbenchTabDropRoute {
    pub target_group: WorkbenchDragTargetGroup,
    pub target_label: &'static str,
    pub target: ResolvedWorkbenchTabDropTarget,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ResolvedWorkbenchTabDropTarget {
    Attach(ResolvedTabDrop),
    Split {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        axis: SplitAxis,
        placement: SplitPlacement,
    },
}
