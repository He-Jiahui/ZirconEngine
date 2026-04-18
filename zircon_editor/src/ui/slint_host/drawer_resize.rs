use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::callback_dispatch;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;
use crate::{ActivityDrawerSlot, LayoutCommand, ShellRegionId};

#[cfg(test)]
use crate::{ShellSizePx, WorkbenchShellGeometry};
#[cfg(test)]
use zircon_ui::UiPoint;

#[cfg(test)]
use super::shell_pointer::WorkbenchShellPointerBridge;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchResizeTargetGroup {
    Left,
    Right,
    Bottom,
}

impl WorkbenchResizeTargetGroup {
    pub(crate) const fn region(self) -> ShellRegionId {
        match self {
            Self::Left => ShellRegionId::Left,
            Self::Right => ShellRegionId::Right,
            Self::Bottom => ShellRegionId::Bottom,
        }
    }
}

#[cfg(test)]
pub(crate) fn resolve_workbench_resize_target_group(
    root_size: ShellSizePx,
    geometry: &WorkbenchShellGeometry,
    point: UiPoint,
) -> Option<WorkbenchResizeTargetGroup> {
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout(root_size, geometry, true);
    bridge.resize_target_at(point)
}

#[cfg(test)]
use crate::EditorManager;

#[cfg(test)]
pub(crate) fn apply_resize_to_group(
    editor_manager: &EditorManager,
    target_group: &str,
    extent: f32,
) -> Result<bool, String> {
    let slots = group_slots(target_group)
        .ok_or_else(|| format!("Unsupported drawer resize target {target_group}"))?;

    let mut changed = false;
    for slot in slots {
        changed |= editor_manager
            .apply_layout_command(LayoutCommand::SetDrawerExtent {
                slot: *slot,
                extent,
            })
            .map_err(|error| error.to_string())?;
    }

    Ok(changed)
}

pub(crate) fn dispatch_resize_to_group(
    runtime: &EditorEventRuntime,
    target_group: &str,
    extent: f32,
) -> Result<SlintDispatchEffects, String> {
    let slots = group_slots(target_group)
        .ok_or_else(|| format!("Unsupported drawer resize target {target_group}"))?;

    let mut combined = SlintDispatchEffects::default();
    for slot in slots {
        let effects = callback_dispatch::dispatch_layout_command(
            runtime,
            LayoutCommand::SetDrawerExtent {
                slot: *slot,
                extent,
            },
        )?;
        combined.presentation_dirty |= effects.presentation_dirty;
        combined.layout_dirty |= effects.layout_dirty;
        combined.render_dirty |= effects.render_dirty;
        combined.sync_asset_workspace |= effects.sync_asset_workspace;
        combined.refresh_asset_details |= effects.refresh_asset_details;
        combined.refresh_visible_asset_previews |= effects.refresh_visible_asset_previews;
        combined.reset_active_layout_preset |= effects.reset_active_layout_preset;
    }

    Ok(combined)
}

fn group_slots(target_group: &str) -> Option<&'static [ActivityDrawerSlot]> {
    match target_group {
        "left" => Some(&[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom]),
        "right" => Some(&[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ]),
        "bottom" => Some(&[
            ActivityDrawerSlot::BottomLeft,
            ActivityDrawerSlot::BottomRight,
        ]),
        _ => None,
    }
}
