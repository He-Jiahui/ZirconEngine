use serde_json::json;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::plugin::ComponentTypeDescriptor;
use crate::scene::{NodeKind, World};

const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";
const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
const MESH_RENDERER_TYPE_PATH: &str = "zircon_runtime::scene::components::MeshRenderer";

#[test]
fn editor_projection_builds_hierarchy_and_reflected_inspector_fields() {
    let mut world = World::empty();
    world
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic descriptor should register");
    let parent = world.spawn_node(NodeKind::Mesh);
    let child = world.spawn_node(NodeKind::Mesh);
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

    let projection = world.editor_projection(Some(child));

    assert_eq!(projection.selected_entity, Some(child));
    assert_eq!(projection.hierarchy_rows.len(), 2);
    assert_eq!(projection.hierarchy_rows[0].entity, parent);
    assert_eq!(projection.hierarchy_rows[0].depth, 0);
    assert!(projection.hierarchy_rows[0].has_children);
    assert!(!projection.hierarchy_rows[0].selected);
    assert_eq!(projection.hierarchy_rows[1].entity, child);
    assert_eq!(projection.hierarchy_rows[1].depth, 1);
    assert!(projection.hierarchy_rows[1].selected);

    let name = projection
        .inspector_fields
        .iter()
        .find(|field| field.component_type_path == NAME_TYPE_PATH && field.field_name == "value")
        .expect("fixed Name field should be reflected into inspector");
    assert_eq!(name.value, ReflectedValue::String("Cloud".to_string()));
    assert!(name.editable);
    assert!(!name.plugin_owned);

    let mesh_model = projection
        .inspector_fields
        .iter()
        .find(|field| {
            field.component_type_path == MESH_RENDERER_TYPE_PATH && field.field_name == "model"
        })
        .expect("fixed MeshRenderer resource fields should be reflected into inspector");
    assert!(matches!(mesh_model.value, ReflectedValue::Resource(_)));
    assert!(!mesh_model.editable);
    assert!(!mesh_model.plugin_owned);

    let coverage = projection
        .inspector_fields
        .iter()
        .find(|field| {
            field.component_type_path == CLOUD_LAYER_TYPE_PATH && field.field_name == "coverage"
        })
        .expect("dynamic component coverage field should be reflected into inspector");
    assert_eq!(coverage.value, ReflectedValue::Scalar(0.75));
    assert_eq!(coverage.component_display_name, "Cloud Layer");
    assert_eq!(coverage.value_type_path, "Scalar");
    assert!(coverage.editable);
    assert!(coverage.plugin_owned);

    let label = projection
        .inspector_fields
        .iter()
        .find(|field| {
            field.component_type_path == CLOUD_LAYER_TYPE_PATH && field.field_name == "label"
        })
        .expect("dynamic component read-only label field should be reflected into inspector");
    assert_eq!(
        label.value,
        ReflectedValue::String("storm front".to_string())
    );
    assert!(!label.editable);
    assert!(label.plugin_owned);
}

#[test]
fn editor_projection_filters_missing_selection_without_storing_editor_state() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);

    let projection = world.editor_projection(Some(entity + 100));

    assert_eq!(projection.selected_entity, None);
    assert_eq!(projection.hierarchy_rows.len(), 1);
    assert!(projection.inspector_fields.is_empty());
    assert!(world.contains_entity(entity));
}

fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}
