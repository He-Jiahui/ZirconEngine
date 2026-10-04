use super::super::super::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, EditorUiBindingPayload, UiEventKind,
    UiSize,
};
use super::super::support::{control_bool, control_string, control_visibility};
use zircon_runtime_interface::ui::tree::UiVisibility;

#[test]
fn workbench_module_commands_update_status_and_module_output_rows() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleAbility", UiEventKind::Click)
        .unwrap()
        .expect("ability module tab should expose a preview binding");

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchAbilityPlaytestButton", UiEventKind::Click)
            .unwrap()
            .expect("ability playtest button should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.ability.playtest.invoke"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchAbilityPlaytestButton",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchAbilityPlaytestButton",
        "checked"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Ability playtest queued")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusMessages", "text").as_deref(),
        Some("1 Message")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchAbilityOutputRow", "value_text").as_deref(),
        Some("Playtest queued   activation phase   GA_DashAttack")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchAbilityOutputRow")
            .expect("ability output row projection after command")
            .value_text
            .as_deref(),
        Some("Playtest queued   activation phase   GA_DashAttack")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleRender", UiEventKind::Click)
        .unwrap()
        .expect("render module tab should expose a preview binding");
    bridge
        .dispatch_control_state("WorkbenchRenderCompileButton", UiEventKind::Click)
        .unwrap()
        .expect("render compile button should expose a preview binding");

    assert!(!control_bool(
        &bridge,
        "WorkbenchRenderCompileButton",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchRenderCompileButton",
        "checked"
    ));

    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Render graph compiled")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRenderCaptureRow", "value_text").as_deref(),
        Some("Windows DX12   frame 1234   Frame Start compiled")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRenderCenterTitle", "text").as_deref(),
        Some("MainPipeline.rp / Lighting Pass")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleBrowse", UiEventKind::Click)
        .unwrap()
        .expect("browse command should expose a preview binding");

    assert!(control_bool(&bridge, "WorkbenchModuleAssets", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleAssetsWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Asset browser focused")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchAssetsOutputRow", "text").as_deref(),
        Some("Browse: focused Content/Environment/Forest")
    );
}

#[test]
fn workbench_extension_commands_are_momentary_but_keep_feedback() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleRender", UiEventKind::Click)
        .unwrap()
        .expect("render module tab should expose a preview binding");
    bridge
        .dispatch_control_state("WorkbenchRenderTools", UiEventKind::Click)
        .unwrap()
        .expect("render tools opener should expose a preview binding");
    bridge
        .dispatch_workbench_render_editor_menu_item_state(
            "WorkbenchRenderToolsMenu",
            "menu.item.render.shader_editor",
        )
        .unwrap()
        .expect("shader editor menu item should expose a preview binding");

    assert!(!control_bool(
        &bridge,
        "WorkbenchExtensionShaderCompileButton",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchExtensionShaderCompileButton",
        "checked"
    ));
    assert!(bridge
        .select_dropdown_option("WorkbenchExtensionShaderTargetDropdown", "spirv")
        .unwrap());
    assert_eq!(
        bridge
            .edit_workbench_module_field(
                "WorkbenchExtensionShaderEntryField",
                "WorkbenchExtension/ShaderEditorEntryEdit",
                "fs_custom",
            )
            .unwrap(),
        Some(true)
    );
    assert!(bridge
        .select_dropdown_option("WorkbenchExtensionShaderLiveCompileDropdown", "manual")
        .unwrap());

    assert!(matches!(
        bridge
            .dispatch_control_state(
                "WorkbenchExtensionShaderCompileButton",
                UiEventKind::Click,
            )
            .unwrap()
            .expect("shader compile button should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.extension.shader_editor.compile.invoke"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchExtensionShaderCompileButton",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchExtensionShaderCompileButton",
        "checked"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Shader compile queued")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchExtensionShaderOutputRow", "value_text").as_deref(),
        Some("Inputs: spirv | fs_custom | Manual")
    );
}

#[test]
fn workbench_shared_module_commands_route_feedback_to_active_module_output() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    bridge
        .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
        .unwrap()
        .expect("material module tab should expose a preview binding");
    bridge
        .dispatch_control_state("WorkbenchModuleCompile", UiEventKind::Click)
        .unwrap()
        .expect("shared compile command should expose a preview binding");
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Material compile queued")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialOutputRow", "text").as_deref(),
        Some("Shader Output: material compile queued")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleBehavior", UiEventKind::Click)
        .unwrap()
        .expect("behavior module tab should expose a preview binding");
    bridge
        .dispatch_control_state("WorkbenchModuleCompile", UiEventKind::Click)
        .unwrap()
        .expect("shared compile command should expose a preview binding");
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Behavior tree compile queued")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchBehaviorOutputRow", "text").as_deref(),
        Some("Runtime Trace: behavior tree compile queued")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleAssets", UiEventKind::Click)
        .unwrap()
        .expect("asset module tab should expose a preview binding");
    bridge
        .dispatch_control_state("WorkbenchModuleCompile", UiEventKind::Click)
        .unwrap()
        .expect("shared compile command should expose a preview binding");
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Asset cook queued")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchAssetsOutputRow", "text").as_deref(),
        Some("Cook: asset build graph queued")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleVfx", UiEventKind::Click)
        .unwrap()
        .expect("vfx module tab should expose a preview binding");
    bridge
        .dispatch_control_state("WorkbenchModuleCompile", UiEventKind::Click)
        .unwrap()
        .expect("shared compile command should expose a preview binding");
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("VFX compile queued")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchVfxOutputRow", "text").as_deref(),
        Some("Compile Output: E_Bolt compile queued")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleDiff", UiEventKind::Click)
        .unwrap()
        .expect("shared diff command should expose a preview binding");
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("VFX diff prepared")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchVfxOutputRow", "text").as_deref(),
        Some("Diff: emitter stack changes compared")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleSimulate", UiEventKind::Click)
        .unwrap()
        .expect("shared simulate command should expose a preview binding");
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("VFX simulation running")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchVfxOutputRow", "text").as_deref(),
        Some("Simulation: preview running at 60 fps")
    );
}
