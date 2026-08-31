use super::super::support::*;
use super::support::*;
use crate::core::extension::FieldEditorInstance;
use crate::ui::retained_host::HostInvalidationMask;
use crate::ui::workbench::snapshot::{
    InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot, InspectorSnapshot,
};

#[test]
fn componentized_workbench_inspector_property_edit_updates_row_preview() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &crate::ui::workbench::snapshot::SceneEntries::default(),
            Some(&inspector_with_component_properties()),
        )
        .unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchTransformScaleX", "value").as_deref(),
        Some("1.25")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformScale", "value").as_deref(),
        Some("X 1.25   Y 2.50   Z 0.75")
    );

    let effects = dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchComponentPropertyVirtualRow02",
        "Inspector/ComponentProperty04Edit",
        "true",
    )
    .expect("inspector property edit route should be handled")
    .unwrap();

    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertyVirtualRow02", "value").as_deref(),
        Some("true")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchComponentPropertyVirtualRow02",
            "value_text"
        )
        .as_deref(),
        Some("true")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchComponentPropertyVirtualRow02")
            .expect("material property row projection")
            .value_text
            .as_deref(),
        Some("true")
    );

    let commit_effects = dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchComponentPropertySlot04Row",
        "Inspector/ComponentProperty04Commit",
        "Visible    false",
    )
    .expect("inspector property commit route should be handled")
    .unwrap();

    assert!(commit_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertySlot04Row", "value").as_deref(),
        Some("false")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertySlot04Row", "value_text").as_deref(),
        Some("false")
    );

    let virtual_effects = dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchComponentPropertyVirtualRow05",
        "Inspector/ComponentProperty04Edit",
        "2",
    )
    .expect("virtual inspector property edit route should be handled")
    .unwrap();

    assert!(virtual_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertyVirtualRow05", "value").as_deref(),
        Some("2")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchComponentPropertyVirtualRow05",
            "value_text"
        )
        .as_deref(),
        Some("2")
    );
}

#[test]
fn componentized_workbench_inspector_scroll_rebinds_only_the_shifted_physical_rows() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_property_scroll");
    let mut inspector = inspector_with_component_properties();
    let prototype = inspector.plugin_components[0].properties[0].clone();
    for index in 5..64 {
        let mut property = prototype.clone();
        property.field_id = format!("property_{index:02}");
        property.name = property.field_id.clone();
        property.label = format!("Property {index:02}");
        property.value = index.to_string();
        inspector.plugin_components[0].properties.push(property);
    }
    let properties = inspector.plugin_components[0].properties.clone();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &crate::ui::workbench::snapshot::SceneEntries::default(),
            Some(&inspector),
        )
        .unwrap();

    let owner_id = bridge
        .surface()
        .unique_control_node_id("WorkbenchInspectorMeshProperties")
        .expect("virtualized component property owner");
    let first_root = bridge
        .surface()
        .virtual_list_prototype_slot_roots(owner_id)
        .unwrap()[0];
    let first_before = bridge
        .surface()
        .virtual_list_binding_for_node(owner_id, first_root)
        .expect("initial physical slot binding")
        .logical_index;
    let scroll_point = control_center(&bridge, "WorkbenchInspectorMeshProperties");

    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Scroll, scroll_point).with_scroll_delta(196.0),
    )
    .expect("component property scroll should publish an incremental paint")
    .unwrap();

    let first_after = bridge
        .surface()
        .virtual_list_binding_for_node(owner_id, first_root)
        .expect("shifted physical slot binding")
        .logical_index;
    assert!(first_after > first_before);
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertySlot04Row", "value_text").as_deref(),
        Some(properties[first_after].value.as_str())
    );
}

