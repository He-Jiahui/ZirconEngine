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
