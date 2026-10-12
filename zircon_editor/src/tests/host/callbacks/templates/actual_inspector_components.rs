use super::super::support::*;
use super::support::*;
use crate::core::editing::intent::EditorIntent;
use crate::core::extension::{
    FieldEditorContainer, InspectorCustomizationChain, InspectorCustomizationDescriptor,
};
use crate::ui::workbench::state::EditorState;
use std::sync::Arc;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::binding::UiBindingValue;

fn default_state() -> EditorState {
    let mut state = EditorState::with_default_selection(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
    );
    state.mark_project_open();
    state
}

fn row_for_field(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge, field_id: &str) -> String {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            let metadata = node.template_metadata.as_ref()?;
            (metadata
                .attributes
                .get("inspector_property_field_id")
                .and_then(toml::Value::as_str)
                == Some(field_id))
            .then(|| metadata.control_id.clone())
            .flatten()
        })
        .unwrap_or_else(|| panic!("actual field must materialize: {field_id}"))
}

fn assert_actual_row_input_state(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    editable: bool,
) {
    let surface = bridge.surface();
    let node = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .expect("actual row node must exist");
    assert!(node.is_render_visible());
    assert_eq!(node.state_flags.focusable, editable);
    assert_eq!(node.state_flags.clickable, editable);
    let arranged = surface
        .arranged_node(node.node_id)
        .expect("row must be arranged");
    assert!(arranged.is_render_visible());
    assert_eq!(arranged.focusable, editable);
    assert_eq!(arranged.clickable, editable);
}

#[test]
fn actual_dynamic_component_rejects_type_outside_owner_namespace() {
    let _guard = env_lock().lock().unwrap();
    let mut state = default_state();
    state.world.expect_with_world_mut(|scene| {
        let error = scene
            .register_component_type(
                zircon_runtime::core::framework::scene::ComponentTypeDescriptor::new(
                    "plugin.audit.First",
                    "audit",
                    "First",
                )
                .with_property("coverage", "Scalar", true),
            )
            .expect_err("type namespace must match its owner");
        assert!(matches!(
            error,
            zircon_runtime::scene::SceneError::ComponentTypePluginPrefixMismatch {
                type_id,
                plugin_id,
            } if type_id == "plugin.audit.First" && plugin_id == "audit"
        ));
        assert!(scene
            .component_type_descriptor("plugin.audit.First")
            .is_none());
    });
}

