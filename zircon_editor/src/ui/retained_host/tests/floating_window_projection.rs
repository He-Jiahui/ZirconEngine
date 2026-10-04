use std::collections::BTreeMap;

use crate::core::commands::{EditorKeymap, MenuBarModel};
use crate::ui::host::NativeWindowHostState;
use crate::ui::workbench::autolayout::{ShellFrame, WorkbenchChromeMetrics};
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::{
    DocumentWorkspaceModel, DrawerRingModel, FloatingWindowModel, MainHostStripModel,
    MainHostStripViewModel, StatusBarModel, WorkbenchViewModel,
};
use crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot;
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use super::{
    build_floating_window_projection_bundle, default_floating_window_frame,
    resolve_floating_window_projection_content_frame, FloatingWindowProjectionFrames,
    FloatingWindowProjectionSharedSource,
};

#[test]
fn floating_window_projection_indexes_native_hosts_once() {
    let source = include_str!("../floating_window_projection.rs");
    let implementation = source.split("#[cfg(test)]").next().unwrap();

    assert!(implementation.contains("HashMap<MainPageId, FloatingWindowProjectionFrames>"));
    assert!(implementation.contains("native_hosts_by_window_id"));
    assert!(implementation.contains(".or_insert(host)"));
    assert!(!implementation.contains("BTreeMap<MainPageId, FloatingWindowProjectionFrames>"));
}

#[test]
fn floating_window_projection_splits_outer_frame_into_strip_and_content() {
    let window_id = MainPageId::new("window:preview");
    let metrics = WorkbenchChromeMetrics::default();
    let requested_frame = ShellFrame::new(120.0, 84.0, 520.0, 340.0);

    let bundle = build_floating_window_projection_bundle(
        &floating_window_projection_model(window_id.clone(), requested_frame),
        Some(shared_source()),
        &metrics,
        &[],
    );

    assert_eq!(
        bundle.frames(&window_id),
        Some(&projection_frames(
            requested_frame,
            &metrics,
            None,
            false,
            None,
        ))
    );
}

#[test]
fn floating_window_projection_clamps_content_height_when_shell_is_shorter_than_header() {
    let window_id = MainPageId::new("window:tiny");
    let metrics = WorkbenchChromeMetrics::default();
    let host_frame = ShellFrame::new(0.0, 0.0, 180.0, 8.0);
    let model = floating_window_projection_model(window_id, ShellFrame::default());

    assert_eq!(
        resolve_floating_window_projection_content_frame(
            &model.floating_windows[0],
            0,
            Some(shared_source()),
            &metrics,
            Some(host_frame),
        ),
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
    let requested_frame = ShellFrame::new(120.0, 84.0, 520.0, 340.0);
    let host_frame = ShellFrame::new(640.0, 320.0, 700.0, 420.0);
    let bundle = build_floating_window_projection_bundle(
        &floating_window_projection_model(window_id.clone(), requested_frame),
        Some(shared_source()),
        &metrics,
        &[NativeWindowHostState::new_for_test(
            window_id.clone(),
            Some(7),
            [
                host_frame.x,
                host_frame.y,
                host_frame.width,
                host_frame.height,
            ],
        )],
    );

    assert_eq!(
        bundle.frames(&window_id),
        Some(&projection_frames(
            host_frame,
            &metrics,
            Some(host_frame),
            true,
            Some("zircon.editor.native_window.window:hosted"),
        ))
    );
}

#[test]
fn build_floating_window_projection_bundle_prefers_native_host_bounds_and_splits_frames() {
    let window_id = MainPageId::new("window:bundle-hosted");
    let metrics = WorkbenchChromeMetrics::default();
    let host_frame = ShellFrame::new(640.0, 320.0, 700.0, 420.0);

    let bundle = build_floating_window_projection_bundle(
        &floating_window_projection_model(window_id.clone(), ShellFrame::default()),
        Some(shared_source()),
        &metrics,
        &[NativeWindowHostState::new_for_test(
            window_id.clone(),
            Some(7),
            [
                host_frame.x,
                host_frame.y,
                host_frame.width,
                host_frame.height,
            ],
        )],
    );

    assert_eq!(
        bundle.frames(&window_id),
        Some(&projection_frames(
            host_frame,
            &metrics,
            Some(host_frame),
            true,
            Some("zircon.editor.native_window.window:bundle-hosted"),
        ))
    );
}

#[test]
fn floating_window_projection_preserves_first_native_host_for_duplicate_window_ids() {
    let window_id = MainPageId::new("window:duplicate-host");
    let metrics = WorkbenchChromeMetrics::default();
    let first_frame = ShellFrame::new(10.0, 20.0, 300.0, 200.0);
    let second_frame = ShellFrame::new(40.0, 50.0, 600.0, 400.0);
    let hosts = [
        NativeWindowHostState::new_for_test(
            window_id.clone(),
            Some(7),
            [
                first_frame.x,
                first_frame.y,
                first_frame.width,
                first_frame.height,
            ],
        ),
        NativeWindowHostState::new_for_test(
            window_id.clone(),
            Some(8),
            [
                second_frame.x,
                second_frame.y,
                second_frame.width,
                second_frame.height,
            ],
        ),
    ];

    let bundle = build_floating_window_projection_bundle(
        &floating_window_projection_model(window_id.clone(), ShellFrame::default()),
        Some(shared_source()),
        &metrics,
        &hosts,
    );

    assert_eq!(bundle.outer_frame(&window_id), Some(first_frame));
}

#[test]
fn build_floating_window_projection_bundle_clamps_requested_frame_with_shared_source() {
    let window_id = MainPageId::new("window:bundle-shared-source");
    let metrics = WorkbenchChromeMetrics::default();
    let requested_frame = ShellFrame::new(1040.0, 188.0, 640.0, 420.0);
    let shared_source = shared_source();

    let bundle = build_floating_window_projection_bundle(
        &floating_window_projection_model(window_id.clone(), requested_frame),
        Some(shared_source),
        &metrics,
        &[],
    );

    assert_eq!(
        bundle.frames(&window_id),
        Some(&FloatingWindowProjectionFrames {
            outer_frame: ShellFrame::new(800.0, 188.0, 640.0, 420.0),
            tab_strip_frame: ShellFrame::new(800.0, 188.0, 640.0, metrics.document_header_height),
            content_frame: ShellFrame::new(
                800.0,
                188.0 + metrics.document_header_height + metrics.separator_thickness,
                640.0,
                420.0 - metrics.document_header_height - metrics.separator_thickness,
            ),
            host_frame: None,
            native_host_present: false,
            surface_tree_id: None,
        }),
        "shared floating-window projection source should clamp the requested frame"
    );
}

#[test]
fn build_floating_window_projection_bundle_uses_shared_default_frame_when_requested_frame_is_missing(
) {
    let window_id = MainPageId::new("window:bundle-shared-default");
    let metrics = WorkbenchChromeMetrics::default();
    let shared_source = shared_source();
    let expected_outer_frame = default_floating_window_frame(
        0,
        shared_source.document_frame,
        shared_source.center_band_frame,
    );

    let bundle = build_floating_window_projection_bundle(
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
            surface_tree_id: None,
        }),
        "missing requested frames should be synthesized from the shared source"
    );
}

