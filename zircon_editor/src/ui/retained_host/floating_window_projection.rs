use std::collections::HashMap;

use zircon_runtime_interface::ui::{event_ui::UiTreeId, layout::UiFrame};

use crate::ui::host::NativeWindowHostState;
use crate::ui::retained_host::callback_dispatch::BuiltinFloatingWindowSourceFrames;
use crate::ui::workbench::autolayout::{
    clamp_floating_window_frame, default_floating_window_frame, ShellFrame, WorkbenchChromeMetrics,
};
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::{FloatingWindowModel, WorkbenchViewModel};

const EPSILON: f32 = 0.001;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FloatingWindowProjectionFrames {
    pub outer_frame: ShellFrame,
    pub tab_strip_frame: ShellFrame,
    pub content_frame: ShellFrame,
    pub host_frame: Option<ShellFrame>,
    pub native_host_present: bool,
    pub surface_tree_id: Option<UiTreeId>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FloatingWindowProjectionBundle {
    frames_by_window_id: HashMap<MainPageId, FloatingWindowProjectionFrames>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct FloatingWindowProjectionSharedSource {
    pub document_frame: ShellFrame,
    pub center_band_frame: ShellFrame,
}

impl FloatingWindowProjectionBundle {
    pub(crate) fn frames(&self, window_id: &MainPageId) -> Option<&FloatingWindowProjectionFrames> {
        self.frames_by_window_id.get(window_id)
    }

    pub(crate) fn outer_frame(&self, window_id: &MainPageId) -> Option<ShellFrame> {
        self.frames(window_id).map(|frames| frames.outer_frame)
    }

    pub(crate) fn tab_strip_frame(&self, window_id: &MainPageId) -> Option<ShellFrame> {
        self.frames(window_id).map(|frames| frames.tab_strip_frame)
    }

    pub(crate) fn content_frame(&self, window_id: &MainPageId) -> Option<ShellFrame> {
        self.frames(window_id).map(|frames| frames.content_frame)
    }
}

pub(crate) fn build_floating_window_projection_bundle_with_shared_source(
    model: &WorkbenchViewModel,
    shared_source: Option<FloatingWindowProjectionSharedSource>,
    metrics: &WorkbenchChromeMetrics,
    native_window_hosts: &[NativeWindowHostState],
) -> FloatingWindowProjectionBundle {
    build_floating_window_projection_bundle_from_windows_with_shared_source(
        &model.floating_windows,
        shared_source,
        metrics,
        native_window_hosts,
    )
}

pub(crate) fn build_floating_window_projection_bundle_from_windows_with_shared_source(
    floating_windows: &[FloatingWindowModel],
    shared_source: Option<FloatingWindowProjectionSharedSource>,
    metrics: &WorkbenchChromeMetrics,
    native_window_hosts: &[NativeWindowHostState],
) -> FloatingWindowProjectionBundle {
    zircon_runtime::profile_counter!("editor", "ui.floating_projection.bundle_build_count", 1);
    zircon_runtime::profile_counter!(
        "editor",
        "ui.floating_projection.native_host_row_count",
        native_window_hosts.len()
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.floating_projection.window_row_count",
        floating_windows.len()
    );
    let mut native_hosts_by_window_id = HashMap::with_capacity(native_window_hosts.len());
    for host in native_window_hosts {
        native_hosts_by_window_id
            .entry(host.window_id.0.as_str())
            .or_insert(host);
    }
    let frames_by_window_id = floating_windows
        .iter()
        .enumerate()
        .map(|(window_index, window)| {
            let native_host = native_hosts_by_window_id
                .get(window.window_id.0.as_str())
                .copied();
            let native_host_present = native_host.is_some();
            let host_frame = native_host.and_then(native_floating_window_host_frame);
            let outer_frame = resolve_floating_window_projected_outer_frame_with_host_frame(
                window,
                window_index,
                shared_source,
                host_frame,
            );
            let tab_strip_frame = ShellFrame::new(
                outer_frame.x,
                outer_frame.y,
                outer_frame.width.max(0.0),
                metrics.document_header_height.max(0.0),
            );
            let content_frame = ShellFrame::new(
                outer_frame.x,
                outer_frame.y
                    + metrics.document_header_height.max(0.0)
                    + metrics.separator_thickness.max(0.0),
                outer_frame.width.max(0.0),
                (outer_frame.height
                    - metrics.document_header_height.max(0.0)
                    - metrics.separator_thickness.max(0.0))
                .max(0.0),
            );
            (
                window.window_id.clone(),
                FloatingWindowProjectionFrames {
                    outer_frame,
                    tab_strip_frame,
                    content_frame,
                    host_frame,
                    native_host_present,
                    surface_tree_id: native_host.map(|host| host.surface_tree_id.clone()),
                },
            )
        })
        .collect();

    FloatingWindowProjectionBundle {
        frames_by_window_id,
    }
}

pub(crate) fn resolve_floating_window_projection_shared_source(
    source_frames: &BuiltinFloatingWindowSourceFrames,
) -> Option<FloatingWindowProjectionSharedSource> {
    Some(FloatingWindowProjectionSharedSource {
        document_frame: shell_frame_from_ui_frame(source_frames.document_frame?),
        center_band_frame: shell_frame_from_ui_frame(source_frames.center_band_frame?),
    })
    .filter(|source| {
        source.document_frame.width > EPSILON
            && source.document_frame.height > EPSILON
            && source.center_band_frame.width > EPSILON
            && source.center_band_frame.height > EPSILON
    })
}

pub(crate) fn resolve_floating_window_projection_base_outer_frame(
    window: &FloatingWindowModel,
    window_index: usize,
    shared_source: Option<FloatingWindowProjectionSharedSource>,
) -> ShellFrame {
    resolve_floating_window_outer_frame_from_shared_source(
        window.requested_frame,
        window_index,
        shared_source,
    )
    .unwrap_or_else(|| valid_requested_frame(window.requested_frame).unwrap_or_default())
}

#[cfg(test)]
pub(crate) fn resolve_floating_window_projection_content_frame(
    window: &FloatingWindowModel,
    window_index: usize,
    shared_source: Option<FloatingWindowProjectionSharedSource>,
    metrics: &WorkbenchChromeMetrics,
    host_frame: Option<ShellFrame>,
) -> ShellFrame {
    let outer_frame = resolve_floating_window_projected_outer_frame_with_host_frame(
        window,
        window_index,
        shared_source,
        host_frame,
    );
    let header_height = metrics.document_header_height.max(0.0);
    let separator_height = metrics.separator_thickness.max(0.0);
    ShellFrame::new(
        outer_frame.x,
        outer_frame.y + header_height + separator_height,
        outer_frame.width.max(0.0),
        (outer_frame.height - header_height - separator_height).max(0.0),
    )
}

pub(crate) fn resolve_native_floating_window_host_frame(
    native_window_hosts: &[NativeWindowHostState],
    window_id: &MainPageId,
) -> Option<ShellFrame> {
    native_window_hosts
        .iter()
        .find(|host| &host.window_id == window_id)
        .and_then(native_floating_window_host_frame)
}

fn native_floating_window_host_frame(host: &NativeWindowHostState) -> Option<ShellFrame> {
    (host.bounds[2] > 0.0 && host.bounds[3] > 0.0).then_some(ShellFrame::new(
        host.bounds[0],
        host.bounds[1],
        host.bounds[2],
        host.bounds[3],
    ))
}

#[cfg(test)]
pub(crate) fn build_floating_window_projection_bundle(
    model: &WorkbenchViewModel,
    shared_source: Option<FloatingWindowProjectionSharedSource>,
    metrics: &WorkbenchChromeMetrics,
    native_window_hosts: &[NativeWindowHostState],
) -> FloatingWindowProjectionBundle {
    build_floating_window_projection_bundle_from_windows_with_shared_source(
        &model.floating_windows,
        shared_source,
        metrics,
        native_window_hosts,
    )
}

#[cfg(test)]
pub(crate) fn build_floating_window_projection_bundle_from_windows(
    floating_windows: &[FloatingWindowModel],
    shared_source: Option<FloatingWindowProjectionSharedSource>,
    metrics: &WorkbenchChromeMetrics,
    native_window_hosts: &[NativeWindowHostState],
) -> FloatingWindowProjectionBundle {
    build_floating_window_projection_bundle_from_windows_with_shared_source(
        floating_windows,
        shared_source,
        metrics,
        native_window_hosts,
    )
}

fn resolve_floating_window_projected_outer_frame_with_host_frame(
    window: &FloatingWindowModel,
    window_index: usize,
    shared_source: Option<FloatingWindowProjectionSharedSource>,
    host_frame: Option<ShellFrame>,
) -> ShellFrame {
    if let Some(host_frame) = host_frame.filter(|frame| frame.width > 0.0 && frame.height > 0.0) {
        return host_frame;
    }
    resolve_floating_window_outer_frame_from_shared_source(
        window.requested_frame,
        window_index,
        shared_source,
    )
    .or_else(|| valid_requested_frame(window.requested_frame))
    .unwrap_or_default()
}

fn resolve_floating_window_outer_frame_from_shared_source(
    requested_frame: ShellFrame,
    window_index: usize,
    shared_source: Option<FloatingWindowProjectionSharedSource>,
) -> Option<ShellFrame> {
    let shared_source = shared_source?;
    let requested_frame = if let Some(requested_frame) = valid_requested_frame(requested_frame) {
        requested_frame
    } else {
        default_floating_window_frame(
            window_index,
            shared_source.document_frame,
            shared_source.center_band_frame,
        )
    };
    Some(clamp_floating_window_frame(
        requested_frame,
        shared_source.center_band_frame,
    ))
}

fn valid_requested_frame(frame: ShellFrame) -> Option<ShellFrame> {
    (frame.width > EPSILON && frame.height > EPSILON).then_some(frame)
}

fn shell_frame_from_ui_frame(frame: UiFrame) -> ShellFrame {
    ShellFrame::new(frame.x, frame.y, frame.width, frame.height)
}

#[cfg(test)]
#[path = "tests/floating_window_projection.rs"]
mod tests;
