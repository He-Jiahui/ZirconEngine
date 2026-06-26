use std::collections::BTreeSet;

use super::super::support::*;
use super::support::{control_bool, control_string, control_visibility};
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;
use crate::ui::retained_host::HostInvalidationMask;
use zircon_runtime_interface::ui::tree::UiVisibility;

const DECLARED_WORKBENCH_MODULE_EVENT_COUNT: usize = 189;

const WORKBENCH_MODULE_EVENT_SOURCES: &[(&str, &str)] = &[
    (
        "workbench/shell/workbench_top_toolbar.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui"
        )),
    ),
    (
        "workbench/modules/core/gameplay/workbench_effect_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_effect_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_material_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ai/workbench_behavior_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/ai/workbench_behavior_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/assets/workbench_assets_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_vfx_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/gameplay/workbench_ability_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/gameplay/workbench_tags_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_tags_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ai/workbench_perception_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/ai/workbench_perception_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/rendering/workbench_render_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui"
        )),
    ),
    (
        "workbench/modules/core/ui/workbench_hud_workspace.zui",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/components/workbench/modules/core/ui/workbench_hud_workspace.zui"
        )),
    ),
];

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

#[derive(Debug)]
struct WorkbenchModuleEventCase {
    source: &'static str,
    control_id: String,
    binding_id: String,
    event_kind: UiEventKind,
}

fn declared_workbench_module_events() -> Vec<WorkbenchModuleEventCase> {
    let mut cases = Vec::new();
    for &(source, document) in WORKBENCH_MODULE_EVENT_SOURCES {
        let parsed = toml::from_str::<toml::Value>(document).unwrap_or_else(|error| {
            panic!("failed to parse {source} as TOML for module event coverage: {error}")
        });
        let nodes = parsed
            .get("nodes")
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("{source} should contain a [nodes] table"));
        for (node_name, node) in nodes {
            let control_id = node
                .get("control_id")
                .and_then(toml::Value::as_str)
                .map(str::to_string);
            let events = node
                .get("events")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten();
            for event in events {
                let Some(binding_id) = event.get("id").and_then(toml::Value::as_str) else {
                    continue;
                };
                if !binding_id.starts_with("WorkbenchModule/") {
                    continue;
                }
                let control_id = control_id.clone().unwrap_or_else(|| {
                    panic!("{source}:{node_name} declares {binding_id} without a control_id")
                });
                cases.push(WorkbenchModuleEventCase {
                    source,
                    control_id,
                    binding_id: binding_id.to_string(),
                    event_kind: module_event_kind(source, node_name, binding_id, event),
                });
            }
        }
    }
    cases
}

fn module_event_kind(
    source: &str,
    node_name: &str,
    binding_id: &str,
    event: &toml::Value,
) -> UiEventKind {
    match event.get("event").and_then(toml::Value::as_str) {
        Some("Click") => UiEventKind::Click,
        Some("Change") => UiEventKind::Change,
        Some("Submit") => UiEventKind::Submit,
        Some(other) => {
            panic!("{source}:{node_name} {binding_id} uses unsupported module event kind {other}")
        }
        None => panic!("{source}:{node_name} {binding_id} is missing an event kind"),
    }
}

#[test]
fn declared_workbench_module_events_dispatch_preview_actions() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let events = declared_workbench_module_events();
    assert_eq!(
        events.len(),
        DECLARED_WORKBENCH_MODULE_EVENT_COUNT,
        "the ZUI module surfaces should keep every declared WorkbenchModule/* event under coverage"
    );

    let mut seen_binding_ids = BTreeSet::new();
    for event in events {
        assert!(
            seen_binding_ids.insert(event.binding_id.clone()),
            "{} is declared more than once",
            event.binding_id
        );
        assert!(
            bridge.has_control(&event.control_id),
            "{} declares {} for missing control {}",
            event.source,
            event.binding_id,
            event.control_id
        );

        let binding = bridge
            .dispatch_binding_state_for_control(&event.control_id, &event.binding_id)
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} on {} failed to dispatch: {error}",
                    event.source, event.binding_id, event.control_id
                )
            })
            .unwrap_or_else(|| {
                panic!(
                    "{} {} on {} did not resolve to a template binding",
                    event.source, event.binding_id, event.control_id
                )
            });
        assert_eq!(binding.path().view_id, "WorkbenchModule");
        assert_eq!(binding.path().event_kind, event.event_kind);
        assert_eq!(
            event.binding_id.strip_prefix("WorkbenchModule/"),
            Some(binding.path().control_id.as_str())
        );
        let EditorUiBindingPayload::MenuAction { action_id } = binding.payload() else {
            panic!(
                "{} {} should dispatch as a Workbench preview menu action",
                event.source, event.binding_id
            );
        };
        assert!(
            is_workbench_preview_action(action_id),
            "{} {} resolves to unregistered preview action {}",
            event.source,
            event.binding_id,
            action_id
        );
    }
}

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
        Some("Playtest queued   predicted activation   GA_DashAttack")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchAbilityOutputRow")
            .expect("ability output row projection after command")
            .value_text
            .as_deref(),
        Some("Playtest queued   predicted activation   GA_DashAttack")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleRender", UiEventKind::Click)
        .unwrap()
        .expect("render module tab should expose a preview binding");
    bridge
        .dispatch_control_state("WorkbenchRenderCompileButton", UiEventKind::Click)
        .unwrap()
        .expect("render compile button should expose a preview binding");

    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Render graph compiled")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRenderCaptureRow", "value_text").as_deref(),
        Some("Windows DX12   30 fps   GPU 6.24 ms   compiled")
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

#[test]
fn workbench_module_dropdowns_open_select_and_close_with_shared_dropdown_path() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let harness = EventRuntimeHarness::new("zircon_workbench_module_dropdown_select");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
        .unwrap()
        .expect("material module tab should expose a preview binding");

    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "popup_open"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialDomainDropdown", "value").as_deref(),
        Some("surface")
    );

    let open_binding = bridge
        .dispatch_control_state("WorkbenchMaterialDomainDropdown", UiEventKind::Change)
        .unwrap()
        .expect("module dropdown should expose its field edit binding");
    assert!(matches!(
        open_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.domain.edit"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "popup_open"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "focused"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "selected"
    ));

    let effects = dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchMaterialDomainDropdown",
        "post_process",
    )
    .expect("module dropdown option selection should dispatch");

    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialDomainDropdown", "value").as_deref(),
        Some("post_process")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialDomainDropdown", "value_text").as_deref(),
        Some("post_process")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchMaterialDomainDropdown")
            .expect("material domain dropdown projection after selection")
            .value_text
            .as_deref(),
        Some("post_process")
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "popup_open"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "focused"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "selected"
    ));
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(harness.runtime.journal().records().len(), 1);

    let no_effects = dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchMaterialDomainDropdown",
        "unsupported_domain",
    )
    .expect("unknown module dropdown option should be swallowed");
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialDomainDropdown", "value").as_deref(),
        Some("post_process")
    );
    assert_eq!(harness.runtime.journal().records().len(), 1);
    assert_eq!(no_effects, UiHostEventEffects::default());
}
