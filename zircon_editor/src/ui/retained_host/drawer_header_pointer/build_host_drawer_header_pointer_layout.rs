use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::workbench::autolayout::{ShellRegionId, WorkbenchChromeMetrics};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::UiFrame;

use super::build_surface::build_surface;
use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;

pub(crate) fn build_host_drawer_header_pointer_layout_with_workbench_layout_frames(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> HostDrawerHeaderPointerLayout {
    build_host_drawer_header_pointer_layout_with_optional_workbench_frames(
        model,
        metrics,
        Some(componentized_workbench_layout_frames),
    )
}

fn build_host_drawer_header_pointer_layout_with_optional_workbench_frames(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    componentized_workbench_layout_frames: Option<BuiltinWorkbenchWindowLayoutFrames>,
) -> HostDrawerHeaderPointerLayout {
    let mut surfaces = Vec::new();

    if let Some(surface) = build_surface_for_region(
        "left",
        drawer_region_frame(ShellRegionId::Left, componentized_workbench_layout_frames),
        model,
        &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        metrics,
        true,
        componentized_workbench_layout_frames
            .and_then(|frames| frames.drawer_header_frame(ShellRegionId::Left)),
    ) {
        surfaces.push(surface);
    }
    if let Some(surface) = build_surface_for_region(
        "right",
        drawer_region_frame(ShellRegionId::Right, componentized_workbench_layout_frames),
        model,
        &[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ],
        metrics,
        false,
        componentized_workbench_layout_frames
            .and_then(|frames| frames.drawer_header_frame(ShellRegionId::Right)),
    ) {
        surfaces.push(surface);
    }
    if let Some(surface) = build_surface_for_region(
        "bottom",
        drawer_region_frame(ShellRegionId::Bottom, componentized_workbench_layout_frames),
        model,
        &[ActivityDrawerSlot::Bottom],
        metrics,
        false,
        componentized_workbench_layout_frames
            .and_then(|frames| frames.drawer_header_frame(ShellRegionId::Bottom)),
    ) {
        surfaces.push(surface);
    }

    HostDrawerHeaderPointerLayout { surfaces }
}

fn drawer_region_frame(
    region: ShellRegionId,
    componentized_workbench_layout_frames: Option<BuiltinWorkbenchWindowLayoutFrames>,
) -> UiFrame {
    componentized_workbench_layout_frames
        .and_then(|frames| frames.drawer_shell_frame(region))
        .filter(ui_frame_is_visible)
        .unwrap_or_default()
}

fn build_surface_for_region(
    key: &str,
    region_frame: UiFrame,
    model: &WorkbenchViewModel,
    slots: &[crate::ui::workbench::layout::ActivityDrawerSlot],
    metrics: &WorkbenchChromeMetrics,
    side_with_rail: bool,
    componentized_drawer_header_frame: Option<UiFrame>,
) -> Option<super::host_drawer_header_pointer_surface::HostDrawerHeaderPointerSurface> {
    let mut surface = build_surface(key, region_frame, model, slots, metrics, side_with_rail)?;
    if let Some(header_frame) = componentized_drawer_header_frame.filter(ui_frame_is_visible) {
        surface.strip_frame = header_frame;
    }
    Some(surface)
}

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}
