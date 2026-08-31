use crate::core::math::{Transform, Vec3};
use crate::scene::World;
use crate::scene::components::{ActiveSelf, Hierarchy, LocalTransform, Name};

#[test]
fn persistent_entity_core_components_use_generic_storage_across_clone_serde_and_records() {
    let mut world = World::empty();
    let parent = world
        .spawn(())
        .expect("parent node should publish before entity-core components");
    let entity = world
        .spawn(())
        .expect("child node should publish before entity-core components");
    let name = Name("Runtime08 entity core".to_string());
    let hierarchy = Hierarchy {
        parent: Some(parent),
    };
    let local_transform = LocalTransform {
        transform: Transform {
            translation: Vec3::new(3.0, 2.0, 1.0),
            ..Transform::default()
        },
    };
    let active_self = ActiveSelf(false);

    world
        .insert(entity, name.clone())
        .expect("name should use component storage");
    world
        .insert(entity, hierarchy.clone())
        .expect("hierarchy should use component storage");
    world
        .insert(entity, local_transform)
        .expect("local transform should use component storage");
    world
        .insert(entity, active_self)
        .expect("active state should use component storage");

    let cloned = world.clone();
    let decoded: World = serde_json::from_str(
        &serde_json::to_string(&world)
            .expect("world persistence must serialize entity-core storage"),
    )
    .expect("world persistence must restore entity-core storage");
    for restored in [&cloned, &decoded] {
        assert_eq!(restored.get::<Name>(entity), Some(&name));
        assert_eq!(restored.get::<Hierarchy>(entity), Some(&hierarchy));
        assert_eq!(
            restored.get::<LocalTransform>(entity),
            Some(&local_transform)
        );
        assert_eq!(restored.get::<ActiveSelf>(entity), Some(&active_self));
    }

    let record = world
        .node_record(entity)
        .expect("entity-core components must project to a node record");
    world.remove_entity(entity).unwrap();
    world
        .insert_node_record(record)
        .expect("record restore should stage entity-core components before final publish");

    type EntityCoreData<'query> = (
        crate::scene::EntityId,
        &'query Name,
        &'query Hierarchy,
        &'query LocalTransform,
        &'query ActiveSelf,
    );
    let rows = world
        .query::<EntityCoreData<'static>>()
        .iter(&world)
        .map(|(entity, _, _, _, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(rows, vec![entity]);
}

#[test]
fn persistent_entity_core_components_do_not_retain_world_map_owners() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let world_source = std::fs::read_to_string(
        manifest_root
            .join("src")
            .join("scene")
            .join("world")
            .join("world.rs"),
    )
    .expect("read World storage owner source");
    let fixed_components_source = std::fs::read_to_string(
        manifest_root
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("fixed_components.rs"),
    )
    .expect("read fixed component adapter source");

    for retired_owner in [
        "pub(super) names:",
        "pub(super) hierarchy:",
        "pub(super) local_transforms:",
        "pub(super) active_self:",
    ] {
        assert!(
            !world_source.contains(retired_owner),
            "World must not retain the retired entity-core map owner: {retired_owner}"
        );
    }
    for retired_adapter in [
        "fixed_component_map!(Name, names)",
        "fixed_component_map!(Hierarchy, hierarchy)",
        "fixed_component_map!(ActiveSelf, active_self)",
        "world.local_transforms.insert(entity, *component)",
    ] {
        assert!(
            !fixed_components_source.contains(retired_adapter),
            "fixed-component adapter must not reintroduce an entity-core map owner: {retired_adapter}"
        );
    }
    assert!(
        world_source.contains("persistent_entity_core_component_snapshot"),
        "World persistence must project entity-core values from generic component storage"
    );
}
