use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_core::CoreRuntime;
use zircon_foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_manager::resolve_config_manager;

use crate::host::slint_host::drawer_resize::{
    apply_resize_to_group, resolve_workbench_resize_target_group, WorkbenchResizeTargetGroup,
};
use crate::host::slint_host::shell_pointer::{
    WorkbenchShellPointerBridge, WorkbenchShellPointerRoute,
};
use crate::host::slint_host::tab_drag::WorkbenchDragTargetGroup;
use crate::{
    ActivityDrawerSlot, EditorManager, ShellFrame, ShellRegionId, ShellSizePx,
    WorkbenchShellGeometry, EDITOR_MANAGER_NAME,
};
use zircon_ui::UiPoint;

fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn env_lock() -> &'static Mutex<()> {
    crate::tests::support::env_lock()
}

fn editor_runtime_with_config_path(path: &Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_asset::module_descriptor())
        .unwrap();
    runtime
        .register_module(crate::module::module_descriptor())
        .unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(crate::module::EDITOR_MODULE_NAME)
        .unwrap();
    runtime
}

#[test]
fn apply_resize_to_left_group_updates_all_left_slots() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_slint_left_resize");
    let runtime = editor_runtime_with_config_path(&path);
    let _config = resolve_config_manager(&runtime.handle()).unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    apply_resize_to_group(manager.as_ref(), "left", 344.0).unwrap();

    let layout = manager.current_layout();
    assert_eq!(layout.drawers[&ActivityDrawerSlot::LeftTop].extent, 344.0);
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::LeftBottom].extent,
        344.0
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn apply_resize_to_bottom_group_updates_all_bottom_slots() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_slint_bottom_resize");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    apply_resize_to_group(manager.as_ref(), "bottom", 228.0).unwrap();

    let layout = manager.current_layout();
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::BottomLeft].extent,
        228.0
    );
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::BottomRight].extent,
        228.0
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn apply_resize_to_right_group_updates_all_right_slots() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_slint_right_resize");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    apply_resize_to_group(manager.as_ref(), "right", 312.0).unwrap();

    let layout = manager.current_layout();
    assert_eq!(layout.drawers[&ActivityDrawerSlot::RightTop].extent, 312.0);
    assert_eq!(
        layout.drawers[&ActivityDrawerSlot::RightBottom].extent,
        312.0
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn apply_resize_to_unknown_group_returns_error() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_slint_unknown_resize");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let error = apply_resize_to_group(manager.as_ref(), "mystery", 240.0).unwrap_err();
    assert!(
        error.contains("Unsupported drawer resize target"),
        "unexpected error: {error}"
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn shared_resize_target_route_resolves_left_right_and_bottom_splitters() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 48.0, 1440.0, 832.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::new(),
        splitter_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(308.0, 48.0, 8.0, 832.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1124.0, 48.0, 8.0, 832.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 716.0, 1440.0, 8.0),
            ),
        ]),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(0.0, 0.0, 0.0, 0.0),
    };
    let shell_size = ShellSizePx::new(1440.0, 900.0);

    assert_eq!(
        resolve_workbench_resize_target_group(shell_size, &geometry, UiPoint::new(312.0, 420.0)),
        Some(WorkbenchResizeTargetGroup::Left)
    );
    assert_eq!(
        resolve_workbench_resize_target_group(shell_size, &geometry, UiPoint::new(1128.0, 420.0)),
        Some(WorkbenchResizeTargetGroup::Right)
    );
    assert_eq!(
        resolve_workbench_resize_target_group(shell_size, &geometry, UiPoint::new(720.0, 720.0)),
        Some(WorkbenchResizeTargetGroup::Bottom)
    );
}

#[test]
fn shared_resize_target_route_ignores_points_outside_splitter_frames() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 48.0, 1440.0, 832.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::new(),
        splitter_frames: BTreeMap::from([(
            ShellRegionId::Left,
            ShellFrame::new(308.0, 48.0, 8.0, 832.0),
        )]),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(0.0, 0.0, 0.0, 0.0),
    };

    assert_eq!(
        resolve_workbench_resize_target_group(
            ShellSizePx::new(1440.0, 900.0),
            &geometry,
            UiPoint::new(420.0, 420.0),
        ),
        None
    );
}

