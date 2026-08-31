mod business_controls;
mod button_semantics;
mod group_separators;
mod logical_scale;
mod run_controls;
mod vertical_alignment;
mod visual_artifacts;

use std::path::PathBuf;

use super::super::support::*;
use super::support::{control_bool, control_float, control_string, control_visibility};
use crate::ui::binding::AssetCommand;
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
const MVP_RUN_CONTROLS_SCREENSHOT: &str = "editor-window-m3-workbench-mvp-run-controls-640x520.png";
const RUN_MODE_SCREENSHOT: &str = "editor-window-m3-workbench-run-mode-1672x941.png";

const COMPACT_CORE_MODULE_TABS: &[&str] = &[
    "WorkbenchModuleScene",
    "WorkbenchModuleEffect",
    "WorkbenchModuleAbility",
    "WorkbenchModuleTags",
    "WorkbenchModulePerception",
    "WorkbenchModuleMaterial",
];

const ULTRA_CORE_MODULE_TABS: &[&str] = &[
    "WorkbenchModuleScene",
    "WorkbenchModuleEffect",
    "WorkbenchModuleAbility",
    "WorkbenchModuleTags",
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

    assert_frame_value("split toolbar height", toolbar.height, 66.0);
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
fn ultra_workbench_toolbar_uses_icon_density_and_keeps_rows_inside_the_shell() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 360.0))
        .unwrap_or_else(|error| panic!("ultra workbench bridge should build: {error:?}"));
    let command_row = bridge
        .control_frame("WorkbenchToolbarCommandRow")
        .expect("ultra toolbar should expose its command row");
    let module_tabs = bridge
        .control_frame("WorkbenchModuleTabs")
        .expect("ultra toolbar should expose its module row");

    for control_id in [
        "WorkbenchToolbarAssets",
        "WorkbenchToolbarOpen",
        "WorkbenchToolbarSave",
        "WorkbenchModulePerception",
        "WorkbenchModuleMaterial",
    ] {
        assert_eq!(
            control_visibility(&bridge, control_id),
            Some(UiVisibility::Collapsed),
            "ultra toolbar should collapse {control_id} into an existing reachable menu"
        );
    }
    for control_id in ULTRA_CORE_MODULE_TABS {
        assert!(
            bridge.control_frame(control_id).is_some(),
            "ultra toolbar should keep {control_id} directly reachable"
        );
    }
    for (control_id, label) in [
        ("WorkbenchModuleSave", "Save"),
        ("WorkbenchModuleBrowse", "Browse"),
        ("WorkbenchModuleCompile", "Compile"),
    ] {
        let frame = bridge
            .control_frame(control_id)
            .unwrap_or_else(|| panic!("ultra toolbar should keep {control_id} reachable"));
        assert_frame_value("ultra icon command width", frame.width, 34.0);
        assert_eq!(
            control_string(&bridge, control_id, "text").as_deref(),
            Some("")
        );
        assert_eq!(
            control_string(&bridge, control_id, "label").as_deref(),
            Some(label)
        );
        assert_eq!(
            control_string(&bridge, control_id, "icon_placement").as_deref(),
            Some("icon_only")
        );
    }

    assert!(
        module_tabs.right() <= 420.0,
        "ultra module row should remain inside the 420px shell"
    );
    for control_id in [
        "WorkbenchToolbarMenu",
        "WorkbenchModuleCompile",
        "WorkbenchRunPlay",
        "WorkbenchRunMode",
        "WorkbenchLayoutGrid",
        "WorkbenchThemeToggle",
    ] {
        let frame = bridge
            .control_frame(control_id)
            .unwrap_or_else(|| panic!("ultra toolbar should expose {control_id}"));
        assert!(
            frame.x >= command_row.x && frame.right() <= command_row.right(),
            "{control_id} should remain inside the ultra command row"
        );
    }
}

