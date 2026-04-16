use zircon_ui::UiPoint;

use crate::host::slint_host::shell_pointer::WorkbenchShellPointerBridge;
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

    pub(crate) fn update_layout(
        &mut self,
        root_size: ShellSizePx,
        geometry: &WorkbenchShellGeometry,
        drawers_visible: bool,
    ) {
        self.shell_pointer
            .update_layout(root_size, geometry, drawers_visible);
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
    let mut bridge = WorkbenchDragTargetBridge::new();
    bridge.update_layout(root_size, geometry, drawers_visible);
    bridge.resolve(point)
}
