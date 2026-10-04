use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::World;

use super::{node_pose_transform_updates, AnimationEvaluationPipeline};

#[test]
fn node_pose_updates_named_descendants_without_touching_root_or_outsiders() {
    let mut world = World::new();
    let actor = world.spawn_node(NodeKind::Empty);
    world.rename_node(actor, "Actor").unwrap();

    let torso = world.spawn_node(NodeKind::Mesh);
    world.rename_node(torso, "Node2:torso").unwrap();
    world.set_parent_checked(torso, Some(actor)).unwrap();

    let arm = world.spawn_node(NodeKind::Mesh);
    world.rename_node(arm, "arm-right").unwrap();
    world.set_parent_checked(arm, Some(torso)).unwrap();

    let outsider = world.spawn_node(NodeKind::Mesh);
    world.rename_node(outsider, "Node2:torso").unwrap();

    let mut actor_transform = Transform::default();
    actor_transform.translation = Vec3::new(5.0, 0.0, -2.0);
    world.update_transform(actor, actor_transform).unwrap();

    let mut torso_pose = Transform::default();
    torso_pose.translation = Vec3::new(0.0, 0.25, 0.0);
    let mut arm_pose = Transform::default();
    arm_pose.translation = Vec3::new(-0.15, 0.3, 0.02);

    let poses = BTreeMap::from([(
        actor,
        AnimationPoseOutput {
            source: AnimationPoseSource::StateMachine,
            active_state: Some("Move".to_string()),
            bones: vec![
                AnimationPoseBone {
                    name: "Node2:torso".to_string(),
                    local_transform: torso_pose,
                },
                AnimationPoseBone {
                    name: "Node3:arm-right".to_string(),
                    local_transform: arm_pose,
                },
            ],
        },
    )]);

    let mut pipeline = AnimationEvaluationPipeline::default();
    pipeline.cache_pose_target_binding(world.compile_descendant_name_index(actor).unwrap());
    let updates = node_pose_transform_updates(&pipeline, &poses);
    for (entity, transform) in updates {
        world.update_transform(entity, transform).unwrap();
    }

    assert_eq!(world.find_node(actor).unwrap().transform, actor_transform);
    assert_eq!(world.find_node(torso).unwrap().transform, torso_pose);
    assert_eq!(world.find_node(arm).unwrap().transform, arm_pose);
    assert_eq!(
        world.find_node(outsider).unwrap().transform,
        Transform::default()
    );
}
