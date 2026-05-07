use zircon_runtime_interface::ui::layout::UiPoint;

use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::slint_host::shell_pointer::HostShellPointerBridge;
use crate::ui::workbench::autolayout::ShellSizePx;

use super::group::HostDragTargetGroup;

pub(crate) struct HostDragTargetBridge {
    shell_pointer: HostShellPointerBridge,
}

impl Default for HostDragTargetBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl HostDragTargetBridge {
    pub(crate) fn new() -> Self {
        Self {
            shell_pointer: HostShellPointerBridge::new(),
        }
    }

    pub(crate) fn update_layout_with_root_frames(
        &mut self,
        root_size: ShellSizePx,
        drawers_visible: bool,
        shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    ) {
        self.shell_pointer.update_layout_with_root_shell_frames(
            root_size,
            drawers_visible,
            &[],
            shared_root_frames,
            None,
        );
    }

    pub(crate) fn resolve(&mut self, point: UiPoint) -> Option<HostDragTargetGroup> {
        self.shell_pointer.drag_target_at(point)
    }
}

pub fn resolve_host_drag_target_group(
    root_size: ShellSizePx,
    drawers_visible: bool,
    point: UiPoint,
) -> Option<HostDragTargetGroup> {
    resolve_host_drag_target_group_with_root_frames(root_size, drawers_visible, point, None)
}

pub fn resolve_host_drag_target_group_with_root_frames(
    root_size: ShellSizePx,
    drawers_visible: bool,
    point: UiPoint,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<HostDragTargetGroup> {
    let mut bridge = HostDragTargetBridge::new();
    bridge.update_layout_with_root_frames(root_size, drawers_visible, shared_root_frames);
    bridge.resolve(point)
}
