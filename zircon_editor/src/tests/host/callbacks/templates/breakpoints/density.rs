use super::super::super::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, UiEventKind, UiSize,
};
use super::super::support::{control_float, control_string, control_visibility};
use super::support::{
    assert_frame_value, rendered_control_frame, COMPACT_WORKBENCH_HEIGHT, COMPACT_WORKBENCH_WIDTH,
};
use zircon_runtime_interface::ui::tree::UiVisibility;

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
