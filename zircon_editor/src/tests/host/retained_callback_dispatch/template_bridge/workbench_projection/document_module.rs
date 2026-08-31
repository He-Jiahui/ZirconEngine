use super::*;

#[test]
fn componentized_workbench_window_template_bridge_exposes_document_tab_runtime_routes() {
    let _guard = env_lock().lock().unwrap();

    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    let document_tabs = bridge
        .host_projection()
        .node_by_control_id("DocumentTabsRoot")
        .expect("componentized Workbench should expose document tabs");
    assert_eq!(document_tabs.component, "DocumentTabs");
    assert!(document_tabs.routes.iter().any(|route| {
        route.binding_id == "DocumentTabs/ActivateTab"
            && route.event_kind == UiEventKind::Change
            && route.route_id.is_some()
    }));
    assert!(document_tabs.routes.iter().any(|route| {
        route.binding_id == "DocumentTabs/CloseTab"
            && route.event_kind == UiEventKind::Submit
            && route.route_id.is_some()
    }));
    assert!(matches!(
        bridge
            .binding_for_control("DocumentTabsRoot", UiEventKind::Change)
            .expect("document tab activation binding")
            .payload(),
        EditorUiBindingPayload::DockCommand(_)
    ));
    assert!(matches!(
        bridge
            .binding_for_control("DocumentTabsRoot", UiEventKind::Submit)
            .expect("document tab close binding")
            .payload(),
        EditorUiBindingPayload::DockCommand(_)
    ));
}

#[test]
fn componentized_workbench_window_template_bridge_updates_module_navigation_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let activity_rail_width =
        zircon_runtime_interface::ui::design_tokens::EditorDesignTokens::workbench_dark()
            .chrome
            .activity_rail_width;

    assert!(control_bool(&bridge, "WorkbenchModuleEffect", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleCompile", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchModuleMaterial",
        "selected"
    ));
    assert!(!control_bool(&bridge, "WorkbenchModuleAssets", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
            .unwrap()
            .expect("material module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchModuleEffect", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleMaterial", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleMaterial", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchModuleAssets", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleMaterialWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleEffectWorkspace"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchMaterialRailGap")
            .expect("visible material rail gap projection")
            .frame
            .width,
        activity_rail_width
    );

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchMaterialNodeRow02", UiEventKind::Click)
            .unwrap()
            .expect("material node row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.node_roughness.select"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialNodeRow02",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialBaseColorRow",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchMaterialDomainDropdown", UiEventKind::Change)
            .unwrap()
            .expect("material domain dropdown should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.domain.edit"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleEffect", UiEventKind::Click)
            .unwrap()
            .expect("effect module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.effect.select"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleEffectWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleMaterialWorkspace"),
        Some(UiVisibility::Collapsed)
    );
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchEffectDamageFireRow", UiEventKind::Click)
            .unwrap()
            .expect("effect asset row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.effect.damage_fire_row.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchEffectHealthRegenRow",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchEffectDamageFireRow",
        "selected"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchEffectModifierHealingRow", UiEventKind::Click)
            .unwrap()
            .expect("effect modifier row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.effect.modifier_healing.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchEffectModifierHealthRow",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchEffectModifierHealingRow",
        "selected"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchEffectMagnitudeField", UiEventKind::Submit)
            .unwrap()
            .expect("effect magnitude field should expose a preview submit binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.effect.magnitude.commit"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
            .unwrap()
            .expect("material module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.select"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchMaterialNormalRow", UiEventKind::Click)
            .unwrap()
            .expect("material normal row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.normal_row.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialBaseColorRow",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialNormalRow",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleDiff", UiEventKind::Click)
            .unwrap()
            .expect("diff module command should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.diff.invoke"
    ));
    assert!(!control_bool(&bridge, "WorkbenchModuleCompile", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleDiff", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleDiff", "checked"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleBrowse", UiEventKind::Click)
            .unwrap()
            .expect("browse module command should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.browse.invoke"
    ));
    assert!(!control_bool(&bridge, "WorkbenchModuleBrowse", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleDiff", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleAssets", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleAssetsWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleMaterialWorkspace"),
        Some(UiVisibility::Collapsed)
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchModuleMaterial",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchAssetsImportButton", UiEventKind::Click)
            .unwrap()
            .expect("assets import button should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.assets.import.invoke"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchAssetsImportButton",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleAbility", UiEventKind::Click)
            .unwrap()
            .expect("ability module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.ability.select"
    ));
    assert!(control_bool(&bridge, "WorkbenchModuleAbility", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleAssets", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleAbilityWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleAssetsWorkspace"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchAbilityRailGap")
            .expect("visible ability rail gap projection")
            .frame
            .width,
        activity_rail_width
    );
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchAbilityTaskCostRow", UiEventKind::Click)
            .unwrap()
            .expect("ability task row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.ability.task_cost.select"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchAbilityTaskCostRow",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchAbilityTaskActivateRow",
        "selected"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchAbilityNameField", UiEventKind::Change)
            .unwrap()
            .expect("ability name field should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.ability.name.edit"
    ));

    for (control_id, action_id, workspace_id) in [
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
            "WorkbenchModuleRender",
            "workbench.module.render.select",
            "WorkbenchModuleRenderWorkspace",
        ),
        (
            "WorkbenchModuleHud",
            "workbench.module.hud.select",
            "WorkbenchModuleHudWorkspace",
        ),
    ] {
        assert!(matches!(
            bridge
                .dispatch_control_state(control_id, UiEventKind::Click)
                .unwrap()
                .expect("additional module tab should expose a preview binding")
                .payload(),
            EditorUiBindingPayload::MenuAction { action_id: dispatched_action }
                if dispatched_action == action_id
        ));
        assert!(control_bool(&bridge, control_id, "selected"));
        assert_eq!(
            control_visibility(&bridge, workspace_id),
            Some(UiVisibility::Visible)
        );
        assert_eq!(
            control_visibility(&bridge, "WorkbenchModuleAbilityWorkspace"),
            Some(UiVisibility::Collapsed)
        );
    }

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchTagsAbilityActivateRow", UiEventKind::Click)
            .unwrap()
            .expect("tags row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.tags.ability_activate.select"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchPerceptionConfigDropdown", UiEventKind::Change)
            .unwrap()
            .expect("perception dropdown should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.perception.config.edit"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchRenderLightingPassRow", UiEventKind::Click)
            .unwrap()
            .expect("render pass row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.render.lighting_pass.select"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchHudScreenDropdown", UiEventKind::Change)
            .unwrap()
            .expect("hud dropdown should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.hud.screen.edit"
    ));
}
