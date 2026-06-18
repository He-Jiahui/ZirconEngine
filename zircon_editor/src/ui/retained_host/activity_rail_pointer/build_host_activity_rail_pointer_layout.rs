use zircon_runtime_interface::ui::layout::UiFrame;

#[cfg(test)]
use crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::workbench::autolayout::{ShellRegionId, WorkbenchChromeMetrics};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::collect_tabs::collect_tabs;
use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;

#[cfg(test)]
pub(crate) fn build_host_activity_rail_pointer_layout(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> HostActivityRailPointerLayout {
    let root_activity_rail_frame = shared_root_frames
        .and_then(|frames| frames.activity_rail_frame)
        .filter(ui_frame_is_visible);
    build_host_activity_rail_pointer_layout_with_workbench_layout_frames(
        model,
        metrics,
        BuiltinWorkbenchWindowLayoutFrames {
            activity_rail_frame: root_activity_rail_frame,
            ..BuiltinWorkbenchWindowLayoutFrames::default()
        },
    )
}

pub(crate) fn build_host_activity_rail_pointer_layout_with_workbench_layout_frames(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> HostActivityRailPointerLayout {
    let left_tabs = collect_tabs(
        model,
        &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
    );
    let right_tabs = collect_tabs(
        model,
        &[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ],
    );
    let right_region = workbench_layout_frames
        .drawer_shell_frame(ShellRegionId::Right)
        .filter(ui_frame_is_visible)
        .unwrap_or_default();
    let rail_width = metrics.rail_width.max(0.0);
    let resolved_left_strip_frame = workbench_layout_frames
        .activity_rail_frame
        .filter(ui_frame_is_visible)
        .unwrap_or_default();

    let left_strip_frame = if resolved_left_strip_frame.width > 0.0
        && resolved_left_strip_frame.height > 0.0
        && !left_tabs.is_empty()
    {
        UiFrame::new(
            resolved_left_strip_frame.x,
            resolved_left_strip_frame.y,
            resolved_left_strip_frame.width.max(0.0),
            resolved_left_strip_frame.height.max(0.0),
        )
    } else {
        UiFrame::default()
    };
    let right_strip_frame = if right_region.width > 0.0 && !right_tabs.is_empty() {
        UiFrame::new(
            (right_region.x + right_region.width - rail_width).max(right_region.x),
            right_region.y,
            rail_width.min(right_region.width.max(0.0)),
            right_region.height.max(0.0),
        )
    } else {
        UiFrame::default()
    };

    HostActivityRailPointerLayout {
        left_strip_frame,
        left_tabs,
        right_strip_frame,
        right_tabs,
    }
}

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}
