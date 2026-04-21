use zircon_runtime::ui::layout::UiFrame;

use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_activity_rail_frame, resolve_root_right_region_frame,
};
use crate::ui::workbench::autolayout::{WorkbenchChromeMetrics, WorkbenchShellGeometry};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::collect_tabs::collect_tabs;
use super::workbench_activity_rail_pointer_layout::WorkbenchActivityRailPointerLayout;

pub(crate) fn build_workbench_activity_rail_pointer_layout(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> WorkbenchActivityRailPointerLayout {
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
    let right_region = resolve_root_right_region_frame(geometry, shared_root_frames);
    let rail_width = metrics.rail_width.max(0.0);
    let resolved_left_strip_frame =
        resolve_root_activity_rail_frame(geometry, metrics, shared_root_frames);

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

    WorkbenchActivityRailPointerLayout {
        left_strip_frame,
        left_tabs,
        right_strip_frame,
        right_tabs,
    }
}
