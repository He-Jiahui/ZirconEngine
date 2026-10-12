use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use crate::tests::support::env_lock;

use super::BuiltinWorkbenchWindowTemplateSurfaceBridge;

const MODULES: &[(&str, &str)] = &[
    ("menu.item.effect", "WorkbenchModuleEffectWorkspace"),
    ("menu.item.material", "WorkbenchModuleMaterialWorkspace"),
    ("menu.item.behavior", "WorkbenchModuleBehaviorWorkspace"),
    ("menu.item.assets", "WorkbenchModuleAssetsWorkspace"),
    ("menu.item.v_f_x", "WorkbenchModuleVfxWorkspace"),
    ("menu.item.ability", "WorkbenchModuleAbilityWorkspace"),
    ("menu.item.tags", "WorkbenchModuleTagsWorkspace"),
    ("menu.item.perception", "WorkbenchModulePerceptionWorkspace"),
    ("menu.item.render", "WorkbenchModuleRenderWorkspace"),
    ("menu.item.h_u_d", "WorkbenchModuleHudWorkspace"),
];

fn select(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge, item: &str) {
    assert!(bridge
        .dispatch_control_state("MenuSlot2", UiEventKind::Click)
        .unwrap()
        .is_some());
    assert!(bridge.control_bool("UnrealWindowMenuPopup", "popup_open"));
    assert!(bridge
        .control_string_array("UnrealWindowMenuPopup", "menu_items")
        .iter()
        .any(|value| value.contains(&format!("action={item}"))));
    assert!(bridge
        .dispatch_workbench_module_overflow_menu_item_state("UnrealWindowMenuPopup", item,)
        .unwrap()
        .is_some());
}

#[test]
fn default_scene_navigation_keeps_the_unreal_tree_without_module_retention() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let nodes = bridge.surface().tree.nodes.len();
    let slots = bridge.surface().tree.layout_slots().len();
    for (_, control) in MODULES {
        assert!(!bridge.has_control(control));
    }
    select(&mut bridge, "menu.item.scene");
    assert!(bridge.module_workspace.is_none());
    assert_eq!(bridge.surface().tree.nodes.len(), nodes);
    assert_eq!(bridge.surface().tree.layout_slots().len(), slots);
    assert!(bridge.control_frame("WorkbenchViewportSurface").is_some());
}

