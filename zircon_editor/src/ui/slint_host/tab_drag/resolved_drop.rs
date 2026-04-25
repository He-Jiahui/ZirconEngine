use crate::ui::workbench::layout::SplitAxis;
use crate::ui::workbench::layout::SplitPlacement;
use crate::ui::workbench::layout::TabInsertionAnchor;
use crate::ui::workbench::layout::WorkspaceTarget;
use crate::ui::workbench::view::ViewHost;

use super::group::HostDragTargetGroup;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedTabDrop {
    pub host: ViewHost,
    pub anchor: Option<TabInsertionAnchor>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedHostTabDropRoute {
    pub target_group: HostDragTargetGroup,
    pub target_label: &'static str,
    pub target: ResolvedHostTabDropTarget,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ResolvedHostTabDropTarget {
    Attach(ResolvedTabDrop),
    Split {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        axis: SplitAxis,
        placement: SplitPlacement,
    },
}