#[test]
fn actual_default_components_reach_snapshot_virtual_rows_and_reject_writes() {
    let _guard = env_lock().lock().unwrap();
    let mut state = default_state();
    let entities = state
        .world
        .expect_with_world(|scene| scene.nodes().iter().map(|node| node.id).collect::<Vec<_>>());
    let mut found = std::collections::BTreeSet::new();
    for entity in entities {
        state
            .apply_intent(EditorIntent::SelectNode(entity))
            .unwrap();
        let snapshot = state.snapshot();
        let inspector = snapshot.inspector.as_ref().unwrap();
        assert!(inspector.plugin_components.is_empty());
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
        bridge
            .sync_scene_and_inspector(&snapshot.scene_entries, Some(inspector))
            .unwrap();
        let property = inspector
            .native_fields
            .first()
            .expect("default Camera/Sun/Cube has native fields");
        found.insert(property.component_type_path.clone());
        let control = row_for_field(&bridge, &property.field_id);
        assert_eq!(
            control_string(&bridge, &control, "value").as_deref(),
            Some(property.value.as_str())
        );
        assert!(!control_bool(
            &bridge,
            &control,
            "inspector_property_editable"
        ));
        assert!(!control_bool(&bridge, &control, "editable_text"));
        assert!(control_bool(&bridge, &control, "read_only"));
        assert!(!control_bool(&bridge, &control, "input_focusable"));
        assert_actual_row_input_state(&bridge, &control, false);
        let center = control_center(&bridge, &control);
        bridge
            .route_pointer_event(
                UiPointerEvent::new(UiPointerEventKind::Down, center)
                    .with_button(UiPointerButton::Primary),
            )
            .unwrap();
        assert!(
            bridge.pointer_focused_target().is_none(),
            "read-only native row must not take text focus"
        );
        bridge
            .route_pointer_event(
                UiPointerEvent::new(UiPointerEventKind::Up, center)
                    .with_button(UiPointerButton::Primary),
            )
            .unwrap();
        assert_eq!(
            bridge
                .edit_inspector_component_property(
                    &control,
                    "Inspector/ComponentProperty04Edit",
                    "999"
                )
                .unwrap(),
            Some(false)
        );
        let effects = dispatch_componentized_workbench_surface_control_edited(
            &mut bridge,
            &control,
            "Inspector/ComponentProperty04Commit",
            "999",
        )
        .unwrap()
        .unwrap();
        assert!(!effects.presentation_dirty);
        assert_eq!(
            control_string(&bridge, &control, "value").as_deref(),
            Some(property.value.as_str())
        );
        assert!(!state
            .can_edit_dynamic_component_field(&property.field_id)
            .unwrap());
        let binding = EditorUiBinding::new(
            "InspectorView",
            "ApplyBatchButton",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::inspector_field_batch(
                "entity://selected",
                vec![InspectorFieldChange::new(
                    &property.field_id,
                    UiBindingValue::string("999"),
                )],
            ),
        );
        assert!(matches!(
            crate::ui::binding_dispatch::apply_inspector_binding(&mut state, &binding),
            Err(
                crate::ui::binding_dispatch::EditorBindingDispatchError::UnsupportedInspectorField(
                    _
                )
            )
        ));
        assert!(crate::ui::binding_dispatch::apply_inspector_draft_field(
            &mut state,
            "entity://selected",
            &property.field_id,
            "999".to_string()
        )
        .is_err());
        assert_eq!(
            state.snapshot().inspector.unwrap().native_fields,
            inspector.native_fields
        );
    }
    for type_name in ["CameraComponent", "DirectionalLight", "MeshRenderer"] {
        assert!(found.contains(&format!("zircon_runtime::scene::components::{type_name}")));
    }
}

#[test]
fn two_actual_dynamic_components_keep_all_rows_and_per_component_admission() {
    let _guard = env_lock().lock().unwrap();
    let mut state = default_state();
    let selected = state.snapshot().inspector.unwrap().id;
    state.world.expect_with_world_mut(|scene| {
        for (type_id, label, value) in [
            ("audit.First", "First", 0.25),
            ("audit.Second", "Second", 0.75),
        ] {
            scene
                .register_component_type(
                    zircon_runtime::core::framework::scene::ComponentTypeDescriptor::new(
                        type_id, "audit", label,
                    )
                    .with_property("coverage", "Scalar", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected, type_id, serde_json::json!({"coverage": value}))
                .unwrap();
        }
    });
    let mut customizations = InspectorCustomizationChain::default();
    customizations
        .register(Arc::new(InspectorCustomizationDescriptor::new(
            "audit.Second",
            "res://ui/editor/inspector.zui",
            "audit.SecondInspector",
        )))
        .unwrap();
    let snapshot = state
        .snapshot_with_inspector_customizations(&customizations, &FieldEditorContainer::builtin());
    let inspector = snapshot.inspector.as_ref().unwrap();
    assert_eq!(inspector.plugin_components.len(), 2);
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, Some(inspector))
        .unwrap();
    for (type_id, editable, expected) in [
        ("audit.First", false, "0.25"),
        ("audit.Second", true, "0.75"),
    ] {
        // Filtering scrolls the real virtual source down to the named component.
        bridge
            .edit_inspector_filter(
                "WorkbenchInspectorFilter",
                "Workbench/InspectorSearchEdit",
                type_id,
            )
            .unwrap();
        let field_id = format!("{type_id}.coverage");
        let row = row_for_field(&bridge, &field_id);
        assert_actual_row_input_state(&bridge, &row, editable);
        assert_eq!(
            control_string(&bridge, &row, "value").as_deref(),
            Some(expected)
        );
        assert_eq!(
            control_bool(&bridge, &row, "inspector_property_editable"),
            editable
        );
        assert_eq!(
            bridge
                .edit_inspector_component_property(&row, "Inspector/ComponentProperty04Edit", "0.9")
                .unwrap(),
            Some(editable)
        );
    }
}
