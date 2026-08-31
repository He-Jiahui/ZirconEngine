use crate::scene::World;
use crate::scene::components::{AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight};

#[test]
fn persistent_lights_use_generic_storage_across_clone_serde_and_records() {
    let mut world = World::empty();
    let entity = world
        .spawn(())
        .expect("default node should publish before lighting components");
    let ambient = AmbientLight::default();
    let directional = DirectionalLight::default();
    let point = PointLight::default();
    let rect = RectLight::default();
    let spot = SpotLight::default();

    world
        .insert(entity, ambient.clone())
        .expect("ambient light should use component storage");
    world
        .insert(entity, directional.clone())
        .expect("directional light should use component storage");
    world
        .insert(entity, point.clone())
        .expect("point light should use component storage");
    world
        .insert(entity, rect.clone())
        .expect("rect light should use component storage");
    world
        .insert(entity, spot.clone())
        .expect("spot light should use component storage");

    let cloned = world.clone();
    let decoded: World = serde_json::from_str(
        &serde_json::to_string(&world).expect("world persistence must serialize lighting storage"),
    )
    .expect("world persistence must restore lighting storage");
    for restored in [&cloned, &decoded] {
        assert_eq!(restored.get::<AmbientLight>(entity), Some(&ambient));
        assert_eq!(restored.get::<DirectionalLight>(entity), Some(&directional));
        assert_eq!(restored.get::<PointLight>(entity), Some(&point));
        assert_eq!(restored.get::<RectLight>(entity), Some(&rect));
        assert_eq!(restored.get::<SpotLight>(entity), Some(&spot));
    }

    let record = world
        .node_record(entity)
        .expect("lighting components must project to a node record");
    world.remove_entity(entity).unwrap();
    world
        .insert_node_record(record)
        .expect("record restore should stage lighting before final publish");

    type LightingData<'query> = (
        crate::scene::EntityId,
        &'query AmbientLight,
        &'query DirectionalLight,
        &'query PointLight,
        &'query RectLight,
        &'query SpotLight,
    );
    let rows = world
        .query::<LightingData<'static>>()
        .iter(&world)
        .map(|(entity, _, _, _, _, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(rows, vec![entity]);
}

#[test]
fn persistent_lights_do_not_retain_world_map_owners() {
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
        "pub(super) ambient_lights:",
        "pub(super) directional_lights:",
        "pub(super) point_lights:",
        "pub(super) rect_lights:",
        "pub(super) spot_lights:",
    ] {
        assert!(
            !world_source.contains(retired_owner),
            "World must not retain the retired light component map owner: {retired_owner}"
        );
    }
    for retired_adapter in [
        "fixed_component_map!(AmbientLight, ambient_lights)",
        "fixed_component_map!(DirectionalLight, directional_lights)",
        "fixed_component_map!(PointLight, point_lights)",
        "fixed_component_map!(RectLight, rect_lights)",
        "fixed_component_map!(SpotLight, spot_lights)",
    ] {
        assert!(
            !fixed_components_source.contains(retired_adapter),
            "fixed-component adapter must not reintroduce a light map owner: {retired_adapter}"
        );
    }
    assert!(
        world_source.contains("persistent_lighting_component_snapshot"),
        "World persistence must project lighting values from generic component storage"
    );
}
