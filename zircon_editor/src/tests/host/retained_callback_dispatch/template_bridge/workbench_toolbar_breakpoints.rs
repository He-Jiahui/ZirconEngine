use std::path::PathBuf;

use super::super::support::*;
use super::support::{control_bool, control_float, control_visibility};
use crate::ui::retained_host::{
    paint_runtime_render_commands_for_test, to_host_contract_workbench_window_nodes,
    HostInvalidationMask, TemplatePaneMenuItemData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::tree::UiVisibility;

const COMPACT_WORKBENCH_WIDTH: u32 = 900;
const COMPACT_WORKBENCH_HEIGHT: u32 = 620;
const NARROW_WORKBENCH_WIDTH: u32 = 640;
const NARROW_WORKBENCH_HEIGHT: u32 = 520;
const FULL_WORKBENCH_WIDTH: u32 = 1672;
const FULL_WORKBENCH_HEIGHT: u32 = 941;
const MODULE_OVERFLOW_SCREENSHOT: &str = "editor-window-m3-workbench-module-overflow-900x620.png";
const MVP_RUN_CONTROLS_SCREENSHOT: &str =
    "editor-window-m3-workbench-mvp-run-controls-640x520.png";
const RUN_MODE_SCREENSHOT: &str = "editor-window-m3-workbench-run-mode-1672x941.png";

const COMPACT_CORE_MODULE_TABS: &[&str] = &[
    "WorkbenchModuleScene",
    "WorkbenchModuleEffect",
    "WorkbenchModuleAbility",
    "WorkbenchModuleTags",
    "WorkbenchModulePerception",
    "WorkbenchModuleMaterial",
];

#[test]
fn compact_workbench_toolbar_separates_command_and_module_tab_rows() {
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

    let toolbar = bridge
        .control_frame("WorkbenchWindowTopToolbarRegion")
        .expect("compact workbench should expose the top toolbar region");
    let file_group = bridge
        .control_frame("WorkbenchToolbarFileGroup")
        .expect("compact toolbar should expose the file group");
    let command_group = bridge
        .control_frame("WorkbenchModuleCommands")
        .expect("compact toolbar should expose module commands");
    let module_tabs = bridge
        .control_frame("WorkbenchModuleTabs")
        .expect("compact toolbar should expose module tabs");

    assert_frame_value("split toolbar height", toolbar.height, 72.0);
    assert_frame_value(
        "module commands share command row",
        command_group.y,
        file_group.y,
    );
    assert!(
        module_tabs.y >= file_group.y + file_group.height,
        "module tabs should live below the command row, got tab y {} and file row bottom {}",
        module_tabs.y,
        file_group.y + file_group.height
    );
    assert!(
        module_tabs.bottom() <= toolbar.bottom(),
        "module tabs should remain inside the toolbar region"
    );
}

#[test]
fn compact_workbench_toolbar_keeps_core_module_tabs_readable_and_collapses_overflow() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    for control_id in COMPACT_CORE_MODULE_TABS {
        let Some(frame) = bridge.control_frame(control_id) else {
            panic!("{control_id} should remain visible in compact toolbar");
        };
        assert!(
            frame.width >= 58.0,
            "{control_id} should keep readable width, got {}",
            frame.width
        );
    }

    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleBehavior"),
        Some(UiVisibility::Collapsed)
    );
    assert!(
        bridge.control_frame("WorkbenchModuleMore").is_some(),
        "compact toolbar should expose a module overflow affordance"
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchToolbarToolGroup"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleDiff"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn mvp_run_controls_remain_reachable_across_narrow_regular_and_wide_layouts() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    for (width, height) in [(640.0, 520.0), (900.0, 620.0), (1260.0, 780.0)] {
        let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, height))
            .unwrap_or_else(|error| panic!("{width}px workbench bridge should build: {error:?}"));
        let command_row = bridge
            .control_frame("WorkbenchToolbarCommandRow")
            .unwrap_or_else(|| panic!("{width}px toolbar should expose its command row"));
        let module_commands = bridge
            .control_frame("WorkbenchModuleCommands")
            .unwrap_or_else(|| panic!("{width}px toolbar should expose primary module commands"));
        let run_group = bridge
            .control_frame("WorkbenchToolbarRunGroup")
            .unwrap_or_else(|| panic!("{width}px toolbar should keep the MVP run group reachable"));
        let play = bridge
            .control_frame("WorkbenchRunPlay")
            .unwrap_or_else(|| panic!("{width}px toolbar should keep Play reachable"));
        let run_mode = bridge
            .control_frame("WorkbenchRunMode")
            .unwrap_or_else(|| panic!("{width}px toolbar should keep Run Mode reachable"));

        assert_frame_value("compact MVP run group width", run_group.width, 70.0);
        assert_frame_value("Play to Run Mode gap", run_mode.x - play.right(), 4.0);
        assert!(
            run_group.x >= module_commands.right(),
            "{width}px MVP run controls should follow primary commands without overlap"
        );
        assert!(
            run_group.right() <= command_row.right(),
            "{width}px MVP run controls should remain inside the command row"
        );
        assert_eq!(
            control_visibility(&bridge, "WorkbenchLayoutGrid"),
            Some(UiVisibility::Collapsed),
            "{width}px toolbar should defer the secondary layout control"
        );
        assert_eq!(
            control_visibility(&bridge, "WorkbenchThemeToggle"),
            Some(UiVisibility::Collapsed),
            "{width}px toolbar should defer the secondary theme control"
        );
    }
}

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
}

