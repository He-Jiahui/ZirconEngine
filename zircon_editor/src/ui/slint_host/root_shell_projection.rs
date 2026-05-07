use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::workbench::autolayout::compact_bottom_height_limit;
use crate::ui::workbench::autolayout::compact_side_width_limit;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::autolayout::ShellRegionId;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(crate) fn resolve_root_center_band_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    shared_root_body_frame(shared_root_frames).unwrap_or_default()
}

pub(crate) fn resolve_root_status_bar_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    shared_root_frames
        .and_then(|frames| frames.status_bar_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
        .unwrap_or_default()
}

pub(crate) fn resolve_root_document_region_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    resolve_root_layout_frames(shared_root_frames).document
}

pub(crate) fn resolve_root_left_region_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    resolve_root_layout_frames(shared_root_frames).left
}

pub(crate) fn resolve_root_right_region_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    resolve_root_layout_frames(shared_root_frames).right
}

pub(crate) fn resolve_root_bottom_region_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    resolve_root_layout_frames(shared_root_frames).bottom
}

pub(crate) fn resolve_root_left_splitter_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    let layout_frames = resolve_root_layout_frames(shared_root_frames);
    if layout_frames.has_visible_drawers && frame_is_visible(layout_frames.left) {
        return vertical_splitter_frame_at(
            layout_frames.left.right(),
            layout_frames.left.y,
            layout_frames.left.height,
        );
    }

    ShellFrame::default()
}

pub(crate) fn resolve_root_right_splitter_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    let layout_frames = resolve_root_layout_frames(shared_root_frames);
    if layout_frames.has_visible_drawers && frame_is_visible(layout_frames.right) {
        let metrics = WorkbenchChromeMetrics::default();
        return vertical_splitter_frame_at(
            layout_frames.right.x - metrics.separator_thickness.max(0.0),
            layout_frames.right.y,
            layout_frames.right.height,
        );
    }

    ShellFrame::default()
}

pub(crate) fn resolve_root_bottom_splitter_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    let layout_frames = resolve_root_layout_frames(shared_root_frames);
    if layout_frames.has_visible_drawers && frame_is_visible(layout_frames.bottom) {
        let metrics = WorkbenchChromeMetrics::default();
        return ShellFrame::new(
            layout_frames.bottom.x,
            layout_frames.bottom.y
                - metrics.separator_thickness.max(0.0)
                - metrics.splitter_hit_size.max(0.0) * 0.5,
            layout_frames.bottom.width,
            metrics.splitter_hit_size.max(0.0),
        );
    }

    ShellFrame::default()
}

