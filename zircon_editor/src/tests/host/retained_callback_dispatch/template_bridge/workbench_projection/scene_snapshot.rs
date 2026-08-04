use super::*;

#[test]
fn componentized_workbench_window_template_bridge_syncs_scene_and_inspector_snapshot_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert_virtual_row_repeat(
        &bridge,
        "WorkbenchInspectorMesh",
        "WorkbenchComponentPropertySlot04Row",
        "WorkbenchComponentPropertyVirtualRow",
        4,
        "v2",
    );
    let scene_entries = SceneEntries::from_entries(
        vec![
            SceneEntry {
                id: 1,
                name: "World".to_string(),
                depth: 0,
            },
            SceneEntry {
                id: 2,
                name: "GameplayRoot".to_string(),
                depth: 1,
            },
            SceneEntry {
                id: 3,
                name: "CameraRig".to_string(),
                depth: 2,
            },
        ],
        [2],
    );
    let inspector = InspectorSnapshot {
        id: 2,
        name: "GameplayRoot".to_string(),
        parent: "World".to_string(),
        translation: ["12.0".to_string(), "3.5".to_string(), "-8.0".to_string()],
        scale: ["1.0".to_string(), "1.0".to_string(), "1.0".to_string()],
        plugin_components: vec![InspectorPluginComponentSnapshot {
            component_id: "zircon.transform".to_string(),
            display_name: "Transform Component".to_string(),
            plugin_id: "zircon.core".to_string(),
            customization_available: false,
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
    };

    bridge
        .sync_scene_and_inspector(&scene_entries, Some(&inspector))
        .unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchSceneRootItem", "text").as_deref(),
        Some("World")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneEnvironmentItem", "text").as_deref(),
        Some("GameplayRoot")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchSceneEnvironmentItem",
        "selected"
    ));
    assert_eq!(
        control_integer(&bridge, "WorkbenchSceneEnvironmentItem", "tree_depth"),
        Some(1)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchScenePropsItem"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchInspectorTitle", "text").as_deref(),
        Some("GameplayRoot")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPosition", "value").as_deref(),
        Some("X 12.0   Y 3.5   Z -8.0")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPositionX", "value").as_deref(),
        Some("12.0")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPositionY", "value").as_deref(),
        Some("3.5")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPositionZ", "value").as_deref(),
        Some("-8.0")
    );
    let position_x = bridge
        .host_projection()
        .node_by_control_id("WorkbenchTransformPositionX")
        .expect("position X field projection");
    assert_eq!(position_x.component, "InputField");
    assert!(control_has_class(
        &bridge,
        "WorkbenchTransformPositionX",
        "workbench-axis-value-field"
    ));
    assert!(position_x.routes.iter().any(|route| {
        route.binding_id == "Inspector/TransformPositionXEdit"
            && route.event_kind == UiEventKind::Change
    }));
    assert!(position_x.routes.iter().any(|route| {
        route.binding_id == "Inspector/TransformPositionXCommit"
            && route.event_kind == UiEventKind::Submit
    }));
    assert_eq!(
        control_string(&bridge, "WorkbenchMeshLabel", "text").as_deref(),
        Some("Transform Component")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMeshRow", "text").as_deref(),
        Some("Visible")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMeshRow", "value_text").as_deref(),
        Some("true")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialRow", "text").as_deref(),
        Some("Cast Shadows")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialRow", "value_text").as_deref(),
        Some("false")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMeshRow", "inspector_property_field_id").as_deref(),
        Some("visible")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchMaterialRow",
            "inspector_property_field_id"
        )
        .as_deref(),
        Some("cast_shadows")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchMaterialRow",
            "inspector_property_value_kind"
        )
        .as_deref(),
        Some("bool")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialRow", "value").as_deref(),
        Some("false")
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialRow",
        "inspector_property_editable"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertySlot03Row", "text").as_deref(),
        Some("Receive Shadows")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertySlot03Row", "value_text").as_deref(),
        Some("true")
    );
    assert_eq!(
        template_contract_node(
            &to_host_contract_workbench_window_nodes(Some(bridge.host_projection())),
            "WorkbenchComponentPropertySlot03Row",
        )
        .layout_content_offset_x,
        34.0
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchComponentPropertySlot04Row",
            "inspector_property_field_id"
        )
        .as_deref(),
        Some("material_slot")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertyVirtualRow05", "text").as_deref(),
        Some("Lightmap")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchComponentPropertyVirtualRow05",
            "value_text"
        )
        .as_deref(),
        Some("1")
    );
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchComponentPropertyVirtualRow05")
        .expect("virtual component property row projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "Inspector/ComponentProperty04Edit"
                && route.event_kind == UiEventKind::Change
        ));
    let mesh_row = bridge
        .host_projection()
        .node_by_control_id("WorkbenchMeshRow")
        .expect("mesh/property row projection");
    assert_eq!(mesh_row.component, "InputField");
    assert!(control_has_class(
        &bridge,
        "WorkbenchMeshRow",
        "workbench-component-property-row"
    ));
    assert!(mesh_row.routes.iter().any(|route| {
        route.binding_id == "Inspector/ComponentProperty01Edit"
            && route.event_kind == UiEventKind::Change
    }));
    assert!(mesh_row.routes.iter().any(|route| {
        route.binding_id == "Inspector/ComponentProperty01Commit"
            && route.event_kind == UiEventKind::Submit
    }));
    let material_row = bridge
        .host_projection()
        .node_by_control_id("WorkbenchMaterialRow")
        .expect("material/property row projection");
    let material_host_nodes =
        to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let material_host_row = template_contract_node(&material_host_nodes, "WorkbenchMaterialRow");
    assert_eq!(
        style_color_u8(
            material_host_row
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([40, 46, 50, 255])
    );
    assert_eq!(
        style_color_u8(material_host_row.button_style.element.border_color.as_ref()),
        Some([52, 61, 67, 255])
    );
    assert_eq!(
        material_host_row.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(181, 192, 197)
    );
    assert!(material_row.routes.iter().any(|route| {
        route.binding_id == "Inspector/ComponentProperty02Edit"
            && route.event_kind == UiEventKind::Change
    }));
    assert!(material_row.routes.iter().any(|route| {
        route.binding_id == "Inspector/ComponentProperty02Commit"
            && route.event_kind == UiEventKind::Submit
    }));

    let extended_entries = SceneEntries::from_entries(
        (0..8)
            .map(|index| SceneEntry {
                id: (index + 1) as u64,
                name: format!("SceneNode_{:02}", index + 1),
                depth: if index == 0 { 0 } else { 1 + index % 3 },
            })
            .collect::<Vec<_>>(),
        [8],
    );
    bridge
        .sync_scene_and_inspector(&extended_entries, Some(&inspector))
        .unwrap();
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneSlot07Item", "text").as_deref(),
        Some("SceneNode_07")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneSlot08Item", "text").as_deref(),
        Some("SceneNode_08")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchSceneSlot08Item",
        "selected"
    ));
    assert_eq!(
        control_integer(&bridge, "WorkbenchSceneSlot08Item", "tree_depth"),
        Some(2)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneSlot09Item"),
        Some(UiVisibility::Collapsed)
    );

    bridge
        .sync_scene_and_inspector(&SceneEntries::default(), None)
        .unwrap();
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneRootItem"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchInspectorTitle", "text").as_deref(),
        Some("No Selection")
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchInspectorTransform"),
        Some(UiVisibility::Collapsed)
    );
    let cleared_material_host_nodes =
        to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let cleared_material_row =
        template_contract_node(&cleared_material_host_nodes, "WorkbenchMaterialRow");
    assert_eq!(
        style_color_u8(
            cleared_material_row
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        None
    );
    assert_eq!(
        style_color_u8(
            cleared_material_row
                .button_style
                .element
                .border_color
                .as_ref()
        ),
        None
    );
    assert_eq!(
        cleared_material_row.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(216, 227, 231)
    );
}

