use super::InspectorNativeFieldSnapshot;
use zircon_runtime::scene::Scene;

#[test]
fn native_fields_use_actual_registered_default_scene_components() {
    let scene = Scene::new();
    let mut types = std::collections::BTreeSet::new();
    for node in scene.nodes() {
        let artifact = scene.inspection_fields_artifact(node.id).unwrap();
        let projected = InspectorNativeFieldSnapshot::project(artifact.fields());
        for property in projected {
            let source = artifact
                .fields()
                .iter()
                .find(|field| {
                    field.component_type_path == property.component_type_path
                        && field.field_name == property.name
                })
                .unwrap();
            assert!(!source.plugin_owned);
            assert_eq!(
                property.value,
                super::super::super::editor_state_snapshot_build::reflected_value_label(
                    &source.value
                )
            );
            assert_eq!(
                property.field_id,
                format!("{}.{}", source.component_type_path, source.field_name)
            );
            types.insert(property.component_type_path);
        }
    }
    for native_type in ["CameraComponent", "DirectionalLight", "MeshRenderer"] {
        assert!(types.contains(&format!("zircon_runtime::scene::components::{native_type}")));
    }
    assert!(!InspectorNativeFieldSnapshot::is_component_type(
        "zircon_runtime::scene::components::LocalTransform"
    ));
    assert!(!InspectorNativeFieldSnapshot::is_component_type(
        "zircon_runtime::scene::components::RenderLayerMask"
    ));
}