#[test]
fn componentized_workbench_inspector_property_edit_rejects_disabled_customizations() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let mut inspector = inspector_with_component_properties();
    inspector.plugin_components[0].customization_available = false;
    bridge
        .sync_scene_and_inspector(
            &crate::ui::workbench::snapshot::SceneEntries::default(),
            Some(&inspector),
        )
        .unwrap();

    assert!(!control_bool(
        &bridge,
        "WorkbenchComponentPropertySlot04Row",
        "inspector_property_editable"
    ));

    let effects = dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchComponentPropertySlot04Row",
        "Inspector/ComponentProperty04Edit",
        "Visible false",
    )
    .expect("disabled customization edit route should be handled")
    .unwrap();

    assert!(!effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertySlot04Row", "value").as_deref(),
        Some("true")
    );
}

#[test]
fn componentized_workbench_transform_axis_edit_updates_field_and_row_preview() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &crate::ui::workbench::snapshot::SceneEntries::default(),
            Some(&inspector_with_component_properties()),
        )
        .unwrap();

    let position_x = bridge
        .host_projection()
        .node_by_control_id("WorkbenchTransformPositionX")
        .expect("position X field projection");
    assert!(position_x.routes.iter().any(|route| {
        route.binding_id == "Inspector/TransformPositionXEdit"
            && route.event_kind == UiEventKind::Change
    }));
    assert!(position_x.routes.iter().any(|route| {
        route.binding_id == "Inspector/TransformPositionXCommit"
            && route.event_kind == UiEventKind::Submit
    }));

    let position_effects = dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchTransformPositionX",
        "Inspector/TransformPositionXEdit",
        "X 42.0",
    )
    .expect("position axis edit route should be handled")
    .unwrap();

    assert!(position_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!position_effects.render_dirty);
    assert!(!position_effects.presentation_dirty);
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPositionX", "value").as_deref(),
        Some("42.0")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPosition", "value").as_deref(),
        Some("X 42.0   Y 3.5   Z -8.0")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchTransformPositionX")
            .expect("position X field projection after edit")
            .value_text
            .as_deref(),
        Some("42.0")
    );

    let rotation_y = bridge
        .host_projection()
        .node_by_control_id("WorkbenchTransformRotationY")
        .expect("rotation Y field projection");
    assert!(rotation_y.routes.is_empty());

    dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchTransformScaleZ",
        "Inspector/TransformScaleZEdit",
        "2.00",
    )
    .expect("scale axis edit route should be handled")
    .unwrap();
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformScaleZ", "value").as_deref(),
        Some("2.00")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformScale", "value").as_deref(),
        Some("X 1.25   Y 2.50   Z 2.00")
    );
}

