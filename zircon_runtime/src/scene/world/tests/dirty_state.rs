use std::sync::Arc;

use crate::core::framework::render::RenderComponentValue;
use crate::core::math::{Transform, Vec3};
use crate::scene::components::WorldMatrix;
use crate::scene::{NodeKind, SystemStage, World};

#[test]
fn checked_reparent_render_stage_publishes_changed_world_matrix() {
    let mut world = World::empty();
    let first_parent = world
        .spawn_node(NodeKind::Cube)
        .expect("first parent should spawn");
    let second_parent = world
        .spawn_node(NodeKind::Cube)
        .expect("second parent should spawn");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("renderable child should spawn");

    world
        .update_transform(
            first_parent,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .expect("first parent transform should update");
    world
        .update_transform(
            second_parent,
            Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
        )
        .expect("second parent transform should update");
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .expect("child transform should update");
    world
        .set_parent_checked(child, Some(first_parent))
        .expect("initial parent should attach");
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let before = world
        .render_component_change_artifact()
        .expect("initial render projection should exist");
    let before_child = before
        .upserts()
        .iter()
        .find(|snapshot| snapshot.entity() == child)
        .expect("initial render projection should include the child");
    assert_eq!(
        before_child.world_matrix(),
        &RenderComponentValue::Present(
            world
                .get::<WorldMatrix>(child)
                .expect("child world matrix should be materialized")
                .0
        )
    );
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(3.0, 0.0, 0.0)
    );

    assert!(world
        .set_parent_checked(child, Some(second_parent))
        .expect("changed parent reparent should succeed"));
    assert!(world.has_pending_scene_systems());
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(6.0, 0.0, 0.0)
    );
    let after = world
        .render_component_change_artifact()
        .expect("changed derived state should publish a render projection");
    assert!(!Arc::ptr_eq(&before, &after));
    assert!(after.journal_generation() > before.journal_generation());
    let after_child = after
        .upserts()
        .iter()
        .find(|snapshot| snapshot.entity() == child)
        .expect("updated render projection should include the child");
    assert_eq!(
        after_child.world_matrix(),
        &RenderComponentValue::Present(
            world
                .get::<WorldMatrix>(child)
                .expect("updated child world matrix should be materialized")
                .0
        )
    );
}

#[test]
fn prepared_viewport_extract_is_idempotent_without_authoring_edits() {
    use crate::core::framework::render::SceneViewportExtractRequest;
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube).unwrap();
    world
        .update_transform(
            entity,
            Transform::from_translation(Vec3::new(2.0, 3.0, 4.0)),
        )
        .unwrap();
    let request = SceneViewportExtractRequest::default();
    let first = world.build_prepared_viewport_render_packet(&request);
    let generation = world.world_generation();
    let artifact = world.render_component_change_artifact().unwrap();
    let second = world.build_prepared_viewport_render_packet(&request);
    assert_eq!(first, second);
    assert_eq!(world.world_generation(), generation);
    assert!(Arc::ptr_eq(
        &artifact,
        &world.render_component_change_artifact().unwrap()
    ));
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn memory_profile_prepared_viewport_packet_10k_entities() {
    use crate::core::framework::render::SceneViewportExtractRequest;
    let mut world = World::empty();
    for _ in 0..10_000 {
        world.spawn_node(NodeKind::Empty).unwrap();
    }
    world.prepare_viewport_render_extract();
    let request = SceneViewportExtractRequest::default();
    let (_, prepared) =
        crate::memory_validation::measure(|| world.build_prepared_viewport_render_packet(&request));
    prepared.report("viewport_packet_10k", "prepared");
    let (_, cloning) =
        crate::memory_validation::measure(|| world.build_viewport_render_packet_cloning(&request));
    cloning.report("viewport_packet_10k", "cloning");
    assert!(prepared.allocations < cloning.allocations);
    assert!(prepared.peak_bytes < cloning.peak_bytes);
}