#[test]
fn build_floating_window_projection_bundle_uses_requested_frame_when_host_bounds_are_empty() {
    let window_id = MainPageId::new("window:bundle-fallback");
    let metrics = WorkbenchChromeMetrics::default();
    let requested_frame = ShellFrame::new(120.0, 84.0, 520.0, 340.0);

    let bundle = build_floating_window_projection_bundle(
        &floating_window_projection_model(window_id.clone(), requested_frame),
        None,
        &metrics,
        &[NativeWindowHostState::new_for_test(
            window_id.clone(),
            Some(9),
            [0.0, 0.0, 0.0, 0.0],
        )],
    );

    assert_eq!(
        bundle.frames(&window_id),
        Some(&projection_frames(
            requested_frame,
            &metrics,
            None,
            true,
            Some("zircon.editor.native_window.window:bundle-fallback"),
        ))
    );
}

fn shared_source() -> FloatingWindowProjectionSharedSource {
    FloatingWindowProjectionSharedSource {
        document_frame: ShellFrame::new(240.0, 96.0, 1120.0, 720.0),
        center_band_frame: ShellFrame::new(0.0, 80.0, 1440.0, 760.0),
    }
}

fn projection_frames(
    outer_frame: ShellFrame,
    metrics: &WorkbenchChromeMetrics,
    host_frame: Option<ShellFrame>,
    native_host_present: bool,
    surface_tree_id: Option<&str>,
) -> FloatingWindowProjectionFrames {
    FloatingWindowProjectionFrames {
        outer_frame,
        tab_strip_frame: ShellFrame::new(
            outer_frame.x,
            outer_frame.y,
            outer_frame.width,
            metrics.document_header_height,
        ),
        content_frame: ShellFrame::new(
            outer_frame.x,
            outer_frame.y + metrics.document_header_height + metrics.separator_thickness,
            outer_frame.width,
            (outer_frame.height - metrics.document_header_height - metrics.separator_thickness)
                .max(0.0),
        ),
        host_frame,
        native_host_present,
        surface_tree_id: surface_tree_id.map(UiTreeId::new),
    }
}

fn floating_window_projection_model(
    window_id: MainPageId,
    requested_frame: ShellFrame,
) -> WorkbenchViewModel {
    WorkbenchViewModel {
        is_playing: false,
        asset_creation_menu: Default::default(),
        keymap: EditorKeymap::default_workbench(),
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
        document: DocumentWorkspaceModel::Workbench {
            page_id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            workspace: DocumentWorkspaceSnapshot::Tabs {
                node_id: Default::default(),
                tabs: Vec::new(),
                active_tab: None,
            },
        },
        status_bar: StatusBarModel::default(),
    }
}
