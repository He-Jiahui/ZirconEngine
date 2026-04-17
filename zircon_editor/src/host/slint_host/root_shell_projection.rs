use crate::host::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::{ShellFrame, ShellRegionId, WorkbenchChromeMetrics, WorkbenchShellGeometry};
use zircon_ui::UiFrame;

pub(crate) fn resolve_root_center_band_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    shared_root_frames
        .and_then(|frames| frames.workbench_body_frame)
        .map(shell_frame)
        .unwrap_or(geometry.center_band_frame)
}

pub(crate) fn resolve_root_status_bar_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    shared_root_frames
        .and_then(|frames| frames.status_bar_frame)
        .map(shell_frame)
        .unwrap_or(geometry.status_bar_frame)
}

pub(crate) fn resolve_root_document_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    if has_visible_drawer_regions(geometry) {
        return resolve_root_visible_drawer_document_region_frame(geometry, shared_root_frames);
    }

    shared_root_frames
        .and_then(|frames| frames.document_host_frame)
        .map(shell_frame)
        .unwrap_or_else(|| geometry.region_frame(ShellRegionId::Document))
}

pub(crate) fn resolve_root_left_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    resolve_root_visible_drawer_region_frame(geometry, shared_root_frames, ShellRegionId::Left)
}

pub(crate) fn resolve_root_right_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    resolve_root_visible_drawer_region_frame(geometry, shared_root_frames, ShellRegionId::Right)
}

pub(crate) fn resolve_root_bottom_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    resolve_root_visible_drawer_region_frame(geometry, shared_root_frames, ShellRegionId::Bottom)
}

pub(crate) fn resolve_root_activity_rail_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    if has_visible_drawer_regions(geometry) {
        return activity_rail_frame_from_region(
            resolve_root_left_region_frame(geometry, shared_root_frames),
            metrics,
        );
    }

    shared_root_frames
        .and_then(|frames| frames.activity_rail_frame)
        .map(shell_frame)
        .unwrap_or_else(|| legacy_root_activity_rail_frame(geometry, metrics))
}

pub(crate) fn resolve_root_document_tabs_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    if has_visible_drawer_regions(geometry) {
        let document = resolve_root_document_region_frame(geometry, shared_root_frames);
        return ShellFrame::new(
            document.x,
            document.y,
            document.width,
            metrics.document_header_height.max(0.0),
        );
    }

    shared_root_frames
        .and_then(|frames| frames.document_tabs_frame)
        .map(shell_frame)
        .unwrap_or_else(|| {
            let document = geometry.region_frame(ShellRegionId::Document);
            ShellFrame::new(
                document.x,
                document.y,
                document.width,
                metrics.document_header_height.max(0.0),
            )
        })
}

pub(crate) fn resolve_root_viewport_content_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
    document_pane_shows_viewport_toolbar: bool,
) -> ShellFrame {
    let viewport_toolbar_height = if document_pane_shows_viewport_toolbar {
        WorkbenchChromeMetrics::default().viewport_toolbar_height
    } else {
        0.0
    };

    if has_visible_drawer_regions(geometry) {
        let document = resolve_root_document_region_frame(geometry, shared_root_frames);
        return ShellFrame::new(
            document.x,
            document.y + viewport_toolbar_height,
            document.width,
            (document.height - viewport_toolbar_height).max(0.0),
        );
    }

    shared_root_frames
        .and_then(|frames| frames.pane_surface_frame)
        .map(|frame| {
            ShellFrame::new(
                frame.x,
                frame.y + viewport_toolbar_height,
                frame.width,
                (frame.height - viewport_toolbar_height).max(0.0),
            )
        })
        .unwrap_or(geometry.viewport_content_frame)
}

pub(crate) fn has_visible_drawer_regions(geometry: &WorkbenchShellGeometry) -> bool {
    [
        ShellRegionId::Left,
        ShellRegionId::Right,
        ShellRegionId::Bottom,
    ]
    .into_iter()
    .any(|region| {
        let frame = geometry.region_frame(region);
        frame.width > f32::EPSILON && frame.height > f32::EPSILON
    })
}

