use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame,
};
use crate::{ShellRegionId, WorkbenchChromeMetrics, WorkbenchShellGeometry, WorkbenchViewModel};

use super::build_surface::build_surface;
use super::workbench_drawer_header_pointer_layout::WorkbenchDrawerHeaderPointerLayout;

pub(crate) fn build_workbench_drawer_header_pointer_layout(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> WorkbenchDrawerHeaderPointerLayout {
    let mut surfaces = Vec::new();

    if let Some(surface) = build_surface_for_region(
        "left",
        ShellRegionId::Left,
        resolve_root_left_region_frame(geometry, shared_root_frames),
        model,
        &[
            crate::ActivityDrawerSlot::LeftTop,
            crate::ActivityDrawerSlot::LeftBottom,
        ],
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
            crate::ActivityDrawerSlot::RightTop,
            crate::ActivityDrawerSlot::RightBottom,
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
            crate::ActivityDrawerSlot::BottomLeft,
            crate::ActivityDrawerSlot::BottomRight,
        ],
        metrics,
        false,
        shared_root_frames,
    ) {
        surfaces.push(surface);
    }

    WorkbenchDrawerHeaderPointerLayout { surfaces }
}

fn build_surface_for_region(
    key: &str,
    region: ShellRegionId,
    region_frame: zircon_runtime::ui::layout::UiFrame,
    model: &WorkbenchViewModel,
    slots: &[crate::ActivityDrawerSlot],
    metrics: &WorkbenchChromeMetrics,
    side_with_rail: bool,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<super::workbench_drawer_header_pointer_surface::WorkbenchDrawerHeaderPointerSurface> {
    let mut surface = build_surface(key, region_frame, model, slots, metrics, side_with_rail)?;
    if let Some(header_frame) = shared_root_frames
        .and_then(|frames| frames.drawer_header_frame(region))
        .filter(|frame| frame.width > f32::EPSILON && frame.height > f32::EPSILON)
    {
        surface.strip_frame = header_frame;
    }
    Some(surface)
}
