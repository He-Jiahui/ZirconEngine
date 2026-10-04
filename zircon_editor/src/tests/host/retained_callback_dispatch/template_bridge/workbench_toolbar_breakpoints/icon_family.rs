use super::super::super::support::{env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, UiSize};
use super::super::support::control_visibility;
use super::support::{
    assert_frame_value, workbench_window_node, COMPACT_WORKBENCH_HEIGHT, COMPACT_WORKBENCH_WIDTH,
    FULL_WORKBENCH_HEIGHT, FULL_WORKBENCH_WIDTH,
};
use zircon_runtime_interface::ui::tree::UiVisibility;

#[test]
fn compact_workbench_module_more_uses_toolbar_overflow_icon() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        COMPACT_WORKBENCH_WIDTH as f32,
        COMPACT_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    let module_more = workbench_window_node(&bridge, "WorkbenchModuleMore");
    assert_eq!(
        module_more.icon_name.as_str(),
        "zircon_editor_shell/toolbar/more-vertical.svg",
        "module overflow should use a toolbar overflow glyph, not a tab/file placeholder"
    );
}

#[test]
fn compact_workbench_file_and_module_commands_use_toolbar_icon_family() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        COMPACT_WORKBENCH_WIDTH as f32,
        COMPACT_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    for (control_id, expected_icon) in [
        (
            "WorkbenchToolbarOpen",
            "zircon_editor_shell/toolbar/folder-open.svg",
        ),
        (
            "WorkbenchToolbarSave",
            "zircon_editor_shell/toolbar/save.svg",
        ),
        (
            "WorkbenchModuleSave",
            "zircon_editor_shell/toolbar/save.svg",
        ),
        (
            "WorkbenchModuleBrowse",
            "zircon_editor_shell/toolbar/folder-open.svg",
        ),
    ] {
        let node = workbench_window_node(&bridge, control_id);
        assert_eq!(
            node.icon_name.as_str(),
            expected_icon,
            "{control_id} should use the shared toolbar icon family"
        );
    }
}

#[test]
fn full_workbench_compile_snap_and_theme_use_toolbar_icon_family() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        FULL_WORKBENCH_WIDTH as f32,
        FULL_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    for (control_id, expected_icon) in [
        (
            "WorkbenchModuleCompile",
            "zircon_editor_shell/toolbar/compile.svg",
        ),
        ("WorkbenchToolSnap", "zircon_editor_shell/toolbar/snap.svg"),
        (
            "WorkbenchThemeToggle",
            "zircon_editor_shell/toolbar/sun.svg",
        ),
    ] {
        let node = workbench_window_node(&bridge, control_id);
        assert_eq!(
            node.icon_name.as_str(),
            expected_icon,
            "{control_id} should use the shared toolbar icon family"
        );
    }
}

#[test]
fn full_workbench_component_lab_and_status_icons_use_shell_asset_paths() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        FULL_WORKBENCH_WIDTH as f32,
        FULL_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    for (control_id, expected_icon) in [
        (
            "WorkbenchButtonIcon",
            "zircon_editor_shell/controls/add.svg",
        ),
        (
            "WorkbenchButtonDelete",
            "zircon_editor_shell/controls/delete.svg",
        ),
        (
            "WorkbenchMiniFolder",
            "zircon_editor_shell/toolbar/folder-open.svg",
        ),
        ("WorkbenchMiniSave", "zircon_editor_shell/toolbar/save.svg"),
        ("WorkbenchMiniEye", "zircon_editor_shell/scene/eye.svg"),
        (
            "WorkbenchMiniEyeOff",
            "zircon_editor_shell/scene/eye-off.svg",
        ),
        ("WorkbenchMiniLock", "zircon_editor_shell/scene/lock.svg"),
        (
            "WorkbenchMiniMore",
            "zircon_editor_shell/toolbar/more-vertical.svg",
        ),
        (
            "WorkbenchStatusSnapToggle",
            "zircon_editor_shell/viewport/magnet.svg",
        ),
        (
            "WorkbenchStatusWorld",
            "zircon_editor_shell/viewport/globe.svg",
        ),
        (
            "WorkbenchStatusTarget",
            "zircon_editor_shell/viewport/crosshair.svg",
        ),
    ] {
        let node = workbench_window_node(&bridge, control_id);
        assert_eq!(
            node.icon_name.as_str(),
            expected_icon,
            "{control_id} should use an explicit shared shell icon asset"
        );
    }
}

#[test]
fn full_workbench_run_mode_uses_toolbar_dropdown_icon() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        FULL_WORKBENCH_WIDTH as f32,
        FULL_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    assert!(
        bridge.control_frame("WorkbenchRunMode").is_some(),
        "full toolbar should expose the run mode trigger"
    );
    let run_mode = workbench_window_node(&bridge, "WorkbenchRunMode");
    assert_eq!(
        run_mode.icon_name.as_str(),
        "zircon_editor_shell/toolbar/dropdown.svg",
        "run mode should use a toolbar dropdown glyph, not a tab/file overflow placeholder"
    );
    let run_group = bridge
        .control_frame("WorkbenchToolbarRunGroup")
        .expect("full toolbar should expose the complete run group");
    assert_frame_value("full run group width", run_group.width, 70.0);
    let layout_group = bridge
        .control_frame("WorkbenchToolbarLayoutGroup")
        .expect("full toolbar should expose the low-priority layout group");
    assert_frame_value("full layout group width", layout_group.width, 68.0);
    for control_id in ["WorkbenchLayoutGrid", "WorkbenchThemeToggle"] {
        assert_eq!(
            control_visibility(&bridge, control_id),
            Some(UiVisibility::Visible),
            "full toolbar should restore {control_id}"
        );
    }
}