#[test]
fn componentized_workbench_module_field_edit_updates_value_preview() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleAbility", UiEventKind::Click)
        .expect("ability module tab should dispatch")
        .expect("ability module tab should expose a preview binding");

    let ability_name = bridge
        .host_projection()
        .node_by_control_id("WorkbenchAbilityNameField")
        .expect("ability name field projection");
    assert!(ability_name.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/AbilityNameEdit"
            && route.event_kind == UiEventKind::Change
    }));
    assert!(ability_name.routes.iter().any(|route| {
        route.binding_id == "WorkbenchModule/AbilityNameCommit"
            && route.event_kind == UiEventKind::Submit
    }));
    assert_eq!(
        control_string(&bridge, "WorkbenchAbilityNameField", "value").as_deref(),
        Some("GA_DashAttack")
    );

    let ability_effects = dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchAbilityNameField",
        "WorkbenchModule/AbilityNameEdit",
        "GA_Dash_Preview",
    )
    .expect("ability module field edit route should be handled")
    .unwrap();

    assert!(ability_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!ability_effects.render_dirty);
    assert!(!ability_effects.presentation_dirty);
    assert_eq!(
        control_string(&bridge, "WorkbenchAbilityNameField", "value").as_deref(),
        Some("GA_Dash_Preview")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchAbilityNameField", "value_text").as_deref(),
        Some("GA_Dash_Preview")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchAbilityNameField")
            .expect("ability name field projection after edit")
            .value_text
            .as_deref(),
        Some("GA_Dash_Preview")
    );

    bridge
        .dispatch_control_state("WorkbenchModuleRender", UiEventKind::Click)
        .expect("render module tab should dispatch")
        .expect("render module tab should expose a preview binding");
    let render_effects = dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchRenderPipelineField",
        "WorkbenchModule/RenderPipelineCommit",
        "ForwardPlus.rp",
    )
    .expect("render module field commit route should be handled")
    .unwrap();
    assert!(render_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(
        control_string(&bridge, "WorkbenchRenderPipelineField", "value").as_deref(),
        Some("ForwardPlus.rp")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRenderPipelineField", "value_text").as_deref(),
        Some("ForwardPlus.rp")
    );

    let ignored_effects = dispatch_componentized_workbench_surface_control_edited(
        &mut bridge,
        "WorkbenchAbilityNameField",
        "WorkbenchModule/RenderPipelineEdit",
        "WrongTarget.rp",
    )
    .expect("known module edit binding should be swallowed on the wrong control")
    .unwrap();
    assert!(!ignored_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(
        control_string(&bridge, "WorkbenchAbilityNameField", "value").as_deref(),
        Some("GA_Dash_Preview")
    );
}

fn inspector_with_component_properties() -> InspectorSnapshot {
    InspectorSnapshot {
        id: 2,
        name: "GameplayRoot".to_string(),
        parent: "World".to_string(),
        translation: ["12.0".to_string(), "3.5".to_string(), "-8.0".to_string()],
        scale: ["1.25".to_string(), "2.50".to_string(), "0.75".to_string()],
        render_layer_mask: 1,
        plugin_components: vec![InspectorPluginComponentSnapshot {
            component_id: "zircon.transform".to_string(),
            display_name: "Transform Component".to_string(),
            plugin_id: "zircon.core".to_string(),
            customization_available: true,
            customization_ui_document: None,
            customization_controller: None,
            customization_template_id: None,
            customization_data_root: None,
            customization_bindings: Vec::new(),
            diagnostic: None,
            properties: vec![
                InspectorPluginComponentPropertySnapshot {
                    field_id: "visible".to_string(),
                    name: "visible".to_string(),
                    label: "Visible".to_string(),
                    value: "true".to_string(),
                    value_kind: "bool".to_string(),
                    editable: true,
                    field_editor: FieldEditorInstance::automatic(),
                },
                InspectorPluginComponentPropertySnapshot {
                    field_id: "cast_shadows".to_string(),
                    name: "cast_shadows".to_string(),
                    label: "Cast Shadows".to_string(),
                    value: "false".to_string(),
                    value_kind: "bool".to_string(),
                    editable: true,
                    field_editor: FieldEditorInstance::automatic(),
                },
                InspectorPluginComponentPropertySnapshot {
                    field_id: "receive_shadows".to_string(),
                    name: "receive_shadows".to_string(),
                    label: "Receive Shadows".to_string(),
                    value: "true".to_string(),
                    value_kind: "bool".to_string(),
                    editable: true,
                    field_editor: FieldEditorInstance::automatic(),
                },
                InspectorPluginComponentPropertySnapshot {
                    field_id: "material_slot".to_string(),
                    name: "material_slot".to_string(),
                    label: "Material Slot".to_string(),
                    value: "0".to_string(),
                    value_kind: "u32".to_string(),
                    editable: true,
                    field_editor: FieldEditorInstance::automatic(),
                },
                InspectorPluginComponentPropertySnapshot {
                    field_id: "lightmap_index".to_string(),
                    name: "lightmap_index".to_string(),
                    label: "Lightmap".to_string(),
                    value: "1".to_string(),
                    value_kind: "u32".to_string(),
                    editable: true,
                    field_editor: FieldEditorInstance::automatic(),
                },
            ],
        }],
    }
}
