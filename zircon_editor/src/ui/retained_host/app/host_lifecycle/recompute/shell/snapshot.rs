use std::sync::Arc;

use crate::ui::layouts::windows::workbench_host_window::ShellPresentation;
use crate::ui::retained_host::app::committed_shell_state::HostLifecyclePanePayloads;
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
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) retained_pane_payloads:
        Option<Arc<HostLifecyclePanePayloads>>,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) retained_shell_presentation:
        Option<Arc<ShellPresentation>>,
}
