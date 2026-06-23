#[cfg(any(test, feature = "integration-contracts"))]
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::layout::UiPoint;

#[cfg(test)]
use crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames;
#[cfg(any(test, feature = "integration-contracts"))]
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::shell_pointer::HostShellPointerBridge;
use crate::ui::workbench::autolayout::ShellSizePx;
#[cfg(feature = "integration-contracts")]
use crate::ui::workbench::autolayout::{
    ShellFrame, ShellRegionId, WorkbenchChromeMetrics, WorkbenchShellGeometry,
};

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

    #[cfg(test)]
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

    #[cfg(any(test, feature = "integration-contracts"))]
    pub(crate) fn update_layout_with_workbench_layout_frames(
        &mut self,
        root_size: ShellSizePx,
        drawers_visible: bool,
        componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    ) {
        self.shell_pointer
            .update_layout_with_workbench_layout_frames(
                root_size,
                drawers_visible,
                &[],
                componentized_workbench_layout_frames,
                None,
            );
    }

    pub(crate) fn resolve(&mut self, point: UiPoint) -> Option<HostDragTargetGroup> {
        self.shell_pointer.drag_target_at(point)
    }
}

#[cfg(test)]
pub fn resolve_host_drag_target_group(
    root_size: ShellSizePx,
    drawers_visible: bool,
    point: UiPoint,
) -> Option<HostDragTargetGroup> {
    resolve_host_drag_target_group_with_root_frames(root_size, drawers_visible, point, None)
}

#[cfg(test)]
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

#[cfg(any(test, feature = "integration-contracts"))]
pub(crate) fn resolve_host_drag_target_group_with_workbench_layout_frames(
    root_size: ShellSizePx,
    drawers_visible: bool,
    point: UiPoint,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<HostDragTargetGroup> {
    let mut bridge = HostDragTargetBridge::new();
    bridge.update_layout_with_workbench_layout_frames(
        root_size,
        drawers_visible,
        componentized_workbench_layout_frames,
    );
    bridge.resolve(point)
}

#[cfg(feature = "integration-contracts")]
pub fn resolve_host_drag_target_group_with_workbench_shell_geometry(
    root_size: ShellSizePx,
    drawers_visible: bool,
    point: UiPoint,
    geometry: &WorkbenchShellGeometry,
    drawer_regions: &[ShellRegionId],
) -> Option<HostDragTargetGroup> {
    resolve_host_drag_target_group_with_workbench_layout_frames(
        root_size,
        drawers_visible,
        point,
        workbench_layout_frames_from_geometry_with_drawers(geometry, drawer_regions),
    )
}

#[cfg(feature = "integration-contracts")]
fn workbench_layout_frames_from_geometry_with_drawers(
    geometry: &WorkbenchShellGeometry,
    drawer_regions: &[ShellRegionId],
) -> BuiltinWorkbenchWindowLayoutFrames {
    BuiltinWorkbenchWindowLayoutFrames {
        center_band_frame: Some(ui_frame(geometry.center_band_frame)),
        left_region_frame: drawer_regions
            .contains(&ShellRegionId::Left)
            .then(|| ui_frame(geometry.region_frame(ShellRegionId::Left))),
        right_region_frame: drawer_regions
            .contains(&ShellRegionId::Right)
            .then(|| ui_frame(geometry.region_frame(ShellRegionId::Right))),
        bottom_region_frame: drawer_regions
            .contains(&ShellRegionId::Bottom)
            .then(|| ui_frame(geometry.region_frame(ShellRegionId::Bottom))),
        document_region_frame: Some(ui_frame(geometry.region_frame(ShellRegionId::Document))),
        status_bar_frame: Some(ui_frame(geometry.status_bar_frame)),
        document_tabs_frame: Some(ui_frame(document_tabs_frame_from_geometry(geometry))),
        ..BuiltinWorkbenchWindowLayoutFrames::default()
    }
}

#[cfg(feature = "integration-contracts")]
fn document_tabs_frame_from_geometry(geometry: &WorkbenchShellGeometry) -> ShellFrame {
    let metrics = WorkbenchChromeMetrics::default();
    let document = geometry.region_frame(ShellRegionId::Document);
    ShellFrame::new(
        document.x,
        document.y,
        document.width,
        metrics.document_header_height.max(0.0),
    )
}

#[cfg(feature = "integration-contracts")]
fn ui_frame(frame: ShellFrame) -> UiFrame {
    UiFrame::new(frame.x, frame.y, frame.width, frame.height)
}