#[test]
fn full_workbench_secondary_module_commands_keep_readable_width() {
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

    let command_group = bridge
        .control_frame("WorkbenchModuleCommands")
        .expect("full toolbar should expose module commands");
    let diff = bridge
        .control_frame("WorkbenchModuleDiff")
        .expect("full toolbar should expose Diff");
    let simulate = bridge
        .control_frame("WorkbenchModuleSimulate")
        .expect("full toolbar should expose Simulate");

    assert_frame_value(
        "full module command group width",
        command_group.width,
        388.0,
    );
    assert!(
        diff.width >= 54.0,
        "Diff should keep one-line body text width, got {}",
        diff.width
    );
    assert!(
        simulate.width >= 50.0,
        "Sim should keep one-line body text width, got {}",
        simulate.width
    );
    assert_frame_value("Diff to Sim gap", simulate.x - diff.right(), 4.0);
}

#[test]
fn compact_workbench_toolbar_uses_slate_command_density() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let mut bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        COMPACT_WORKBENCH_WIDTH as f32,
        COMPACT_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    let toolbar_frame = bridge
        .control_frame("WorkbenchWindowTopToolbarRegion")
        .expect("compact workbench should expose the top toolbar region");
    assert_frame_value("toolbar height", toolbar_frame.height, 72.0);

    for control_id in COMPACT_CORE_MODULE_TABS {
        let frame = bridge
            .control_frame(control_id)
            .unwrap_or_else(|| panic!("{control_id} should remain visible"));
        assert_frame_value(&format!("{control_id} height"), frame.height, 34.0);
    }

    let module_more = bridge
        .control_frame("WorkbenchModuleMore")
        .expect("compact toolbar should expose module overflow");
    assert_frame_value("module more width", module_more.width, 34.0);
    assert_frame_value("module more height", module_more.height, 30.0);

    let module_commands = bridge
        .control_frame("WorkbenchModuleCommands")
        .expect("compact toolbar should expose primary module commands");
    assert_frame_value("module command group width", module_commands.width, 276.0);
    assert_frame_value("module command group height", module_commands.height, 36.0);

    let save = bridge
        .control_frame("WorkbenchModuleSave")
        .expect("save command should remain visible");
    let browse = bridge
        .control_frame("WorkbenchModuleBrowse")
        .expect("browse command should remain visible");
    let compile = bridge
        .control_frame("WorkbenchModuleCompile")
        .expect("compile command should remain visible");
    assert_frame_value("save command width", save.width, 72.0);
    assert_frame_value("save command height", save.height, 30.0);
    assert_frame_value("browse command width", browse.width, 92.0);
    assert_frame_value("browse command height", browse.height, 30.0);
    assert_frame_value("compile command width", compile.width, 104.0);
    assert_frame_value("compile command height", compile.height, 30.0);
    assert_frame_value("save to browse gap", browse.x - (save.x + save.width), 4.0);
    assert_frame_value(
        "browse to compile gap",
        compile.x - (browse.x + browse.width),
        4.0,
    );

    bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("module overflow should dispatch")
        .expect("module overflow should expose a binding");
    let menu = bridge
        .control_frame("WorkbenchModuleOverflowMenu")
        .expect("module overflow menu should open below the compact toolbar");
    assert_frame_value("module overflow menu y", menu.y, toolbar_frame.bottom());
    assert_frame_value(
        "module overflow menu x follows more button",
        menu.x,
        clamped_toolbar_menu_x(module_more.x, menu.width, COMPACT_WORKBENCH_WIDTH as f32),
    );
    assert_frame_value(
        "module overflow popup anchor x",
        control_float(&bridge, "WorkbenchModuleOverflowMenu", "popup_anchor_x")
            .expect("module overflow popup should store anchor x") as f32,
        menu.x,
    );
    assert_frame_value(
        "module overflow popup anchor y",
        control_float(&bridge, "WorkbenchModuleOverflowMenu", "popup_anchor_y")
            .expect("module overflow popup should store anchor y") as f32,
        menu.y,
    );
}

