use super::{
    SceneComponentAssetRecord, SceneComponentSerializer, SceneComponentSerializerRegistry,
    SceneProjectError, World,
};
use crate::asset::project::ProjectManager;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::components::NodeRecord;
use serde_json::Value;

fn noop_capture(
    _project: &ProjectManager,
    _world: &World,
    _record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    Ok(None)
}

fn noop_instantiate(
    _project: &ProjectManager,
    _row: &SceneComponentAssetRecord,
    _record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    Ok(None)
}

#[test]
fn builtin_scene_component_registry_routes_stable_identity() {
    assert_eq!(
        SceneComponentSerializerRegistry::builtin().registered_type_ids(),
        vec![super::SPRITE_TYPE_ID, super::MESH_TYPE_ID]
    );
}

#[test]
fn descriptor_installation_is_atomic_across_provider_batch() {
    let mut registry = SceneComponentSerializerRegistry::builtin();
    registry
        .register_provider(
            SceneComponentSerializer {
                type_id: "tests.scene.atomic.good",
                schema_id: "tests.scene.atomic.good.v1",
                schema_version: 1,
                provider_id: "tests.scene.atomic",
                capture: noop_capture,
                instantiate: noop_instantiate,
            },
            ComponentTypeDescriptor::new("tests.scene.atomic.good", "tests.scene.atomic", "Good")
                .with_property("value", "Scalar", true),
        )
        .unwrap();
    registry
        .register_provider(
            SceneComponentSerializer {
                type_id: "tests.scene.atomic.bad",
                schema_id: "tests.scene.atomic.bad.v1",
                schema_version: 1,
                provider_id: "tests.scene.atomic",
                capture: noop_capture,
                instantiate: noop_instantiate,
            },
            ComponentTypeDescriptor::new("tests.scene.atomic.bad", "tests.scene.atomic", "Bad")
                .with_property("value", "Scalar", true)
                .with_property("value", "Scalar", true),
        )
        .unwrap();

    let mut world = World::empty();
    assert!(registry.install_into_world(&mut world).is_err());
    assert!(world
        .component_type_descriptor("tests.scene.atomic.good")
        .is_none());
    assert!(world
        .registered_dynamic_component_id("tests.scene.atomic.good")
        .is_none());
    assert!(world
        .component_type_descriptor("tests.scene.atomic.bad")
        .is_none());
}