pub(crate) fn resolve_root_activity_rail_frame(
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    let layout_frames = resolve_root_layout_frames(shared_root_frames);
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
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> ShellFrame {
    let layout_frames = resolve_root_layout_frames(shared_root_frames);
    if layout_frames.has_visible_drawers && frame_is_visible(layout_frames.document) {
        return document_tabs_frame_from_document(layout_frames.document, metrics);
    }

    shared_root_frames
        .and_then(|frames| frames.document_tabs_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
        .or_else(|| {
            visible_frame(layout_frames.document)
                .map(|document| document_tabs_frame_from_document(document, metrics))
        })
        .unwrap_or_default()
}

pub(crate) fn resolve_root_viewport_content_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    document_pane_shows_viewport_toolbar: bool,
) -> ShellFrame {
    let metrics = WorkbenchChromeMetrics::default();
    let viewport_toolbar_height = if document_pane_shows_viewport_toolbar {
        metrics.viewport_toolbar_height
    } else {
        0.0
    };

    let layout_frames = resolve_root_layout_frames(shared_root_frames);
    if layout_frames.has_visible_drawers && frame_is_visible(layout_frames.document) {
        return document_viewport_content_frame(
            layout_frames.document,
            &metrics,
            viewport_toolbar_height,
        );
    }

    if let Some(pane_surface) = shared_pane_surface_frame(shared_root_frames) {
        return ShellFrame::new(
            pane_surface.x,
            pane_surface.y + viewport_toolbar_height,
            pane_surface.width,
            (pane_surface.height - viewport_toolbar_height).max(0.0),
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
            document.y
                + document_tabs_extent
                + metrics.separator_thickness.max(0.0)
                + viewport_toolbar_height,
            document.width,
            (document.height
                - document_tabs_extent
                - metrics.separator_thickness.max(0.0)
                - viewport_toolbar_height)
                .max(0.0),
        );
    }

    visible_frame(layout_frames.document)
        .map(|document| {
            document_viewport_content_frame(document, &metrics, viewport_toolbar_height)
        })
        .unwrap_or_default()
}

fn document_tabs_frame_from_document(
    document: ShellFrame,
    metrics: &WorkbenchChromeMetrics,
) -> ShellFrame {
    ShellFrame::new(
        document.x,
        document.y,
        document.width,
        metrics.document_header_height.max(0.0),
    )
}

fn document_viewport_content_frame(
    document: ShellFrame,
    metrics: &WorkbenchChromeMetrics,
    viewport_toolbar_height: f32,
) -> ShellFrame {
    let document_chrome_height =
        metrics.document_header_height.max(0.0) + metrics.separator_thickness.max(0.0);
    ShellFrame::new(
        document.x,
        document.y + document_chrome_height + viewport_toolbar_height,
        document.width,
        (document.height - document_chrome_height - viewport_toolbar_height).max(0.0),
    )
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
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> RootLayoutFrames {
    let shell = shared_root_shell_frame(shared_root_frames);
    let left = compact_shared_side_frame(
        shell,
        shared_visible_drawer_shell_frame(shared_root_frames, ShellRegionId::Left)
            .unwrap_or_default(),
        ShellRegionId::Left,
    );
    let right = compact_shared_side_frame(
        shell,
        shared_visible_drawer_shell_frame(shared_root_frames, ShellRegionId::Right)
            .unwrap_or_default(),
        ShellRegionId::Right,
    );
    let bottom = compact_shared_bottom_frame(
        shared_root_body_frame(shared_root_frames),
        shared_visible_drawer_shell_frame(shared_root_frames, ShellRegionId::Bottom)
            .unwrap_or_default(),
    );
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
            .unwrap_or_default(),
            right,
            bottom,
            has_visible_drawers: true,
        };
    }

    RootLayoutFrames {
        document: visible_frame(document).unwrap_or_default(),
        ..RootLayoutFrames::default()
    }
}

fn compact_shared_side_frame(
    shell: Option<ShellFrame>,
    frame: ShellFrame,
    region: ShellRegionId,
) -> ShellFrame {
    if !frame_is_visible(frame) {
        return frame;
    }

    let Some(shell) = shell.filter(|frame| frame_is_visible(*frame)) else {
        return frame;
    };
    let Some(limit) = compact_side_width_limit(region, shell.width.max(0.0)) else {
        return frame;
    };

    let width = frame.width.min(limit);
    match region {
        ShellRegionId::Left => ShellFrame::new(frame.x, frame.y, width, frame.height),
        ShellRegionId::Right => {
            ShellFrame::new(shell.x + shell.width - width, frame.y, width, frame.height)
        }
        ShellRegionId::Bottom | ShellRegionId::Document => frame,
    }
}

fn compact_shared_bottom_frame(body: Option<ShellFrame>, bottom: ShellFrame) -> ShellFrame {
    if !frame_is_visible(bottom) {
        return bottom;
    }

    let Some(body) = body.filter(|frame| frame_is_visible(*frame)) else {
        return bottom;
    };
    let metrics = WorkbenchChromeMetrics::default();
    let available_height = (body.height - metrics.separator_thickness.max(0.0)).max(0.0);
    let Some(limit) = compact_bottom_height_limit(available_height) else {
        return bottom;
    };

    let height = bottom.height.min(limit);
    ShellFrame::new(
        bottom.x,
        body.y + body.height - height,
        bottom.width,
        height,
    )
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

fn vertical_splitter_frame_at(edge_x: f32, y: f32, height: f32) -> ShellFrame {
    let metrics = WorkbenchChromeMetrics::default();
    ShellFrame::new(
        edge_x - metrics.splitter_hit_size.max(0.0) * 0.5,
        y,
        metrics.splitter_hit_size.max(0.0),
        height.max(0.0),
    )
}

fn shared_document_region_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.document_host_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
}

fn shared_visible_drawer_shell_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    region: ShellRegionId,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.drawer_shell_frame(region))
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
}

fn shared_pane_surface_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.pane_surface_frame)
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
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.shell_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
}

fn shared_root_body_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.host_body_frame)
        .map(shell_frame)
        .filter(|frame| frame_is_visible(*frame))
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
