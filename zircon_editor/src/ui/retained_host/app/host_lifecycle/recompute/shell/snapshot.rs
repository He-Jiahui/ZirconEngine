use crate::ui::retained_host::callback_dispatch;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewDescriptor;

pub(in crate::ui::retained_host::app::host_lifecycle::recompute) struct RecomputeShellSnapshot {
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) layout: WorkbenchLayout,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) chrome: EditorChromeSnapshot,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) model: WorkbenchViewModel,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) geometry:
        WorkbenchShellGeometry,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) componentized_workbench_layout_frames:
        callback_dispatch::BuiltinWorkbenchWindowLayoutFrames,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) reuse_shell_layout: bool,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) descriptors:
        Vec<ViewDescriptor>,
}
