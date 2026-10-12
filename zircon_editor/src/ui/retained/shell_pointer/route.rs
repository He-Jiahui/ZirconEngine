use crate::ui::retained_host::drawer_resize::HostResizeTargetGroup;
use crate::ui::retained_host::tab_drag::HostDragTargetGroup;
use crate::ui::workbench::layout::DockEdge;
use crate::ui::workbench::layout::MainPageId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HostShellPointerRoute {
    DragTarget(HostDragTargetGroup),
    DocumentEdge(DockEdge),
    FloatingWindow(MainPageId),
    FloatingWindowEdge {
        window_id: MainPageId,
        edge: DockEdge,
    },
    Resize(HostResizeTargetGroup),
}