#[test]
fn module_switches_reuse_ids_and_preserve_unreal_geometry_and_style() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let baseline_frames = bridge.frames();
    let base_ids = bridge
        .surface()
        .tree
        .nodes
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let base_style = bridge
        .surface()
        .tree
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            node.template_metadata.as_ref().map(|metadata| {
                (
                    *id,
                    (
                        metadata.style_overrides.clone(),
                        metadata.style_tokens.clone(),
                    ),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let baseline_nodes = bridge.surface().tree.nodes.len();
    let baseline_slots = bridge.surface().tree.layout_slots().len();
    let baseline_max_id = bridge
        .surface()
        .tree
        .nodes
        .keys()
        .map(|id| id.0)
        .max()
        .unwrap();
    let mut sizes = BTreeMap::new();
    let mut previous_root = None;
    let mut previous_incarnation = None;
    for cycle in 0..3 {
        for (item, control) in MODULES {
            select(&mut bridge, item);
            assert!(bridge.has_control(control));
            for (_, other) in MODULES {
                assert_eq!(bridge.has_control(other), other == control);
            }
            let root = bridge.module_workspace.as_ref().unwrap().root_id;
            let incarnation = bridge.surface().tree.nodes[&root].paint_order;
            if let Some(previous) = previous_root {
                assert_eq!(
                    root, previous,
                    "switching must reuse the retired root's numeric ID"
                );
                assert_ne!(
                    Some(incarnation),
                    previous_incarnation,
                    "reused IDs need a new incarnation"
                );
            }
            previous_root = Some(root);
            previous_incarnation = Some(incarnation);
            let size = (
                bridge.surface().tree.nodes.len(),
                bridge.surface().tree.layout_slots().len(),
                bridge
                    .surface()
                    .tree
                    .nodes
                    .keys()
                    .map(|id| id.0)
                    .max()
                    .unwrap(),
            );
            if cycle == 0 {
                sizes.insert(*item, size);
            } else {
                assert_eq!(
                    sizes[item], size,
                    "repeated switches must not grow IDs or slot storage demand"
                );
            }
            assert_eq!(bridge.frames(), baseline_frames);
            for id in &base_ids {
                assert!(bridge.surface().tree.nodes.contains_key(id));
                if let Some(expected) = base_style.get(id) {
                    let metadata = bridge.surface().tree.nodes[id]
                        .template_metadata
                        .as_ref()
                        .unwrap();
                    assert_eq!(
                        &(
                            metadata.style_overrides.clone(),
                            metadata.style_tokens.clone()
                        ),
                        expected
                    );
                }
            }
        }
        select(&mut bridge, "menu.item.scene");
        assert!(bridge.module_workspace.is_none());
        assert_eq!(bridge.surface().tree.nodes.len(), baseline_nodes);
        assert_eq!(bridge.surface().tree.layout_slots().len(), baseline_slots);
        assert_eq!(
            bridge
                .surface()
                .tree
                .nodes
                .keys()
                .map(|id| id.0)
                .max()
                .unwrap(),
            baseline_max_id
        );
        assert_eq!(bridge.frames(), baseline_frames);
    }
}

#[test]
fn failed_child_mount_preserves_the_live_module_and_input_state() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    select(&mut bridge, "menu.item.ability");
    let before = bridge.surface().clone();
    let before_projection = bridge.host_projection().clone();
    let before_bindings = bridge.bindings_by_id.clone();
    let (projection, mut surface, _) = bridge.runtime.project_workbench_module_workspace(
        "res://ui/editor/modules/core/gameplay/effect_workspace.zui",
        "WorkbenchEffectWorkspace",
    ).unwrap();
    let root = surface.tree.roots[0];
    surface
        .tree
        .nodes
        .get_mut(&root)
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .control_id = Some("WorkbenchWindowRoot".to_string());
    let current = bridge.module_workspace.as_ref().unwrap();
    assert!(bridge
        .template_surface
        .stage_module_workspace(
            bridge.runtime.as_ref(),
            Some(current.root_id),
            Some("WorkbenchModuleAbilityWorkspace"),
            Some((projection, surface)),
        )
        .is_err());
    assert_eq!(bridge.surface(), &before);
    assert_eq!(bridge.host_projection(), &before_projection);
    assert_eq!(bridge.bindings_by_id, before_bindings);
    assert!(bridge.has_control("WorkbenchModuleAbilityWorkspace"));
    assert!(!bridge.has_control("WorkbenchModuleEffectWorkspace"));
}

#[test]
fn returning_to_scene_retires_module_focus_and_routes() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let base_ids = bridge
        .surface()
        .tree
        .nodes
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    select(&mut bridge, "menu.item.effect");
    let module_ids = bridge
        .surface()
        .tree
        .nodes
        .keys()
        .filter(|id| !base_ids.contains(id))
        .copied()
        .collect::<Vec<_>>();
    let candidates = module_ids
        .iter()
        .copied()
        .filter(|id| bridge.surface().tree.nodes[id].is_focus_candidate())
        .collect::<Vec<_>>();
    let focused = candidates
        .into_iter()
        .find(|id| bridge.template_surface.surface.focus_node(*id).is_ok())
        .expect("the live effect workspace must expose a focusable input");
    let binding = bridge
        .binding_for_control("WorkbenchEffectDurationRow", UiEventKind::Click)
        .unwrap()
        .clone();
    assert!(bridge
        .dispatch_binding_state_for_control(
            "WorkbenchEffectDurationRow",
            "WorkbenchModule/EffectDurationRow"
        )
        .unwrap()
        .is_some());
    select(&mut bridge, "menu.item.scene");
    assert!(!bridge.has_control("WorkbenchEffectDurationRow"));
    assert!(bridge
        .binding_by_id("WorkbenchModule/EffectDurationRow")
        .is_none());
    assert!(bridge
        .binding_for_control("WorkbenchEffectDurationRow", UiEventKind::Click)
        .is_none());
    assert_ne!(bridge.surface().focus.focused, Some(focused));
    for id in module_ids {
        assert!(bridge.surface().tree.node(id).is_none());
        assert!(bridge.surface().component_states.get(id).is_none());
    }
    assert_eq!(binding.path().view_id, "WorkbenchModule");
}