#[test]
fn unified_shell_pointer_bridge_routes_drag_targets_and_resize_targets_from_one_surface() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 48.0, 1440.0, 832.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(0.0, 48.0, 308.0, 832.0),
            ),
            (
                ShellRegionId::Document,
                ShellFrame::new(316.0, 48.0, 808.0, 668.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1132.0, 48.0, 308.0, 668.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 724.0, 1440.0, 156.0),
            ),
        ]),
        splitter_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(308.0, 48.0, 8.0, 832.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1124.0, 48.0, 8.0, 668.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 716.0, 1440.0, 8.0),
            ),
        ]),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(316.0, 82.0, 808.0, 634.0),
    };
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        ShellSizePx::new(1440.0, 900.0),
        &geometry,
        true,
        &[],
        &[],
    );

    assert_eq!(
        bridge.drag_target_at(UiPoint::new(1428.0, 240.0)),
        Some(WorkbenchDragTargetGroup::Right)
    );
    assert_eq!(
        bridge.begin_resize(UiPoint::new(312.0, 420.0)),
        Some(WorkbenchShellPointerRoute::Resize(
            WorkbenchResizeTargetGroup::Left
        ))
    );
}

#[test]
fn unified_shell_pointer_bridge_keeps_resize_route_captured_until_pointer_up() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 48.0, 1440.0, 832.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(0.0, 48.0, 308.0, 832.0),
            ),
            (
                ShellRegionId::Document,
                ShellFrame::new(316.0, 48.0, 1124.0, 668.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 724.0, 1440.0, 156.0),
            ),
        ]),
        splitter_frames: BTreeMap::from([(
            ShellRegionId::Left,
            ShellFrame::new(308.0, 48.0, 8.0, 832.0),
        )]),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(316.0, 82.0, 1124.0, 634.0),
    };
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        ShellSizePx::new(1440.0, 900.0),
        &geometry,
        true,
        &[],
        &[],
    );

    assert_eq!(
        bridge.begin_resize(UiPoint::new(312.0, 420.0)),
        Some(WorkbenchShellPointerRoute::Resize(
            WorkbenchResizeTargetGroup::Left
        ))
    );
    assert_eq!(
        bridge.update_resize(UiPoint::new(900.0, 420.0)),
        Some(WorkbenchResizeTargetGroup::Left)
    );
    assert_eq!(
        bridge.finish_resize(UiPoint::new(900.0, 420.0)),
        Some(WorkbenchResizeTargetGroup::Left)
    );
    assert_eq!(bridge.update_resize(UiPoint::new(900.0, 420.0)), None);
}

#[test]
fn shared_resize_surface_replaces_legacy_direct_resize_callback_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/callback_wiring.rs"
    ));
    let docking = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/workspace_docking.rs"
    ));

    for needle in [
        "callback begin_drawer_resize(x: float, y: float);",
        "callback update_drawer_resize(x: float, y: float);",
        "callback finish_drawer_resize(x: float, y: float);",
        "root.begin_drawer_resize(",
        "root.update_drawer_resize(",
        "root.finish_drawer_resize(",
        "ui.on_begin_drawer_resize(",
        "ui.on_update_drawer_resize(",
        "ui.on_finish_drawer_resize(",
        "fn begin_drawer_resize(",
        "fn update_drawer_resize(",
        "fn finish_drawer_resize(",
    ] {
        let found =
            workbench.contains(needle) || wiring.contains(needle) || docking.contains(needle);
        assert!(
            !found,
            "drawer resize path still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "callback workbench_resize_pointer_event(kind: int, x: float, y: float);",
        "root.workbench_resize_pointer_event(",
    ] {
        assert!(
            workbench.contains(needle),
            "workbench shell is missing shared resize pointer hook `{needle}`"
        );
    }

    assert!(
        wiring.contains("ui.on_workbench_resize_pointer_event("),
        "slint host callback wiring must register shared resize pointer callback"
    );
    assert!(
        docking.contains("fn workbench_resize_pointer_event("),
        "workspace docking host must handle shared resize pointer events"
    );
}
