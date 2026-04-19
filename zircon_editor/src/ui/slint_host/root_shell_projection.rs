use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::{ShellFrame, ShellRegionId, WorkbenchChromeMetrics, WorkbenchShellGeometry};
use zircon_runtime::ui::layout::UiFrame;

pub(crate) fn resolve_root_center_band_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    shared_root_frames
        .and_then(|frames| frames.workbench_body_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
        .or_else(|| visible_frame(root_geometry_center_band_frame(geometry)))
        .unwrap_or_default()
}

pub(crate) fn resolve_root_status_bar_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    shared_root_frames
        .and_then(|frames| frames.status_bar_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
        .or_else(|| visible_frame(root_geometry_status_bar_frame(geometry)))
        .unwrap_or_default()
}

pub(crate) fn resolve_root_document_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    resolve_root_layout_frames(geometry, shared_root_frames).document
}

pub(crate) fn resolve_root_left_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    resolve_root_layout_frames(geometry, shared_root_frames).left
}

pub(crate) fn resolve_root_right_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    resolve_root_layout_frames(geometry, shared_root_frames).right
}

pub(crate) fn resolve_root_bottom_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    resolve_root_layout_frames(geometry, shared_root_frames).bottom
}

pub(crate) fn resolve_root_activity_rail_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    let layout_frames = resolve_root_layout_frames(geometry, shared_root_frames);
    if frame_is_visible(layout_frames.left) {
        return activity_rail_frame_from_region(layout_frames.left, metrics);
    }

    shared_root_frames
        .and_then(|frames| frames.activity_rail_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
        .unwrap_or_default()
}

pub(crate) fn resolve_root_document_tabs_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    let layout_frames = resolve_root_layout_frames(geometry, shared_root_frames);
    if layout_frames.has_visible_drawers && frame_is_visible(layout_frames.document) {
        return ShellFrame::new(
            layout_frames.document.x,
            layout_frames.document.y,
            layout_frames.document.width,
            metrics.document_header_height.max(0.0),
        );
    }

    shared_root_frames
        .and_then(|frames| frames.document_tabs_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
        .or_else(|| {
            visible_frame(layout_frames.document).map(|document| {
                ShellFrame::new(
                    document.x,
                    document.y,
                    document.width,
                    metrics.document_header_height.max(0.0),
                )
            })
        })
        .unwrap_or_default()
}