#[test]
fn ultra_toolbar_density_restores_after_resize() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 360.0))
        .unwrap_or_else(|error| panic!("ultra workbench bridge should build: {error:?}"));
    let regular = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1260.0, 720.0))
        .unwrap_or_else(|error| panic!("regular workbench bridge should build: {error:?}"));
    bridge
        .recompute_layout(UiSize::new(1260.0, 720.0))
        .expect("resized workbench should recompute");

    for (control_id, label, width, icon_only) in [
        ("WorkbenchModuleSave", "Save", 34.0, true),
        ("WorkbenchModuleBrowse", "Browse", 34.0, true),
        ("WorkbenchModuleCompile", "Compile", 104.0, false),
    ] {
        assert_eq!(
            control_string(&bridge, control_id, "text").as_deref(),
            Some(if icon_only { "" } else { label })
        );
        assert_eq!(
            control_string(&bridge, control_id, "icon_placement").as_deref(),
            Some(if icon_only { "icon_only" } else { "leading" })
        );
        assert_frame_value(
            "restored module command width",
            bridge
                .control_frame(control_id)
                .unwrap_or_else(|| panic!("resized toolbar should expose {control_id}"))
                .width,
            width,
        );
    }
    for control_id in [
        "WorkbenchToolbarAssets",
        "WorkbenchToolbarOpen",
        "WorkbenchToolbarSave",
    ] {
        assert!(
            bridge.control_frame(control_id).is_some(),
            "resized toolbar should restore {control_id}"
        );
    }
    for control_id in [
        "WorkbenchModuleDiff",
        "WorkbenchModuleSimulate",
        "WorkbenchToolbarToolGroup",
        "WorkbenchModuleMore",
    ] {
        assert_eq!(
            control_visibility(&bridge, control_id),
            control_visibility(&regular, control_id),
            "resizing out of Ultra should match a fresh regular projection for {control_id}"
        );
    }
}

#[test]
fn ultra_toolbar_main_menu_keeps_hidden_file_commands_reachable() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 360.0))
        .unwrap_or_else(|error| panic!("ultra workbench bridge should build: {error:?}"));

    bridge
        .dispatch_control_state("WorkbenchToolbarMenu", UiEventKind::Click)
        .expect("ultra toolbar main menu should dispatch")
        .expect("ultra toolbar main menu should expose a binding");
    assert!(control_bool(
        &bridge,
        "WorkbenchToolbarMainMenu",
        "popup_open"
    ));

    let asset_browser = bridge
        .main_menu_item_binding("WorkbenchToolbarMainMenu", "menu.item.asset_browser")
        .expect("the main menu should retain the hidden Asset Browser command");
    assert_eq!(
        asset_browser.payload(),
        &EditorUiBindingPayload::asset_command(AssetCommand::OpenAssetBrowser)
    );

    for (action_id, expected_command) in [
        ("menu.item.open_project", "file.project.open"),
        ("menu.item.save_project", "file.project.save"),
    ] {
        let binding = bridge
            .main_menu_item_binding("WorkbenchToolbarMainMenu", action_id)
            .unwrap_or_else(|| panic!("the main menu should retain {expected_command}"));
        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::EditorCommand { command_id }
                if command_id == expected_command
        ));
    }
}