#[test]
fn compact_workbench_module_more_opens_overflow_menu_and_selects_hidden_module() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let harness = EventRuntimeHarness::new("zircon_workbench_module_overflow_menu");
    let mut bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        COMPACT_WORKBENCH_WIDTH as f32,
        COMPACT_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleOverflowMenu"),
        Some(UiVisibility::Collapsed)
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("module overflow should dispatch")
        .expect("module overflow should expose a binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id } if action_id == "workbench.module.more.open"
    ));
    assert!(control_bool(&bridge, "WorkbenchModuleMore", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleMore", "checked"));
    assert!(control_bool(
        &bridge,
        "WorkbenchModuleOverflowMenu",
        "popup_open"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleOverflowMenu"),
        Some(UiVisibility::Visible)
    );

    let opened_menu = workbench_window_node(&bridge, "WorkbenchModuleOverflowMenu");
    assert!(opened_menu.popup_open);
    assert_eq!(opened_menu.structured_menu_items.row_count(), 5);
    assert_eq!(
        structured_menu_item(&opened_menu, 0).label.as_str(),
        "Behavior"
    );
    assert_eq!(
        structured_menu_item(&opened_menu, 0).action_id.as_str(),
        "menu.item.behavior"
    );
    assert_eq!(
        structured_menu_item(&opened_menu, 1).label.as_str(),
        "Render"
    );
    assert_eq!(
        structured_menu_item(&opened_menu, 2).label.as_str(),
        "Assets"
    );
    assert_eq!(structured_menu_item(&opened_menu, 3).label.as_str(), "VFX");
    assert_eq!(structured_menu_item(&opened_menu, 4).label.as_str(), "HUD");

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchModuleOverflowMenu",
        "menu.item.behavior",
    )
    .expect("module overflow menu item should be handled")
    .expect("module overflow menu item should dispatch");

    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!control_bool(&bridge, "WorkbenchModuleMore", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleMore", "checked"));
    assert!(!control_bool(
        &bridge,
        "WorkbenchModuleOverflowMenu",
        "popup_open"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleOverflowMenu"),
        Some(UiVisibility::Collapsed)
    );
    assert!(control_bool(&bridge, "WorkbenchModuleBehavior", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleBehavior", "checked"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleBehaviorWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleEffectWorkspace"),
        Some(UiVisibility::Collapsed)
    );

    bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("module overflow should reopen")
        .expect("module overflow should expose a binding");
    let reopened_menu = workbench_window_node(&bridge, "WorkbenchModuleOverflowMenu");
    assert!(
        structured_menu_item(&reopened_menu, 0).checked,
        "reopened overflow menu should reflect the active hidden module"
    );
}

#[test]
#[ignore = "writes a visual artifact under docs/tests/editor"]
fn capture_workbench_module_overflow_visual_artifact() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        COMPACT_WORKBENCH_WIDTH as f32,
        COMPACT_WORKBENCH_HEIGHT as f32,
    ))
    .expect("workbench bridge should build");
    for control_id in [
        "WorkbenchPopupMenu",
        "WorkbenchDropdownButton",
        "WorkbenchInputDropdown",
    ] {
        bridge
            .close_popup(control_id)
            .expect("component lab popup samples should close for focused screenshot capture");
    }
    bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("module overflow should dispatch")
        .expect("module overflow should expose a binding");
    let menu_frame = bridge
        .control_frame("WorkbenchModuleOverflowMenu")
        .expect("opened module overflow menu should have a frame");
    let bytes = paint_runtime_render_commands_for_test(
        COMPACT_WORKBENCH_WIDTH,
        COMPACT_WORKBENCH_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    assert!(
        first_non_black_pixel_in_frame(
            &bytes,
            COMPACT_WORKBENCH_WIDTH,
            COMPACT_WORKBENCH_HEIGHT,
            menu_frame
        )
        .is_some(),
        "opened module overflow menu should paint visible pixels"
    );

    let path = module_overflow_screenshot_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("module overflow screenshot directory should exist");
    }
    image::save_buffer_with_format(
        &path,
        &bytes,
        COMPACT_WORKBENCH_WIDTH,
        COMPACT_WORKBENCH_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("module overflow screenshot should be written");
    println!("wrote {}", path.display());
}

#[test]
#[ignore = "writes a visual artifact under docs/tests/editor"]
fn capture_narrow_workbench_mvp_run_controls_visual_artifact() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        NARROW_WORKBENCH_WIDTH as f32,
        NARROW_WORKBENCH_HEIGHT as f32,
    ))
    .expect("narrow workbench bridge should build");
    let run_group = bridge
        .control_frame("WorkbenchToolbarRunGroup")
        .expect("narrow toolbar should keep the MVP run group reachable");
    let play = bridge
        .control_frame("WorkbenchRunPlay")
        .expect("narrow toolbar should keep Play reachable");
    let run_mode = bridge
        .control_frame("WorkbenchRunMode")
        .expect("narrow toolbar should keep Run Mode reachable");
    assert_frame_value("narrow MVP run group width", run_group.width, 70.0);
    assert_frame_value("narrow Play to Run Mode gap", run_mode.x - play.right(), 4.0);

    let bytes = paint_runtime_render_commands_for_test(
        NARROW_WORKBENCH_WIDTH,
        NARROW_WORKBENCH_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    for (label, frame) in [("Play", play), ("Run Mode", run_mode)] {
        assert!(
            first_non_black_pixel_in_frame(
                &bytes,
                NARROW_WORKBENCH_WIDTH,
                NARROW_WORKBENCH_HEIGHT,
                frame,
            )
            .is_some(),
            "narrow {label} control should paint visible pixels"
        );
    }

    let path = mvp_run_controls_screenshot_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("MVP run controls screenshot directory should exist");
    }
    image::save_buffer_with_format(
        &path,
        &bytes,
        NARROW_WORKBENCH_WIDTH,
        NARROW_WORKBENCH_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("MVP run controls screenshot should be written");
    println!("wrote {}", path.display());
}

