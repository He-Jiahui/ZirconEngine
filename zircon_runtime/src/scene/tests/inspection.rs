use serde_json::json;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::{NodeKind, World};

use super::authoring_boundary::{
    SERIALIZED_AUTHORING_TOKENS, assert_text_excludes_authoring_tokens,
};

const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";
const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
const MESH_RENDERER_TYPE_PATH: &str = "zircon_runtime::scene::components::MeshRenderer";

#[test]
fn world_inspection_artifacts_build_hierarchy_and_reflected_fields() {
    let mut world = World::empty();
    world
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic descriptor should register");
    let parent = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .rename_node(parent, "Weather Root")
        .expect("parent should be named");
    world
        .rename_node(child, "Cloud")
        .expect("child should be named");
    world
        .set_parent_checked(child, Some(parent))
        .expect("child should be parented");
    world
        .set_dynamic_component(
            child,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.75, "label": "storm front" }),
        )
        .expect("dynamic component should attach");

    let hierarchy = world.inspection_artifact();
    let fields = world
        .inspection_fields_artifact(child)
        .expect("existing node should project reflected fields");

    assert_eq!(hierarchy.hierarchy_rows().len(), 2);
    assert_eq!(hierarchy.hierarchy_rows()[0].entity, parent);
    assert_eq!(hierarchy.hierarchy_rows()[0].depth, 0);
    assert!(hierarchy.hierarchy_rows()[0].has_children);
    assert_eq!(hierarchy.hierarchy_rows()[1].entity, child);
    assert_eq!(hierarchy.hierarchy_rows()[1].depth, 1);

    let name = fields
        .fields()
        .iter()
        .find(|field| field.component_type_path == NAME_TYPE_PATH && field.field_name == "value")
        .expect("fixed Name field should be reflected into inspection");
    assert_eq!(name.value, ReflectedValue::String("Cloud".to_string()));
    assert!(name.writable);
    assert!(!name.plugin_owned);

    let mesh_model = fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path == MESH_RENDERER_TYPE_PATH && field.field_name == "model"
        })
        .expect("fixed MeshRenderer resource fields should be reflected into inspection");
    assert!(matches!(mesh_model.value, ReflectedValue::Resource(_)));
    assert!(!mesh_model.writable);
    assert!(!mesh_model.plugin_owned);

    let mesh_order = fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path == MESH_RENDERER_TYPE_PATH
                && field.field_name == "order_in_layer"
        })
        .expect("fixed MeshRenderer order should be reflected into inspection");
    assert_eq!(mesh_order.value, ReflectedValue::Integer(0));
    assert!(mesh_order.writable);
    assert!(!mesh_order.plugin_owned);

    let mesh_render_queue = fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path == MESH_RENDERER_TYPE_PATH
                && field.field_name == "render_queue"
        })
        .expect("fixed MeshRenderer render queue should be reflected into inspection");
    assert_eq!(mesh_render_queue.value, ReflectedValue::Integer(0));
    assert!(mesh_render_queue.writable);
    assert!(!mesh_render_queue.plugin_owned);

    let mesh_material_queue = fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path == MESH_RENDERER_TYPE_PATH
                && field.field_name == "material_queue"
        })
        .expect("fixed MeshRenderer material queue should be reflected into inspection");
    assert_eq!(mesh_material_queue.value, ReflectedValue::Integer(0));
    assert!(mesh_material_queue.writable);
    assert!(!mesh_material_queue.plugin_owned);

    let mesh_depth_bias = fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path == MESH_RENDERER_TYPE_PATH && field.field_name == "depth_bias"
        })
        .expect("fixed MeshRenderer depth bias should be reflected into inspection");
    assert_eq!(mesh_depth_bias.value, ReflectedValue::Scalar(0.0));
    assert!(mesh_depth_bias.writable);
    assert!(!mesh_depth_bias.plugin_owned);

    let coverage = fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path == CLOUD_LAYER_TYPE_PATH && field.field_name == "coverage"
        })
        .expect("dynamic component coverage field should be reflected into inspection");
    assert_eq!(coverage.value, ReflectedValue::Scalar(0.75));
    assert_eq!(coverage.component_display_name, "Cloud Layer");
    assert_eq!(coverage.value_type_path, "Scalar");
    assert!(coverage.writable);
    assert!(coverage.plugin_owned);

    let label = fields
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path == CLOUD_LAYER_TYPE_PATH && field.field_name == "label"
        })
        .expect("dynamic component read-only label field should be reflected into inspection");
    assert_eq!(
        label.value,
        ReflectedValue::String("storm front".to_string())
    );
    assert!(!label.writable);
    assert!(label.plugin_owned);
}

#[test]
fn world_inspection_artifacts_reject_missing_entity_fields_without_authoring_state() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");

    let hierarchy = world.inspection_artifact();

    assert_eq!(hierarchy.hierarchy_rows().len(), 1);
    assert!(world.inspection_fields_artifact(entity + 100).is_none());
    assert!(world.contains_entity(entity));
}

#[test]
fn world_inspection_hierarchy_serialization_excludes_editor_authoring_tokens() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .rename_node(entity, "Runtime Mesh")
        .expect("entity should be named");

    let inspection = world.inspection_artifact();
    let serialized = serde_json::to_string(inspection.hierarchy_rows())
        .expect("inspection hierarchy should serialize");

    assert_text_excludes_authoring_tokens(
        "world inspection serialization",
        &serialized,
        SERIALIZED_AUTHORING_TOKENS,
    );
}

fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}