#[test]
fn mvp_run_and_layout_controls_remain_reachable_across_all_layout_tiers() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    for (width, height, tools_visible) in [
        (420.0, 360.0, false),
        (640.0, 520.0, false),
        (900.0, 620.0, false),
        (1260.0, 780.0, true),
    ] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, height))
                .unwrap_or_else(|error| {
                    panic!("{width}px workbench bridge should build: {error:?}")
                });
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
        let layout_group = bridge
            .control_frame("WorkbenchToolbarLayoutGroup")
            .unwrap_or_else(|| panic!("{width}px toolbar should keep Layout and Theme reachable"));

        assert_frame_value("compact MVP run group width", run_group.width, 70.0);
        assert_frame_value("compact layout group width", layout_group.width, 68.0);
        assert_frame_value("Play to Run Mode gap", run_mode.x - play.right(), 4.0);
        assert!(
            run_group.x >= module_commands.right(),
            "{width}px MVP run controls should follow primary commands without overlap"
        );
        assert!(
            run_group.right() <= command_row.right(),
            "{width}px MVP run controls should remain inside the command row"
        );
        for control_id in ["WorkbenchLayoutGrid", "WorkbenchThemeToggle"] {
            assert_eq!(
                control_visibility(&bridge, control_id),
                Some(UiVisibility::Visible),
                "{width}px toolbar should keep the iconized {control_id} command reachable"
            );
        }
        assert!(
            layout_group.x >= run_group.right(),
            "{width}px layout commands should follow Run without overlap"
        );
        assert!(
            layout_group.right() <= command_row.right(),
            "{width}px layout commands should remain inside the command row"
        );
        assert_eq!(
            control_visibility(&bridge, "WorkbenchToolbarToolGroup"),
            Some(if tools_visible {
                UiVisibility::Visible
            } else {
                UiVisibility::Collapsed
            }),
            "{width}px toolbar should project the transform-tool group by business priority"
        );
        if tools_visible {
            let tools = bridge
                .control_frame("WorkbenchToolbarToolGroup")
                .unwrap_or_else(|| panic!("{width}px toolbar should expose transform tools"));
            assert!(
                tools.x >= module_commands.right() && tools.right() <= run_group.x,
                "{width}px transform tools should fit between module and run groups"
            );
        }

        let play_binding = bridge
            .dispatch_control_state("WorkbenchRunPlay", UiEventKind::Click)
            .unwrap_or_else(|error| panic!("{width}px Play should dispatch: {error:?}"))
            .unwrap_or_else(|| panic!("{width}px Play should expose a binding"));
        assert!(matches!(
            play_binding.payload(),
            EditorUiBindingPayload::EditorCommand { command_id }
                if command_id == "runtime.play_mode.enter"
        ));
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
        292.0,
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
    assert_frame_value("toolbar height", toolbar_frame.height, 66.0);

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
    assert_frame_value("module command group width", module_commands.width, 180.0);
    assert_frame_value("module command group height", module_commands.height, 34.0);

    let save = bridge
        .control_frame("WorkbenchModuleSave")
        .expect("save command should remain visible");
    let browse = bridge
        .control_frame("WorkbenchModuleBrowse")
        .expect("browse command should remain visible");
    let compile = bridge
        .control_frame("WorkbenchModuleCompile")
        .expect("compile command should remain visible");
    assert_frame_value("save command width", save.width, 34.0);
    assert_frame_value("save command height", save.height, 30.0);
    assert_frame_value("browse command width", browse.width, 34.0);
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
    let menu = rendered_control_frame(&bridge, "WorkbenchModuleOverflowMenu");
    let arranged_menu = bridge
        .control_frame("WorkbenchModuleOverflowMenu")
        .expect("module overflow menu should open below the compact toolbar");
    assert_frame_value("module overflow menu y", menu.y, toolbar_frame.bottom());
    assert_frame_value(
        "module overflow menu x follows more button",
        menu.x,
        module_more.x,
    );
    assert_frame_value(
        "module overflow rendered width",
        menu.width,
        arranged_menu.width,
    );
    assert_eq!(
        control_float(&bridge, "WorkbenchModuleOverflowMenu", "popup_anchor_x"),
        None
    );
    assert_eq!(
        control_float(&bridge, "WorkbenchModuleOverflowMenu", "popup_anchor_y"),
        None
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
    assert_eq!(opened_menu.structured_menu_items.row_count(), 7);
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
    assert_eq!(structured_menu_item(&opened_menu, 5).label.as_str(), "Diff");
    assert_eq!(structured_menu_item(&opened_menu, 6).label.as_str(), "Sim");

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

#[test]
fn ultra_module_overflow_exposes_every_hidden_module_and_secondary_command() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let harness = EventRuntimeHarness::new("zircon_workbench_ultra_module_overflow_menu");
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 360.0))
        .unwrap_or_else(|error| panic!("ultra workbench bridge should build: {error:?}"));
    bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("ultra module overflow should dispatch")
        .expect("ultra module overflow should expose a binding");

    let opened_menu = workbench_window_node(&bridge, "WorkbenchModuleOverflowMenu");
    assert_eq!(opened_menu.structured_menu_items.row_count(), 9);
    let labels = (0..9)
        .map(|row| structured_menu_item(&opened_menu, row).label.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        labels.iter().map(String::as_str).collect::<Vec<_>>(),
        vec![
            "Perception",
            "Material",
            "Behavior",
            "Render",
            "Assets",
            "VFX",
            "HUD",
            "Diff",
            "Sim",
        ]
    );
    assert!(
        opened_menu.frame.y + opened_menu.frame.height <= 360.0,
        "ultra overflow menu should remain inside the 360px shell"
    );

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchModuleOverflowMenu",
        "menu.item.diff",
    )
    .expect("hidden Diff menu item should be handled")
    .expect("hidden Diff menu item should dispatch through the source binding");
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
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

fn rendered_control_frame(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> UiFrame {
    let node_id = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                .filter(|candidate| *candidate == control_id)
                .map(|_| node.node_id)
        })
        .unwrap_or_else(|| panic!("{control_id} should resolve to one runtime node"));
    bridge
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| command.node_id == node_id)
        .map(|command| command.frame)
        .max_by(|left, right| {
            let left_area = left.width.max(0.0) * left.height.max(0.0);
            let right_area = right.width.max(0.0) * right.height.max(0.0);
            left_area.total_cmp(&right_area)
        })
        .unwrap_or_else(|| panic!("{control_id} should emit popup render commands"))
}
