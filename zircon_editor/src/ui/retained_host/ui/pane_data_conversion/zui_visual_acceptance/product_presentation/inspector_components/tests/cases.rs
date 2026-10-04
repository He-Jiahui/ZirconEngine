use crate::core::editing::intent::EditorIntent;
use crate::core::extension::{
    FieldEditorContainer, InspectorCustomizationChain, InspectorCustomizationDescriptor,
};
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::state::EditorState;
use std::sync::Arc;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;

#[test]
fn actual_selected_native_fields_survive_product_inspector_json() {
    let mut state = EditorState::with_default_selection(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
    );
    state.mark_project_open();
    let entities = state
        .world
        .expect_with_world(|scene| scene.nodes().iter().map(|node| node.id).collect::<Vec<_>>());
    let mut types = std::collections::BTreeSet::new();
    for entity in entities {
        state
            .apply_intent(EditorIntent::SelectNode(entity))
            .unwrap();
        let chrome = EditorChromeSnapshot::build(
            state.snapshot(),
            &WorkbenchLayout::default(),
            Vec::new(),
            Vec::new(),
            None,
        );
        let source = chrome.inspector.as_ref().unwrap();
        assert!(source.plugin_components.is_empty());
        let value =
            serde_json::to_value(super::super::inspector_snapshot(&chrome).unwrap()).unwrap();
        let components = value["components"].as_array().unwrap();
        assert!(!components.is_empty());
        for field in &source.native_fields {
            let component = components
                .iter()
                .find(|component| component["id"] == field.component_type_path)
                .expect("actual native component must survive serialized product projection");
            let property = component["properties"]
                .as_array()
                .unwrap()
                .iter()
                .find(|property| property["id"] == field.field_id)
                .unwrap();
            assert_eq!(component["title"], field.component_label);
            assert_eq!(property["label"], field.label);
            assert_eq!(property["value"], field.value);
            assert_eq!(property["kind"], field.value_kind);
            assert_eq!(property["editable"], false);
            types.insert(field.component_type_path.clone());
        }
        assert_eq!(
            components
                .iter()
                .map(|component| component["properties"].as_array().unwrap().len())
                .sum::<usize>(),
            source.native_fields.len()
        );
    }
    for name in ["CameraComponent", "DirectionalLight", "MeshRenderer"] {
        assert!(types.contains(&format!("zircon_runtime::scene::components::{name}")));
    }
}

#[test]
fn actual_multiple_dynamic_components_survive_neutral_product_json_with_admission() {
    let mut state = EditorState::with_default_selection(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
    );
    state.mark_project_open();
    let selected = state.snapshot().inspector.unwrap().id;
    state.world.expect_with_world_mut(|scene| {
        for (id, label, value) in [
            ("plugin.audit.First", "First", 0.25),
            ("plugin.audit.Second", "Second", 0.75),
        ] {
            scene
                .register_component_type(
                    zircon_runtime::core::framework::scene::ComponentTypeDescriptor::new(
                        id, "audit", label,
                    )
                    .with_property("coverage", "Scalar", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected, id, serde_json::json!({"coverage":value}))
                .unwrap();
        }
    });
    let mut customizations = InspectorCustomizationChain::default();
    customizations
        .register(Arc::new(InspectorCustomizationDescriptor::new(
            "plugin.audit.Second",
            "res://ui/editor/inspector.zui",
            "audit.SecondInspector",
        )))
        .unwrap();
    let data = state
        .snapshot_with_inspector_customizations(&customizations, &FieldEditorContainer::builtin());
    let chrome = EditorChromeSnapshot::build(
        data,
        &WorkbenchLayout::default(),
        Vec::new(),
        Vec::new(),
        None,
    );
    let source = chrome.inspector.as_ref().unwrap();
    let value = serde_json::to_value(super::super::inspector_snapshot(&chrome).unwrap()).unwrap();
    let components = value["components"].as_array().unwrap();
    assert_eq!(source.plugin_components.len(), 2);
    for component in &source.plugin_components {
        let serialized = components
            .iter()
            .find(|entry| entry["id"] == component.component_id)
            .unwrap();
        for field in &component.properties {
            let serialized_field = serialized["properties"]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["id"] == field.field_id)
                .unwrap();
            assert_eq!(serialized_field["value"], field.value);
            assert_eq!(
                serialized_field["editable"],
                field.editable && component.customization_available
            );
        }
    }
    assert!(!source.native_fields.is_empty());
    assert!(components.iter().any(|entry| source
        .native_fields
        .iter()
        .any(|field| entry["id"] == field.component_type_path)));
}