pub(crate) fn resolve_root_viewport_content_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
    document_pane_shows_viewport_toolbar: bool,
) -> ShellFrame {
    let metrics = WorkbenchChromeMetrics::default();
    let viewport_toolbar_height = if document_pane_shows_viewport_toolbar {
        metrics.viewport_toolbar_height
    } else {
        0.0
    };

    let layout_frames = resolve_root_layout_frames(geometry, shared_root_frames);
    if layout_frames.has_visible_drawers && frame_is_visible(layout_frames.document) {
        return ShellFrame::new(
            layout_frames.document.x,
            layout_frames.document.y + viewport_toolbar_height,
            layout_frames.document.width,
            (layout_frames.document.height - viewport_toolbar_height).max(0.0),
        );
    }

    if let (Some(document), Some(document_tabs)) = (
        visible_frame(layout_frames.document),
        shared_root_frames
            .and_then(|frames| frames.document_tabs_frame)
            .map(shell_frame)
            .filter(|frame| frame_is_visible(*frame)),
    ) {
        let document_tabs_extent = document_tabs.height.max(0.0);
        return ShellFrame::new(
            document.x,
            document.y + document_tabs_extent + viewport_toolbar_height,
            document.width,
            (document.height - document_tabs_extent - viewport_toolbar_height).max(0.0),
        );
    }

    shared_root_frames
        .and_then(|frames| frames.pane_surface_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
        .or_else(|| {
            visible_frame(layout_frames.document).map(|document| {
                ShellFrame::new(
                    document.x,
                    document.y + viewport_toolbar_height,
                    document.width,
                    (document.height - viewport_toolbar_height).max(0.0),
                )
            })
        })
        .unwrap_or_default()
}

fn shell_frame(frame: UiFrame) -> ShellFrame {
    ShellFrame::new(frame.x, frame.y, frame.width, frame.height)
}

#[derive(Clone, Copy, Debug, Default)]
struct RootLayoutFrames {
    left: ShellFrame,
    document: ShellFrame,
    right: ShellFrame,
    bottom: ShellFrame,
    has_visible_drawers: bool,
}

fn resolve_root_layout_frames(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> RootLayoutFrames {
    let left = shared_visible_drawer_shell_frame(shared_root_frames, ShellRegionId::Left)
        .unwrap_or_default();
    let right = shared_visible_drawer_shell_frame(shared_root_frames, ShellRegionId::Right)
        .unwrap_or_default();
    let bottom = shared_visible_drawer_shell_frame(shared_root_frames, ShellRegionId::Bottom)
        .unwrap_or_default();
    let document = shared_document_region_frame(shared_root_frames).unwrap_or_default();
    let left_visible = frame_is_visible(left);
    let right_visible = frame_is_visible(right);
    let bottom_visible = frame_is_visible(bottom);
    let has_visible_drawers = left_visible || right_visible || bottom_visible;

    if has_visible_drawers {
        return RootLayoutFrames {
            left,
            document: derive_document_frame_from_drawer_layout(
                shared_root_shell_frame(shared_root_frames),
                shared_root_body_frame(shared_root_frames),
                left,
                right,
                bottom,
            )
            .or_else(|| visible_frame(document))
            .unwrap_or_else(|| root_geometry_region_frame(geometry, ShellRegionId::Document)),
            right,
            bottom,
            has_visible_drawers: true,
        };
    }

    if let Some(layout_frames) =
        derive_layout_frames_from_geometry_with_shared_root(geometry, shared_root_frames)
    {
        return layout_frames;
    }

    RootLayoutFrames {
        document: visible_frame(document)
            .unwrap_or_else(|| root_geometry_region_frame(geometry, ShellRegionId::Document)),
        ..RootLayoutFrames::default()
    }
}

fn activity_rail_frame_from_region(
    left_region: ShellFrame,
    metrics: &WorkbenchChromeMetrics,
) -> ShellFrame {
    ShellFrame::new(
        left_region.x,
        left_region.y,
        metrics.rail_width.min(left_region.width.max(0.0)),
        left_region.height.max(0.0),
    )
}

fn shared_document_region_frame(
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.document_host_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
}

fn shared_visible_drawer_shell_frame(
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
    region: ShellRegionId,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.drawer_shell_frame(region))
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
}

fn visible_frame(frame: ShellFrame) -> Option<ShellFrame> {
    frame_is_visible(frame).then_some(frame)
}

fn frame_is_visible(frame: ShellFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

fn shared_root_shell_frame(
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.shell_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
}

fn shared_root_body_frame(
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.workbench_body_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
}

fn derive_layout_frames_from_geometry_with_shared_root(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<RootLayoutFrames> {
    let left_width = root_geometry_region_frame(geometry, ShellRegionId::Left)
        .width
        .max(0.0);
    let right_width = root_geometry_region_frame(geometry, ShellRegionId::Right)
        .width
        .max(0.0);
    let bottom_height = root_geometry_region_frame(geometry, ShellRegionId::Bottom)
        .height
        .max(0.0);
    let left_visible = left_width > f32::EPSILON;
    let right_visible = right_width > f32::EPSILON;
    let bottom_visible = bottom_height > f32::EPSILON;

    if !(left_visible || right_visible || bottom_visible) {
        return None;
    }

    let metrics = WorkbenchChromeMetrics::default();
    let separator = metrics.separator_thickness.max(0.0);
    let body_frame = match shared_root_body_frame(shared_root_frames) {
        Some(frame) => frame,
        None => {
            return Some(RootLayoutFrames {
                left: root_geometry_region_frame(geometry, ShellRegionId::Left),
                document: root_geometry_region_frame(geometry, ShellRegionId::Document),
                right: root_geometry_region_frame(geometry, ShellRegionId::Right),
                bottom: root_geometry_region_frame(geometry, ShellRegionId::Bottom),
                has_visible_drawers: true,
            });
        }
    };
    let shell_frame = shared_root_shell_frame(shared_root_frames).unwrap_or_else(|| {
        ShellFrame::new(
            body_frame.x,
            body_frame.y,
            body_frame.width,
            body_frame.height,
        )
    });
    let center_height = (body_frame.height
        - if bottom_visible {
            bottom_height + separator
        } else {
            0.0
        })
    .max(0.0);
    let left = left_visible
        .then_some(ShellFrame::new(
            shell_frame.x,
            body_frame.y,
            left_width,
            center_height,
        ))
        .unwrap_or_default();
    let right = right_visible
        .then_some(ShellFrame::new(
            shell_frame.x + shell_frame.width - right_width,
            body_frame.y,
            right_width,
            center_height,
        ))
        .unwrap_or_default();
    let document_x = shell_frame.x
        + if left_visible {
            left_width + separator
        } else {
            0.0
        };
    let document_width = (shell_frame.width
        - if left_visible {
            left_width + separator
        } else {
            0.0
        }
        - if right_visible {
            right_width + separator
        } else {
            0.0
        })
    .max(0.0);
    let document = ShellFrame::new(document_x, body_frame.y, document_width, center_height);
    let bottom = bottom_visible
        .then_some(ShellFrame::new(
            shell_frame.x,
            body_frame.y + center_height + separator,
            shell_frame.width,
            bottom_height,
        ))
        .unwrap_or_default();

    Some(RootLayoutFrames {
        left,
        document,
        right,
        bottom,
        has_visible_drawers: true,
    })
}

fn derive_document_frame_from_drawer_layout(
    shell_frame: Option<ShellFrame>,
    body_frame: Option<ShellFrame>,
    left: ShellFrame,
    right: ShellFrame,
    bottom: ShellFrame,
) -> Option<ShellFrame> {
    let body_frame = body_frame?;
    let shell_frame = shell_frame.unwrap_or_else(|| {
        ShellFrame::new(
            body_frame.x,
            body_frame.y,
            body_frame.width,
            body_frame.height,
        )
    });
    let metrics = WorkbenchChromeMetrics::default();
    let separator = metrics.separator_thickness.max(0.0);
    let left_visible = frame_is_visible(left);
    let right_visible = frame_is_visible(right);
    let bottom_visible = frame_is_visible(bottom);
    let document_x = shell_frame.x
        + if left_visible {
            left.width + separator
        } else {
            0.0
        };
    let document_width = (shell_frame.width
        - if left_visible {
            left.width + separator
        } else {
            0.0
        }
        - if right_visible {
            right.width + separator
        } else {
            0.0
        })
    .max(0.0);
    let document_height = (body_frame.height
        - if bottom_visible {
            bottom.height + separator
        } else {
            0.0
        })
    .max(0.0);
    Some(ShellFrame::new(
        document_x,
        body_frame.y,
        document_width,
        document_height,
    ))
}

fn root_geometry_region_frame(
    geometry: &WorkbenchShellGeometry,
    region: ShellRegionId,
) -> ShellFrame {
    let WorkbenchShellGeometry { region_frames, .. } = geometry;
    region_frames.get(&region).copied().unwrap_or_default()
}

fn root_geometry_center_band_frame(geometry: &WorkbenchShellGeometry) -> ShellFrame {
    let WorkbenchShellGeometry {
        center_band_frame, ..
    } = geometry;
    *center_band_frame
}

fn root_geometry_status_bar_frame(geometry: &WorkbenchShellGeometry) -> ShellFrame {
    let WorkbenchShellGeometry {
        status_bar_frame, ..
    } = geometry;
    *status_bar_frame
}