#[test]
fn stale_public_child_mount_rejects_actual_live_property_and_focus_updates() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    for change_focus in [false, true] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
        select(&mut bridge, "menu.item.effect");
        let parent = bridge.control_node_id("UnrealSceneViewViewport").unwrap();
        let (mut staged, _, proof) =
            zircon_runtime::ui::v2::UiV2SurfaceBuilder::stage_child_surface(
                bridge.surface(),
                parent,
                None,
                None,
            )
            .unwrap();
        if change_focus {
            let candidates = bridge
                .surface()
                .tree
                .nodes
                .iter()
                .filter_map(|(id, node)| {
                    (node.is_focus_candidate() && bridge.surface().focus.focused != Some(*id))
                        .then_some(*id)
                })
                .collect::<Vec<_>>();
            let candidate = candidates
                .into_iter()
                .find(|id| bridge.template_surface.surface.focus_node(*id).is_ok())
                .expect("the actual module exposes a focusable input");
            assert_eq!(bridge.surface().focus.focused, Some(candidate));
        } else {
            let status = bridge
                .control_node_id("WorkbenchWindowStatusBarRegion")
                .unwrap();
            let report = bridge
                .template_surface
                .surface
                .mutate_property(zircon_runtime::ui::surface::UiPropertyMutationRequest::new(
                    status,
                    "visible",
                    zircon_runtime_interface::ui::component::UiValue::Bool(false),
                ))
                .unwrap();
            assert_eq!(
                report.status,
                zircon_runtime::ui::surface::UiPropertyMutationStatus::Accepted
            );
        }
        let before_surface = bridge.surface().clone();
        let before_projection = bridge.host_projection().clone();
        let before_bindings = bridge.bindings_by_id.clone();
        assert!(
            zircon_runtime::ui::v2::UiV2SurfaceBuilder::commit_child_surface(
                &mut staged,
                &mut bridge.template_surface.surface,
                &proof,
            )
            .is_err()
        );
        assert_eq!(bridge.surface(), &before_surface);
        assert_eq!(bridge.host_projection(), &before_projection);
        assert_eq!(bridge.bindings_by_id, before_bindings);
        assert!(bridge.has_control("WorkbenchModuleEffectWorkspace"));
    }
}

#[test]
fn module_tools_trigger_invocation_proof_rejects_forged_sources_and_paths() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    for (item, menu) in [
        ("menu.item.assets", "WorkbenchAssetsWorldToolsMenu"),
        ("menu.item.assets", "WorkbenchAssetsGameplayToolsMenu"),
        ("menu.item.assets", "WorkbenchAssetsProductionToolsMenu"),
        ("menu.item.ability", "WorkbenchAbilityAnimationToolsMenu"),
        ("menu.item.render", "WorkbenchRenderToolsMenu"),
        ("menu.item.h_u_d", "WorkbenchHudToolsMenu"),
    ] {
        select(&mut bridge, item);
        let spec = crate::ui::retained_host::host_contract::componentized_workbench_regions::module_tools_menu_spec(menu).unwrap();
        let metadata = |control: &str| bridge.surface().tree.nodes.values()
            .filter_map(|node| node.template_metadata.as_ref())
            .find(|metadata| metadata.control_id.as_deref() == Some(control)).unwrap().clone();
        let root = metadata(spec.root_control_id);
        let trigger = metadata(spec.trigger_control_id);
        let owned = |root: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
                     trigger: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata| {
            spec.trigger_source_is_owned(
                root.source_path.as_deref().unwrap_or(""), root.instance_path.as_deref(),
                trigger.source_path.as_deref().unwrap_or(""),
                trigger.source_node_id.as_deref().unwrap_or(""), trigger.instance_path.as_deref(),
            )
        };
        assert!(owned(&root, &trigger), "the actual compiled caller is admitted: {menu}");
        let current = bridge.surface().surface_frame();
        let trigger_row = bridge.host_projection().node_by_control_id(spec.trigger_control_id).unwrap();
        assert!(std::sync::Arc::ptr_eq(trigger_row.source_surface_frame.as_ref().unwrap(), &current));
        let mut wrong_caller = trigger.clone();
        wrong_caller.instance_path.as_mut().unwrap().last_mut().unwrap().source_node_id = "forged_caller".into();
        assert!(!owned(&root, &wrong_caller));
        let mut wrong_caller_path = trigger.clone();
        wrong_caller_path.instance_path.as_mut().unwrap().last_mut().unwrap().source_path = "res://forged/module.zui".into();
        assert!(!owned(&root, &wrong_caller_path));
        let mut missing_chain = trigger.clone();
        missing_chain.instance_path = None;
        assert!(!owned(&root, &missing_chain));
        let mut missing_root_chain = root.clone();
        missing_root_chain.instance_path = None;
        assert!(!owned(&missing_root_chain, &trigger));
        let mut wrong_prefix = trigger.clone();
        wrong_prefix.instance_path.as_mut().unwrap().insert(0,
            zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep {
                source_path: "res://foreign/root.zui".into(), source_node_id: "foreign_instance".into(),
            });
        assert!(!owned(&root, &wrong_prefix), "an arbitrary matching caller cannot authorize a foreign prefix");
        let mut forged_primitive = trigger.clone();
        forged_primitive.source_path = root.source_path.clone();
        assert!(!owned(&root, &forged_primitive));
        let mut forged_leaf = trigger.clone();
        forged_leaf.source_node_id = Some(spec.trigger_source_node_id.into());
        assert!(!owned(&root, &forged_leaf));
        assert!(std::sync::Arc::ptr_eq(&current, &bridge.surface().surface_frame()));
        assert_eq!(metadata(spec.trigger_control_id), trigger, "negative probes never rewrite the live source");
    }
}