fn shell_frame(frame: UiFrame) -> ShellFrame {
    ShellFrame::new(frame.x, frame.y, frame.width, frame.height)
}

fn legacy_root_activity_rail_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
) -> ShellFrame {
    activity_rail_frame_from_region(geometry.region_frame(ShellRegionId::Left), metrics)
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

fn resolve_root_visible_drawer_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
    region: ShellRegionId,
) -> ShellFrame {
    let legacy_frame = geometry.region_frame(region);
    if legacy_frame.width <= f32::EPSILON || legacy_frame.height <= f32::EPSILON {
        return legacy_frame;
    }

    if let Some(frame) = shared_root_frames
        .and_then(|frames| frames.drawer_shell_frame(region))
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
    {
        return frame;
    }

    shared_root_body_frame(shared_root_frames)
        .map(|body| {
            let separator = WorkbenchChromeMetrics::default()
                .separator_thickness
                .max(0.0);
            let bottom = geometry.region_frame(ShellRegionId::Bottom);
            let bottom_separator = if frame_is_visible(bottom) {
                separator
            } else {
                0.0
            };
            let center_height = (body.height - bottom.height.max(0.0) - bottom_separator).max(0.0);

            match region {
                ShellRegionId::Left => {
                    ShellFrame::new(body.x, body.y, legacy_frame.width, center_height)
                }
                ShellRegionId::Right => ShellFrame::new(
                    body.x + body.width - legacy_frame.width,
                    body.y,
                    legacy_frame.width,
                    center_height,
                ),
                ShellRegionId::Bottom => ShellFrame::new(
                    body.x,
                    body.y + body.height - legacy_frame.height,
                    body.width,
                    legacy_frame.height,
                ),
                ShellRegionId::Document => legacy_frame,
            }
        })
        .unwrap_or(legacy_frame)
}

fn resolve_root_visible_drawer_document_region_frame(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> ShellFrame {
    let legacy_frame = geometry.region_frame(ShellRegionId::Document);
    let separator = WorkbenchChromeMetrics::default()
        .separator_thickness
        .max(0.0);

    shared_root_body_frame(shared_root_frames)
        .map(|body| {
            let left = shared_root_frames
                .and_then(|frames| frames.drawer_shell_frame(ShellRegionId::Left))
                .map(shell_frame)
                .filter(|frame| frame_is_visible(*frame))
                .unwrap_or_else(|| geometry.region_frame(ShellRegionId::Left));
            let right = shared_root_frames
                .and_then(|frames| frames.drawer_shell_frame(ShellRegionId::Right))
                .map(shell_frame)
                .filter(|frame| frame_is_visible(*frame))
                .unwrap_or_else(|| geometry.region_frame(ShellRegionId::Right));
            let bottom = shared_root_frames
                .and_then(|frames| frames.drawer_shell_frame(ShellRegionId::Bottom))
                .map(shell_frame)
                .filter(|frame| frame_is_visible(*frame))
                .unwrap_or_else(|| geometry.region_frame(ShellRegionId::Bottom));
            let left_visible = frame_is_visible(left);
            let right_visible = frame_is_visible(right);
            let bottom_visible = frame_is_visible(bottom);
            let left_width = left.width.max(0.0);
            let right_width = right.width.max(0.0);
            let bottom_height = bottom.height.max(0.0);
            let left_separator = if left_visible { separator } else { 0.0 };
            let right_separator = if right_visible { separator } else { 0.0 };
            let bottom_separator = if bottom_visible { separator } else { 0.0 };
            ShellFrame::new(
                body.x + left_width + left_separator,
                body.y,
                (body.width - left_width - right_width - left_separator - right_separator).max(0.0),
                (body.height - bottom_height - bottom_separator).max(0.0),
            )
        })
        .unwrap_or(legacy_frame)
}

fn shared_root_body_frame(
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.workbench_body_frame)
        .map(shell_frame)
}

fn frame_is_visible(frame: ShellFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}
