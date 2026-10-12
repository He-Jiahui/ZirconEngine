use super::super::super::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, EditorUiBindingPayload, UiEventKind,
    UiSize,
};
use super::super::support::{control_bool, control_visibility};
use zircon_runtime_interface::ui::tree::UiVisibility;

const MODULE_SWITCH_CASES: &[(&str, &str, &str)] = &[
    (
        "WorkbenchModuleEffect",
        "workbench.module.effect.select",
        "WorkbenchModuleEffectWorkspace",
    ),
    (
        "WorkbenchModuleAbility",
        "workbench.module.ability.select",
        "WorkbenchModuleAbilityWorkspace",
    ),
    (
        "WorkbenchModuleTags",
        "workbench.module.tags.select",
        "WorkbenchModuleTagsWorkspace",
    ),
    (
        "WorkbenchModulePerception",
        "workbench.module.perception.select",
        "WorkbenchModulePerceptionWorkspace",
    ),
    (
        "WorkbenchModuleMaterial",
        "workbench.module.material.select",
        "WorkbenchModuleMaterialWorkspace",
    ),
    (
        "WorkbenchModuleBehavior",
        "workbench.module.behavior.select",
        "WorkbenchModuleBehaviorWorkspace",
    ),
    (
        "WorkbenchModuleRender",
        "workbench.module.render.select",
        "WorkbenchModuleRenderWorkspace",
    ),
    (
        "WorkbenchModuleAssets",
        "workbench.module.assets.select",
        "WorkbenchModuleAssetsWorkspace",
    ),
    (
        "WorkbenchModuleVfx",
        "workbench.module.vfx.select",
        "WorkbenchModuleVfxWorkspace",
    ),
    (
        "WorkbenchModuleHud",
        "workbench.module.hud.select",
        "WorkbenchModuleHudWorkspace",
    ),
];

#[test]
fn workbench_module_tabs_switch_exactly_one_module_workspace() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
        .unwrap()
        .expect("material module tab should expose a preview binding");

    for &(tab_control_id, expected_action, expected_workspace_id) in MODULE_SWITCH_CASES {
        assert!(matches!(
            bridge
                .dispatch_control_state(tab_control_id, UiEventKind::Click)
                .unwrap()
                .expect("module tab should expose a preview binding")
                .payload(),
            EditorUiBindingPayload::MenuAction { action_id } if action_id == expected_action
        ));

        assert!(control_bool(&bridge, tab_control_id, "selected"));
        assert!(control_bool(&bridge, tab_control_id, "checked"));
        assert_eq!(
            control_visibility(&bridge, expected_workspace_id),
            Some(UiVisibility::Visible)
        );
        assert_eq!(
            bridge.control_frame(expected_workspace_id).is_some(),
            true,
            "{expected_workspace_id} should be part of the current projection"
        );
        assert_eq!(
            bridge.control_frame("WorkbenchSceneWorkspace").is_some(),
            true,
            "module tabs should keep the scene shell lane projected for the activity rail"
        );

        for &(_, _, workspace_id) in MODULE_SWITCH_CASES {
            let expected_visibility = if workspace_id == expected_workspace_id {
                Some(UiVisibility::Visible)
            } else {
                Some(UiVisibility::Collapsed)
            };
            assert_eq!(
                control_visibility(&bridge, workspace_id),
                expected_visibility,
                "{workspace_id} visibility after selecting {tab_control_id}"
            );
            assert_eq!(
                bridge.control_frame(workspace_id).is_some(),
                workspace_id == expected_workspace_id,
                "{workspace_id} projection frame after selecting {tab_control_id}"
            );
        }
    }
}

#[test]
fn workbench_scene_tab_restores_scene_workspace_and_hides_module_workspaces() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleRender", UiEventKind::Click)
        .unwrap()
        .expect("render module tab should expose a preview binding");

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleScene", UiEventKind::Click)
            .unwrap()
            .expect("scene module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.scene.select"
    ));
    assert!(control_bool(&bridge, "WorkbenchModuleScene", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        bridge.control_frame("WorkbenchSceneWorkspace").is_some(),
        true,
        "scene workspace should return to the projection"
    );
    for &(_, _, workspace_id) in MODULE_SWITCH_CASES {
        assert_eq!(
            control_visibility(&bridge, workspace_id),
            Some(UiVisibility::Collapsed),
            "{workspace_id} should stay hidden in scene mode"
        );
        assert_eq!(
            bridge.control_frame(workspace_id).is_some(),
            false,
            "{workspace_id} should not be projected in scene mode"
        );
    }
}

#[test]
fn row_selection_is_exclusive_within_its_layout_parent() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert!(control_bool(
        &bridge,
        "WorkbenchExtensionAnimationCompressionLocomotionRow",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchExtensionAnimationCompressionRunClipTableRow",
        "selected"
    ));

    bridge
        .dispatch_control_state(
            "WorkbenchExtensionAnimationCompressionAttackClipTableRow",
            UiEventKind::Click,
        )
        .unwrap()
        .expect("animation compression table row should expose a preview binding");

    assert!(control_bool(
        &bridge,
        "WorkbenchExtensionAnimationCompressionLocomotionRow",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchExtensionAnimationCompressionRunClipTableRow",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchExtensionAnimationCompressionAttackClipTableRow",
        "selected"
    ));
}
