use zircon_runtime::ui::layout::UiPoint;

use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::ui::slint_host::shell_pointer::WorkbenchShellPointerBridge;
use crate::{ShellSizePx, WorkbenchShellGeometry};

use super::group::WorkbenchDragTargetGroup;

pub(crate) struct WorkbenchDragTargetBridge {
    shell_pointer: WorkbenchShellPointerBridge,
}

impl Default for WorkbenchDragTargetBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkbenchDragTargetBridge {
    pub(crate) fn new() -> Self {
        Self {
            shell_pointer: WorkbenchShellPointerBridge::new(),
        }
    }

    pub(crate) fn update_layout_with_root_frames(
        &mut self,
        root_size: ShellSizePx,
        geometry: &WorkbenchShellGeometry,
        drawers_visible: bool,
        shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
    ) {
        self.shell_pointer.update_layout_with_root_shell_frames(
            root_size,
            geometry,
            drawers_visible,
            &[],
            shared_root_frames,
            None,
        );
    }

    pub(crate) fn resolve(&mut self, point: UiPoint) -> Option<WorkbenchDragTargetGroup> {
        self.shell_pointer.drag_target_at(point)
    }
}

pub fn resolve_workbench_drag_target_group(
    root_size: ShellSizePx,
    geometry: &WorkbenchShellGeometry,
    drawers_visible: bool,
    point: UiPoint,
) -> Option<WorkbenchDragTargetGroup> {
    resolve_workbench_drag_target_group_with_root_frames(
        root_size,
        geometry,
        drawers_visible,
        point,
        None,
    )
}

pub fn resolve_workbench_drag_target_group_with_root_frames(
    root_size: ShellSizePx,
    geometry: &WorkbenchShellGeometry,
    drawers_visible: bool,
    point: UiPoint,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<WorkbenchDragTargetGroup> {
    let mut bridge = WorkbenchDragTargetBridge::new();
    bridge.update_layout_with_root_frames(root_size, geometry, drawers_visible, shared_root_frames);
    bridge.resolve(point)
}
