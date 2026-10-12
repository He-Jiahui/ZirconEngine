use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::workbench::autolayout::ShellRegionId;
use crate::ui::workbench::layout::{ActivityDrawerSlot, LayoutCommand};

#[cfg(test)]
use crate::ui::retained_host::callback_dispatch::{
    BuiltinHostRootShellFrames, BuiltinWorkbenchWindowLayoutFrames,
};
#[cfg(test)]
use crate::ui::workbench::autolayout::ShellSizePx;
#[cfg(test)]
use zircon_runtime_interface::ui::layout::UiPoint;

#[cfg(test)]
use super::shell_pointer::HostShellPointerBridge;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostResizeTargetGroup {
    Left,
    Right,
    Bottom,
}

impl HostResizeTargetGroup {
    pub(crate) const fn region(self) -> ShellRegionId {
        match self {
            Self::Left => ShellRegionId::Left,
            Self::Right => ShellRegionId::Right,
            Self::Bottom => ShellRegionId::Bottom,
        }
    }
}

#[cfg(test)]
pub(crate) fn resolve_host_resize_target_group_with_root_frames(
    root_size: ShellSizePx,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    point: UiPoint,
) -> Option<HostResizeTargetGroup> {
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_root_shell_frames(root_size, true, &[], shared_root_frames, None);
    bridge.resize_target_at(point)
}

#[cfg(test)]
pub(crate) fn resolve_host_resize_target_group_with_workbench_layout_frames(
    root_size: ShellSizePx,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    point: UiPoint,
) -> Option<HostResizeTargetGroup> {
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_workbench_layout_frames(
        root_size,
        true,
        &[],
        componentized_workbench_layout_frames,
        None,
    );
    bridge.resize_target_at(point)
}

#[cfg(test)]
use crate::ui::host::EditorManager;

#[cfg(test)]
pub(crate) fn apply_resize_to_group(
    editor_manager: &EditorManager,
    target_group: &str,
    extent: f32,
) -> Result<bool, String> {
    let slot = group_slot(target_group)
        .ok_or_else(|| format!("Unsupported drawer resize target {target_group}"))?;
    editor_manager
        .apply_layout_command(LayoutCommand::SetDrawerRegionExtent { slot, extent })
        .map_err(|error| error.to_string())
}

pub(crate) fn dispatch_resize_to_group(
    runtime: &EditorHostEventController,
    target_group: &str,
    extent: f32,
) -> Result<UiHostEventEffects, String> {
    let slot = group_slot(target_group)
        .ok_or_else(|| format!("Unsupported drawer resize target {target_group}"))?;
    callback_dispatch::dispatch_layout_command(
        runtime,
        LayoutCommand::SetDrawerRegionExtent { slot, extent },
    )
}

fn group_slot(target_group: &str) -> Option<ActivityDrawerSlot> {
    match target_group {
        "left" => Some(ActivityDrawerSlot::LeftTop),
        "right" => Some(ActivityDrawerSlot::RightTop),
        "bottom" => Some(ActivityDrawerSlot::Bottom),
        _ => None,
    }
}
