use crate::scene::components::{CameraComponent, MeshRenderer, Mobility, RenderLayerMask};
use crate::scene::World;

#[test]
fn persistent_scene_render_components_use_generic_storage_across_clone_serde_and_records() {
    let mut world = World::empty();
    let entity = world
        .spawn(())
        .expect("default node should publish before render components");
    let camera = CameraComponent::default();
    let mesh = MeshRenderer::default();
    let render_layer_mask = RenderLayerMask(0x0f);
    let mobility = Mobility::Static;

    world
        .insert(entity, camera.clone())
        .expect("camera should use component storage");
    world
        .insert(entity, mesh.clone())
        .expect("mesh renderer should use component storage");
    world
        .insert(entity, render_layer_mask)
        .expect("render layer mask should use component storage");
    world
        .insert(entity, mobility)
        .expect("mobility should use component storage");

    let cloned = world.clone();
    let decoded: World = serde_json::from_str(
        &serde_json::to_string(&world)
            .expect("world persistence must serialize scene-render storage"),
    )
    .expect("world persistence must restore scene-render storage");
    for restored in [&cloned, &decoded] {
        assert_eq!(restored.get::<CameraComponent>(entity), Some(&camera));
        assert_eq!(restored.get::<MeshRenderer>(entity), Some(&mesh));
        assert_eq!(
            restored.get::<RenderLayerMask>(entity),
            Some(&render_layer_mask)
        );
        assert_eq!(restored.get::<Mobility>(entity), Some(&mobility));
    }

    let record = world
        .node_record(entity)
        .expect("scene-render components must project to a node record");
    world.remove_entity(entity).unwrap();
    world
        .insert_node_record(record)
        .expect("record restore should stage scene-render components before final publish");

    type SceneRenderData<'query> = (
        crate::scene::EntityId,
        &'query CameraComponent,
        &'query MeshRenderer,
        &'query RenderLayerMask,
        &'query Mobility,
    );
    let rows = world
        .query::<SceneRenderData<'static>>()
        .iter(&world)
        .map(|(entity, _, _, _, _)| entity)
        .collect::<Vec<_>>();
    assert_eq!(rows, vec![entity]);
}

#[test]
fn persistent_scene_render_components_do_not_retain_world_map_owners() {
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
        "pub(super) cameras:",
        "pub(super) mesh_renderers:",
        "pub(super) render_layer_masks:",
        "pub(super) mobility:",
    ] {
        assert!(
            !world_source.contains(retired_owner),
            "World must not retain the retired scene-render map owner: {retired_owner}"
        );
    }
    for retired_adapter in [
        "fixed_component_map!(CameraComponent, cameras)",
        "fixed_component_map!(MeshRenderer, mesh_renderers)",
        "fixed_component_map!(RenderLayerMask, render_layer_masks)",
        "fixed_component_map!(Mobility, mobility)",
    ] {
        assert!(
            !fixed_components_source.contains(retired_adapter),
            "fixed-component adapter must not retain a scene-render map owner: {retired_adapter}"
        );
    }
    assert!(
        world_source.contains("persistent_scene_render_component_snapshot"),
        "World persistence must project scene-render values from generic component storage"
    );
}
