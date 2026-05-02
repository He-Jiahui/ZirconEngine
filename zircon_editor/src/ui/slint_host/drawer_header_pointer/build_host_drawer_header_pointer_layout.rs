use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame,
};
use crate::ui::workbench::autolayout::{
    ShellRegionId, WorkbenchChromeMetrics, WorkbenchShellGeometry,
};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::UiFrame;

use super::build_surface::build_surface;
use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;

pub(crate) fn build_host_drawer_header_pointer_layout(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> HostDrawerHeaderPointerLayout {
    let mut surfaces = Vec::new();

    if let Some(surface) = build_surface_for_region(
        "left",
        ShellRegionId::Left,
        resolve_root_left_region_frame(geometry, shared_root_frames),
        model,
        &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        metrics,
        true,
        shared_root_frames,
    ) {
        surfaces.push(surface);
    }
    if let Some(surface) = build_surface_for_region(
        "right",
        ShellRegionId::Right,
        resolve_root_right_region_frame(geometry, shared_root_frames),
        model,
        &[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ],
        metrics,
        false,
        shared_root_frames,
    ) {
        surfaces.push(surface);
    }
    if let Some(surface) = build_surface_for_region(
        "bottom",
        ShellRegionId::Bottom,
        resolve_root_bottom_region_frame(geometry, shared_root_frames),
        model,
        &[
            ActivityDrawerSlot::BottomLeft,
            ActivityDrawerSlot::BottomRight,
        ],
        metrics,
        false,
        shared_root_frames,
    ) {
        surfaces.push(surface);
    }

    HostDrawerHeaderPointerLayout { surfaces }
}

fn build_surface_for_region(
    key: &str,
    region: ShellRegionId,
    region_frame: UiFrame,
    model: &WorkbenchViewModel,
    slots: &[crate::ui::workbench::layout::ActivityDrawerSlot],
    metrics: &WorkbenchChromeMetrics,
    side_with_rail: bool,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<super::host_drawer_header_pointer_surface::HostDrawerHeaderPointerSurface> {
    let mut surface = build_surface(key, region_frame, model, slots, metrics, side_with_rail)?;
    if let Some(header_frame) = shared_root_frames
        .and_then(|frames| frames.drawer_header_frame(region))
        .filter(|frame| frame.width > f32::EPSILON && frame.height > f32::EPSILON)
    {
        surface.strip_frame = header_frame;
    }
    Some(surface)
}