#[test]
#[ignore = "writes a visual artifact under docs/tests/editor"]
fn capture_full_workbench_run_mode_visual_artifact() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        FULL_WORKBENCH_WIDTH as f32,
        FULL_WORKBENCH_HEIGHT as f32,
    ))
    .expect("workbench bridge should build");
    let run_mode_frame = bridge
        .control_frame("WorkbenchRunMode")
        .expect("full toolbar should expose the run mode trigger");
    let run_mode = workbench_window_node(&bridge, "WorkbenchRunMode");
    assert_eq!(
        run_mode.icon_name.as_str(),
        "zircon_editor_shell/toolbar/dropdown.svg",
        "run mode should use a toolbar dropdown glyph for the visual artifact"
    );

    let bytes = paint_runtime_render_commands_for_test(
        FULL_WORKBENCH_WIDTH,
        FULL_WORKBENCH_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    assert!(
        first_non_black_pixel_in_frame(
            &bytes,
            FULL_WORKBENCH_WIDTH,
            FULL_WORKBENCH_HEIGHT,
            run_mode_frame
        )
        .is_some(),
        "visible run mode trigger should paint pixels in the full-width artifact"
    );

    let path = run_mode_screenshot_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("run mode screenshot directory should exist");
    }
    image::save_buffer_with_format(
        &path,
        &bytes,
        FULL_WORKBENCH_WIDTH,
        FULL_WORKBENCH_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("run mode screenshot should be written");
    println!("wrote {}", path.display());
}

fn workbench_window_node(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> TemplatePaneNodeData {
    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to native host nodes"))
}

fn structured_menu_item(node: &TemplatePaneNodeData, row: usize) -> TemplatePaneMenuItemData {
    node.structured_menu_items
        .row_data(row)
        .unwrap_or_else(|| panic!("structured menu item row {row} should exist"))
}

fn module_overflow_screenshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
        .join(MODULE_OVERFLOW_SCREENSHOT)
}

fn mvp_run_controls_screenshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
        .join(MVP_RUN_CONTROLS_SCREENSHOT)
}

fn run_mode_screenshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
        .join(RUN_MODE_SCREENSHOT)
}

fn first_non_black_pixel_in_frame(
    bytes: &[u8],
    width: u32,
    height: u32,
    frame: UiFrame,
) -> Option<[u8; 4]> {
    let start_x = frame.x.floor().max(0.0) as u32;
    let start_y = frame.y.floor().max(0.0) as u32;
    let end_x = (frame.x + frame.width).ceil().min(width as f32) as u32;
    let end_y = (frame.y + frame.height).ceil().min(height as f32) as u32;

    (start_y..end_y)
        .flat_map(|y| (start_x..end_x).map(move |x| (x, y)))
        .map(|(x, y)| pixel(bytes, width, x, y))
        .find(|pixel| *pixel != [0, 0, 0, 255])
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn assert_frame_value(label: &str, actual: f32, expected: f32) {
    const EPSILON: f32 = 0.01;
    assert!(
        (actual - expected).abs() <= EPSILON,
        "{label} should be {expected}, got {actual}"
    );
}

fn clamped_toolbar_menu_x(authored_x: f32, menu_width: f32, root_width: f32) -> f32 {
    const EDGE_MARGIN: f32 = 8.0;
    let min_x = EDGE_MARGIN.min(root_width * 0.5);
    let max_x = root_width - min_x - menu_width;
    if max_x >= min_x {
        authored_x.clamp(min_x, max_x)
    } else {
        0.0_f32.max(root_width - menu_width)
    }
}
