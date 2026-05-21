use serde_json::json;
use zircon_runtime::plugin::ComponentTypeDescriptor;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::{SceneInspectorFieldValue, SceneViewportController};

const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";

#[test]
fn viewport_edit_mode_projection_consumes_runtime_reflection_inspector_fields() {
    let mut scene = Scene::empty();
    scene
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic component descriptor should register");
    let entity = scene.spawn_node(NodeKind::Mesh);
    scene
        .rename_node(entity, "Cloud")
        .expect("test entity should be named");
    scene
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.75, "label": "storm front" }),
        )
        .expect("dynamic component should attach");

    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.set_selected_node(Some(entity));

    let projection = controller.build_edit_mode_projection(&scene);

    assert!(projection.inspector_fields.iter().any(|field| {
        field.property_path.as_deref() == Some("Name.value")
            && field.value == SceneInspectorFieldValue::Text("Cloud".to_string())
            && field.editable
    }));
    assert!(projection.inspector_fields.iter().any(|field| {
        field.property_path.as_deref() == Some("Transform.translation")
            && matches!(field.value, SceneInspectorFieldValue::Vec3(_))
            && field.editable
    }));
    assert!(projection.inspector_fields.iter().any(|field| {
        field.property_path.as_deref() == Some("MeshRenderer.model")
            && matches!(field.value, SceneInspectorFieldValue::Resource(_))
            && !field.editable
    }));
    assert!(projection.inspector_fields.iter().any(|field| {
        field.component == "Cloud Layer"
            && field.label == "Coverage"
            && field.property_path.as_deref() == Some("weather.Component.CloudLayer.coverage")
            && field.value == SceneInspectorFieldValue::Scalar(0.75)
            && field.editable
    }));
    assert!(projection.inspector_fields.iter().any(|field| {
        field.component == "Cloud Layer"
            && field.label == "Label"
            && field.property_path.as_deref() == Some("weather.Component.CloudLayer.label")
            && field.value == SceneInspectorFieldValue::Text("storm front".to_string())
            && !field.editable
    }));
}

#[test]
fn viewport_edit_mode_projection_exposes_ambient_and_rect_light_fields() {
    let mut scene = Scene::empty();
    let ambient = scene.spawn_node(NodeKind::AmbientLight);
    let rect = scene.spawn_node(NodeKind::RectLight);

    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.set_selected_node(Some(ambient));

    let ambient_projection = controller.build_edit_mode_projection(&scene);
    assert_eq!(ambient_projection.stats.light_count, 2);
    assert!(ambient_projection.inspector_fields.iter().any(|field| {
        field.component == "Ambient Light"
            && field.label == "Intensity"
            && field.property_path.as_deref() == Some("AmbientLight.intensity")
            && field.value == SceneInspectorFieldValue::Scalar(80.0)
            && field.editable
    }));
    assert!(ambient_projection.inspector_fields.iter().any(|field| {
        field.component == "Ambient Light"
            && field.label == "Affects Lightmapped Meshes"
            && field.property_path.as_deref() == Some("AmbientLight.affects_lightmapped_meshes")
            && field.value == SceneInspectorFieldValue::Bool(true)
            && field.editable
    }));

    controller.set_selected_node(Some(rect));

    let rect_projection = controller.build_edit_mode_projection(&scene);
    assert!(rect_projection.inspector_fields.iter().any(|field| {
        field.component == "Rect Light"
            && field.label == "Range"
            && field.property_path.as_deref() == Some("RectLight.range")
            && field.value == SceneInspectorFieldValue::Scalar(20.0)
            && field.editable
    }));
    assert!(rect_projection.inspector_fields.iter().any(|field| {
        field.component == "Rect Light"
            && field.label == "Size"
            && field.property_path.as_deref() == Some("RectLight.size")
            && field.value == SceneInspectorFieldValue::Vec2([1.0, 1.0])
            && field.editable
    }));
}

fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}