#[test]
fn componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert_virtual_row_repeat(
        &bridge,
        "WorkbenchSceneTree",
        "WorkbenchSceneSlot10Item",
        "WorkbenchSceneVirtualItem",
        10,
        "v2",
    );
    let inspector = InspectorSnapshot {
        id: 13,
        name: "SceneNode_13".to_string(),
        parent: "SceneNode_12".to_string(),
        translation: ["0.0".to_string(), "1.0".to_string(), "2.0".to_string()],
        scale: ["1.0".to_string(), "1.0".to_string(), "1.0".to_string()],
        plugin_components: Vec::new(),
    };
    let thirteen_entries = numbered_scene_entries(13, 12);
    bridge
        .sync_scene_and_inspector(&thirteen_entries, Some(&inspector))
        .unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchSceneVirtualItem11", "text").as_deref(),
        Some("SceneNode_11")
    );
    assert_eq!(
        control_integer(&bridge, "WorkbenchSceneVirtualItem12", "scene_node_id"),
        Some(12)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneVirtualItem13", "text").as_deref(),
        Some("SceneNode_13")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchSceneVirtualItem13",
        "selected"
    ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchSceneVirtualItem13")
        .is_some());
    let virtual_binding = bridge
        .binding_for_control("WorkbenchSceneVirtualItem13", UiEventKind::Click)
        .expect("virtual scene row binding should resolve through authored prototype route");
    assert!(matches!(
        virtual_binding.payload(),
        EditorUiBindingPayload::SelectionCommand(_)
    ));
    assert!(
        bridge
            .surface()
            .last_rebuild_report
            .control_pool_created_count
            >= 3
    );

    let two_entries = numbered_scene_entries(2, 1);
    bridge
        .sync_scene_and_inspector(&two_entries, Some(&inspector))
        .unwrap();
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchSceneVirtualItem11")
        .is_none());
    assert!(
        bridge
            .surface()
            .last_rebuild_report
            .control_pool_recycled_count
            >= 3
    );

    let twelve_entries = numbered_scene_entries(12, 11);
    bridge
        .sync_scene_and_inspector(&twelve_entries, Some(&inspector))
        .unwrap();
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneVirtualItem12", "text").as_deref(),
        Some("SceneNode_12")
    );
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchSceneVirtualItem12")
        .is_some());
    assert!(
        bridge
            .surface()
            .last_rebuild_report
            .control_pool_reused_count
            >= 2
    );
}
