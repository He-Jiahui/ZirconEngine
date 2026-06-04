use serde_json::json;
use zircon_runtime::core::framework::navigation::NAV_MESH_SURFACE_COMPONENT_TYPE;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::world::World;

use crate::navigation_component_descriptors;

#[test]
fn navigation_dynamic_component_descriptor_accepts_vec_and_resource_json() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .register_component_type(navigation_component_descriptors()[0].clone())
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "volume_center": [1.0, 2.0, 3.0],
                "output_asset": {"resource": "res://navigation/level.znavmesh"}
            }),
        )
        .unwrap();

    assert!(world
        .dynamic_component(entity, NAV_MESH_SURFACE_COMPONENT_TYPE)
        .is_some());
}
