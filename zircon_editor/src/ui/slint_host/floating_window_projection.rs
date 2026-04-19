use std::collections::BTreeMap;

use zircon_runtime::ui::layout::UiFrame;

use crate::ui::slint_host::callback_dispatch::BuiltinFloatingWindowSourceFrames;
use crate::{
    autolayout::{clamp_floating_window_frame, default_floating_window_frame},
    FloatingWindowModel, MainPageId, NativeWindowHostState, ShellFrame, WorkbenchChromeMetrics,
    WorkbenchViewModel,
};
#[cfg(test)]
use crate::{ShellRegionId, WorkbenchShellGeometry};

const EPSILON: f32 = 0.001;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FloatingWindowProjectionFrames {
    pub outer_frame: ShellFrame,
    pub tab_strip_frame: ShellFrame,
    pub content_frame: ShellFrame,
    pub host_frame: Option<ShellFrame>,
    pub native_host_present: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FloatingWindowProjectionBundle {
    frames_by_window_id: BTreeMap<MainPageId, FloatingWindowProjectionFrames>,
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
    let frames_by_window_id = floating_windows
        .iter()
        .enumerate()
        .map(|(window_index, window)| {
            let native_host_present = native_window_hosts
                .iter()
                .any(|host| host.window_id == window.window_id);
            let host_frame =
                resolve_native_floating_window_host_frame(native_window_hosts, &window.window_id);
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

#[cfg(test)]
pub(crate) fn floating_window_projection_shared_source_from_geometry(
    geometry: &WorkbenchShellGeometry,
) -> Option<FloatingWindowProjectionSharedSource> {
    let document_frame = geometry.region_frame(ShellRegionId::Document);
    let center_band_frame = geometry.center_band_frame;
    Some(FloatingWindowProjectionSharedSource {
        document_frame,
        center_band_frame,
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
        .and_then(|host| {
            (host.bounds[2] > 0.0 && host.bounds[3] > 0.0).then_some(ShellFrame::new(
                host.bounds[0],
                host.bounds[1],
                host.bounds[2],
                host.bounds[3],
            ))
        })
}

#[cfg(test)]
pub(crate) fn build_floating_window_projection_bundle(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    native_window_hosts: &[NativeWindowHostState],
) -> FloatingWindowProjectionBundle {
    build_floating_window_projection_bundle_from_windows_with_geometry(
        &model.floating_windows,
        geometry,
        metrics,
        native_window_hosts,
    )
}

#[cfg(test)]
pub(crate) fn build_floating_window_projection_bundle_from_windows(
    floating_windows: &[FloatingWindowModel],
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    native_window_hosts: &[NativeWindowHostState],
) -> FloatingWindowProjectionBundle {
    build_floating_window_projection_bundle_from_windows_with_geometry(
        floating_windows,
        geometry,
        metrics,
        native_window_hosts,
    )
}

#[cfg(test)]
fn build_floating_window_projection_bundle_from_windows_with_geometry(
    floating_windows: &[FloatingWindowModel],
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    native_window_hosts: &[NativeWindowHostState],
) -> FloatingWindowProjectionBundle {
    let shared_source = floating_window_projection_shared_source_from_geometry(geometry);
    let frames_by_window_id = floating_windows
        .iter()
        .enumerate()
        .map(|(window_index, window)| {
            let native_host_present = native_window_hosts
                .iter()
                .any(|host| host.window_id == window.window_id);
            let host_frame =
                resolve_native_floating_window_host_frame(native_window_hosts, &window.window_id);
            let outer_frame = resolve_floating_window_projected_outer_frame_with_fallback(
                window,
                window_index,
                shared_source,
                host_frame,
                geometry.floating_window_frame(&window.window_id),
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
                },
            )
        })
        .collect();

    FloatingWindowProjectionBundle {
        frames_by_window_id,
    }
}

fn resolve_floating_window_projected_outer_frame_with_fallback(
    window: &FloatingWindowModel,
    window_index: usize,
    shared_source: Option<FloatingWindowProjectionSharedSource>,
    host_frame: Option<ShellFrame>,
    geometry_fallback: ShellFrame,
) -> ShellFrame {
    if let Some(host_frame) = host_frame.filter(|frame| frame.width > 0.0 && frame.height > 0.0) {
        return host_frame;
    }

    resolve_floating_window_outer_frame_from_shared_source(
        window.requested_frame,
        window_index,
        shared_source,
    )
    .or_else(|| valid_requested_frame(geometry_fallback))
    .or_else(|| valid_requested_frame(window.requested_frame))
    .unwrap_or_default()
}

fn resolve_floating_window_projected_outer_frame_with_host_frame(
    window: &FloatingWindowModel,
    window_index: usize,
    shared_source: Option<FloatingWindowProjectionSharedSource>,
    host_frame: Option<ShellFrame>,
) -> ShellFrame {
    resolve_floating_window_projected_outer_frame_with_fallback(
        window,
        window_index,
        shared_source,
        host_frame,
        ShellFrame::default(),
    )
}

#[cfg(test)]
pub(crate) fn resolve_floating_window_outer_frame(
    geometry: &WorkbenchShellGeometry,
    window_id: &MainPageId,
) -> ShellFrame {
    resolve_floating_window_outer_frame_with_host_frame(geometry, window_id, None)
}

#[cfg(test)]
pub(crate) fn resolve_floating_window_outer_frame_with_host_frame(
    geometry: &WorkbenchShellGeometry,
    window_id: &MainPageId,
    host_frame: Option<ShellFrame>,
) -> ShellFrame {
    if let Some(host_frame) = host_frame.filter(|frame| frame.width > 0.0 && frame.height > 0.0) {
        return host_frame;
    }
    geometry.floating_window_frame(window_id)
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
pub(crate) fn resolve_floating_window_tab_strip_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    window_id: &MainPageId,
) -> ShellFrame {
    resolve_floating_window_tab_strip_frame_with_host_frame(geometry, metrics, window_id, None)
}

#[cfg(test)]
pub(crate) fn resolve_floating_window_tab_strip_frame_with_host_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    window_id: &MainPageId,
    host_frame: Option<ShellFrame>,
) -> ShellFrame {
    let outer =
        resolve_floating_window_outer_frame_with_host_frame(geometry, window_id, host_frame);
    ShellFrame::new(
        outer.x,
        outer.y,
        outer.width.max(0.0),
        metrics.document_header_height.max(0.0),
    )
}

#[cfg(test)]
pub(crate) fn resolve_floating_window_content_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    window_id: &MainPageId,
) -> ShellFrame {
    resolve_floating_window_content_frame_with_host_frame(geometry, metrics, window_id, None)
}

#[cfg(test)]
pub(crate) fn resolve_floating_window_content_frame_with_host_frame(
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    window_id: &MainPageId,
    host_frame: Option<ShellFrame>,
) -> ShellFrame {
    let outer =
        resolve_floating_window_outer_frame_with_host_frame(geometry, window_id, host_frame);
    let header_height = metrics.document_header_height.max(0.0);
    let separator_height = metrics.separator_thickness.max(0.0);
    ShellFrame::new(
        outer.x,
        outer.y + header_height + separator_height,
        outer.width.max(0.0),
        (outer.height - header_height - separator_height).max(0.0),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        DocumentWorkspaceSnapshot, DrawerRingModel, FloatingWindowModel, MainHostStripModel,
        MainHostStripViewModel, MainPageId, MenuBarModel, NativeWindowHostState, ShellFrame,
        StatusBarModel, WorkbenchChromeMetrics, WorkbenchShellGeometry, WorkbenchViewModel,
    };

    use super::{
        build_floating_window_projection_bundle,
        build_floating_window_projection_bundle_with_shared_source, default_floating_window_frame,
        resolve_floating_window_content_frame,
        resolve_floating_window_content_frame_with_host_frame, resolve_floating_window_outer_frame,
        resolve_floating_window_outer_frame_with_host_frame,
        resolve_floating_window_tab_strip_frame,
        resolve_floating_window_tab_strip_frame_with_host_frame, FloatingWindowProjectionFrames,
        FloatingWindowProjectionSharedSource,
    };

    #[test]
    fn floating_window_projection_splits_outer_frame_into_strip_and_content() {
        let window_id = MainPageId::new("window:preview");
        let metrics = WorkbenchChromeMetrics::default();
        let mut geometry = WorkbenchShellGeometry {
            floating_window_frames: BTreeMap::new(),
            ..Default::default()
        };
        geometry.floating_window_frames.insert(
            window_id.clone(),
            ShellFrame::new(120.0, 84.0, 520.0, 340.0),
        );

        assert_eq!(
            resolve_floating_window_outer_frame(&geometry, &window_id),
            ShellFrame::new(120.0, 84.0, 520.0, 340.0)
        );
        assert_eq!(
            resolve_floating_window_tab_strip_frame(&geometry, &metrics, &window_id),
            ShellFrame::new(120.0, 84.0, 520.0, metrics.document_header_height)
        );
        assert_eq!(
            resolve_floating_window_content_frame(&geometry, &metrics, &window_id),
            ShellFrame::new(
                120.0,
                84.0 + metrics.document_header_height + metrics.separator_thickness,
                520.0,
                340.0 - metrics.document_header_height - metrics.separator_thickness,
            )
        );
    }

    #[test]
    fn floating_window_projection_clamps_content_height_when_shell_is_shorter_than_header() {
        let window_id = MainPageId::new("window:tiny");
        let metrics = WorkbenchChromeMetrics::default();
        let mut geometry = WorkbenchShellGeometry {
            floating_window_frames: BTreeMap::new(),
            ..Default::default()
        };
        geometry
            .floating_window_frames
            .insert(window_id.clone(), ShellFrame::new(0.0, 0.0, 180.0, 8.0));

        assert_eq!(
            resolve_floating_window_content_frame(&geometry, &metrics, &window_id),
            ShellFrame::new(
                0.0,
                metrics.document_header_height + metrics.separator_thickness,
                180.0,
                0.0,
            )
        );
    }

    #[test]
    fn floating_window_projection_prefers_host_bounds_when_present() {
        let window_id = MainPageId::new("window:hosted");
        let metrics = WorkbenchChromeMetrics::default();
        let mut geometry = WorkbenchShellGeometry {
            floating_window_frames: BTreeMap::new(),
            ..Default::default()
        };
        geometry.floating_window_frames.insert(
            window_id.clone(),
            ShellFrame::new(120.0, 84.0, 520.0, 340.0),
        );
        let host_frame = ShellFrame::new(640.0, 320.0, 700.0, 420.0);

        assert_eq!(
            resolve_floating_window_outer_frame_with_host_frame(
                &geometry,
                &window_id,
                Some(host_frame),
            ),
            host_frame
        );
        assert_eq!(
            resolve_floating_window_tab_strip_frame_with_host_frame(
                &geometry,
                &metrics,
                &window_id,
                Some(host_frame),
            ),
            ShellFrame::new(
                host_frame.x,
                host_frame.y,
                host_frame.width,
                metrics.document_header_height,
            )
        );
        assert_eq!(
            resolve_floating_window_content_frame_with_host_frame(
                &geometry,
                &metrics,
                &window_id,
                Some(host_frame),
            ),
            ShellFrame::new(
                host_frame.x,
                host_frame.y + metrics.document_header_height + metrics.separator_thickness,
                host_frame.width,
                host_frame.height - metrics.document_header_height - metrics.separator_thickness,
            )
        );
        assert_eq!(
            resolve_floating_window_outer_frame(&geometry, &window_id),
            ShellFrame::new(120.0, 84.0, 520.0, 340.0),
            "legacy helper without host frame should keep layout geometry semantics"
        );
        assert_eq!(
            resolve_floating_window_tab_strip_frame(&geometry, &metrics, &window_id),
            ShellFrame::new(120.0, 84.0, 520.0, metrics.document_header_height),
        );
        assert_eq!(
            resolve_floating_window_content_frame(&geometry, &metrics, &window_id),
            ShellFrame::new(
                120.0,
                84.0 + metrics.document_header_height + metrics.separator_thickness,
                520.0,
                340.0 - metrics.document_header_height - metrics.separator_thickness,
            ),
        );
    }

    #[test]
    fn build_floating_window_projection_bundle_prefers_native_host_bounds_and_splits_frames() {
        let window_id = MainPageId::new("window:bundle-hosted");
        let metrics = WorkbenchChromeMetrics::default();
        let mut geometry = WorkbenchShellGeometry {
            floating_window_frames: BTreeMap::new(),
            ..Default::default()
        };
        geometry.floating_window_frames.insert(
            window_id.clone(),
            ShellFrame::new(120.0, 84.0, 520.0, 340.0),
        );
        let host_frame = ShellFrame::new(640.0, 320.0, 700.0, 420.0);

        let bundle = build_floating_window_projection_bundle(
            &floating_window_projection_model(window_id.clone(), ShellFrame::default()),
            &geometry,
            &metrics,
            &[NativeWindowHostState {
                window_id: window_id.clone(),
                handle: Some(7),
                bounds: [
                    host_frame.x,
                    host_frame.y,
                    host_frame.width,
                    host_frame.height,
                ],
            }],
        );

        assert_eq!(
            bundle.frames(&window_id),
            Some(&FloatingWindowProjectionFrames {
                outer_frame: host_frame,
                tab_strip_frame: ShellFrame::new(
                    host_frame.x,
                    host_frame.y,
                    host_frame.width,
                    metrics.document_header_height,
                ),
                content_frame: ShellFrame::new(
                    host_frame.x,
                    host_frame.y + metrics.document_header_height + metrics.separator_thickness,
                    host_frame.width,
                    host_frame.height
                        - metrics.document_header_height
                        - metrics.separator_thickness,
                ),
                host_frame: Some(host_frame),
                native_host_present: true,
            })
        );
    }

    #[test]
    fn build_floating_window_projection_bundle_prefers_shared_projection_source_over_stale_geometry(
    ) {
        let window_id = MainPageId::new("window:bundle-shared-source");
        let metrics = WorkbenchChromeMetrics::default();
        let requested_frame = ShellFrame::new(1040.0, 188.0, 640.0, 420.0);
        let shared_source = FloatingWindowProjectionSharedSource {
            document_frame: ShellFrame::new(240.0, 96.0, 1120.0, 720.0),
            center_band_frame: ShellFrame::new(0.0, 80.0, 1440.0, 760.0),
        };

        let bundle = build_floating_window_projection_bundle_with_shared_source(
            &floating_window_projection_model(window_id.clone(), requested_frame),
            Some(shared_source),
            &metrics,
            &[],
        );

        assert_eq!(
            bundle.frames(&window_id),
            Some(&FloatingWindowProjectionFrames {
                outer_frame: ShellFrame::new(800.0, 188.0, 640.0, 420.0),
                tab_strip_frame: ShellFrame::new(
                    800.0,
                    188.0,
                    640.0,
                    metrics.document_header_height
                ),
                content_frame: ShellFrame::new(
                    800.0,
                    188.0 + metrics.document_header_height + metrics.separator_thickness,
                    640.0,
                    420.0 - metrics.document_header_height - metrics.separator_thickness,
                ),
                host_frame: None,
                native_host_present: false,
            }),
            "shared floating-window projection source should clamp the requested frame without any geometry fallback"
        );
    }

    #[test]
    fn build_floating_window_projection_bundle_uses_shared_default_frame_when_requested_frame_is_missing(
    ) {
        let window_id = MainPageId::new("window:bundle-shared-default");
        let metrics = WorkbenchChromeMetrics::default();
        let shared_source = FloatingWindowProjectionSharedSource {
            document_frame: ShellFrame::new(240.0, 96.0, 1120.0, 720.0),
            center_band_frame: ShellFrame::new(0.0, 80.0, 1440.0, 760.0),
        };
        let expected_outer_frame = default_floating_window_frame(
            0,
            shared_source.document_frame,
            shared_source.center_band_frame,
        );

        let bundle = build_floating_window_projection_bundle_with_shared_source(
            &floating_window_projection_model(window_id.clone(), ShellFrame::default()),
            Some(shared_source),
            &metrics,
            &[],
        );

        assert_eq!(
            bundle.frames(&window_id),
            Some(&FloatingWindowProjectionFrames {
                outer_frame: expected_outer_frame,
                tab_strip_frame: ShellFrame::new(
                    expected_outer_frame.x,
                    expected_outer_frame.y,
                    expected_outer_frame.width,
                    metrics.document_header_height,
                ),
                content_frame: ShellFrame::new(
                    expected_outer_frame.x,
                    expected_outer_frame.y
                        + metrics.document_header_height
                        + metrics.separator_thickness,
                    expected_outer_frame.width,
                    expected_outer_frame.height
                        - metrics.document_header_height
                        - metrics.separator_thickness,
                ),
                host_frame: None,
                native_host_present: false,
            }),
            "missing requested frames should now be synthesized from shared source rather than legacy geometry"
        );
    }

    #[test]
    fn build_floating_window_projection_bundle_falls_back_to_geometry_when_host_bounds_are_empty() {
        let window_id = MainPageId::new("window:bundle-fallback");
        let metrics = WorkbenchChromeMetrics::default();
        let mut geometry = WorkbenchShellGeometry {
            floating_window_frames: BTreeMap::new(),
            ..Default::default()
        };
        let geometry_frame = ShellFrame::new(120.0, 84.0, 520.0, 340.0);
        geometry
            .floating_window_frames
            .insert(window_id.clone(), geometry_frame);

        let bundle = build_floating_window_projection_bundle(
            &floating_window_projection_model(window_id.clone(), ShellFrame::default()),
            &geometry,
            &metrics,
            &[NativeWindowHostState {
                window_id: window_id.clone(),
                handle: Some(9),
                bounds: [0.0, 0.0, 0.0, 0.0],
            }],
        );

        assert_eq!(
            bundle.frames(&window_id),
            Some(&FloatingWindowProjectionFrames {
                outer_frame: geometry_frame,
                tab_strip_frame: ShellFrame::new(
                    geometry_frame.x,
                    geometry_frame.y,
                    geometry_frame.width,
                    metrics.document_header_height,
                ),
                content_frame: ShellFrame::new(
                    geometry_frame.x,
                    geometry_frame.y + metrics.document_header_height + metrics.separator_thickness,
                    geometry_frame.width,
                    geometry_frame.height
                        - metrics.document_header_height
                        - metrics.separator_thickness,
                ),
                host_frame: None,
                native_host_present: true,
            })
        );
    }

    fn floating_window_projection_model(
        window_id: MainPageId,
        requested_frame: ShellFrame,
    ) -> WorkbenchViewModel {
        WorkbenchViewModel {
            menu_bar: MenuBarModel { menus: Vec::new() },
            host_strip: MainHostStripViewModel {
                mode: MainHostStripModel::Workbench,
                pages: Vec::new(),
                active_page: MainPageId::workbench(),
                breadcrumbs: Vec::new(),
            },
            drawer_ring: DrawerRingModel {
                visible: false,
                drawers: BTreeMap::new(),
            },
            tool_windows: BTreeMap::new(),
            document_tabs: Vec::new(),
            floating_windows: vec![FloatingWindowModel {
                window_id,
                title: "Preview".to_string(),
                requested_frame,
                focused_view: None,
                tabs: Vec::new(),
            }],
            document: crate::DocumentWorkspaceModel::Workbench {
                page_id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                workspace: DocumentWorkspaceSnapshot::Tabs {
                    tabs: Vec::new(),
                    active_tab: None,
                },
            },
            status_bar: StatusBarModel {
                primary_text: String::new(),
                secondary_text: None,
                viewport_label: String::new(),
            },
        }
    }
}
