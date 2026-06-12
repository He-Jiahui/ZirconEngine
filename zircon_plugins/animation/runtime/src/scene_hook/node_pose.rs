use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::math::Transform;
use zircon_runtime::scene::components::SceneNode;
use zircon_runtime::scene::{EntityId, LevelSystem, World};

pub(super) fn apply_pose_transforms_to_scene_nodes(
    level: &LevelSystem,
    poses: &BTreeMap<EntityId, AnimationPoseOutput>,
) {
    if poses.is_empty() {
        return;
    }

    level.with_world_mut(|world| {
        let updates = node_pose_transform_updates(world, poses);
        for (entity, transform) in updates {
            let _ = world.update_transform(entity, transform);
        }
    });
}

fn node_pose_transform_updates(
    world: &World,
    poses: &BTreeMap<EntityId, AnimationPoseOutput>,
) -> Vec<(EntityId, Transform)> {
    let nodes = world.node_records();
    let parent_by_entity = nodes
        .iter()
        .map(|node| (node.id, node.parent))
        .collect::<BTreeMap<_, _>>();
    let mut updates = Vec::new();

    for (root, pose) in poses {
        for bone in &pose.bones {
            if let Some(entity) =
                find_descendant_pose_target(*root, &bone.name, &nodes, &parent_by_entity)
            {
                updates.push((entity, bone.local_transform));
            }
        }
    }

    updates
}

fn find_descendant_pose_target(
    root: EntityId,
    bone_name: &str,
    nodes: &[SceneNode],
    parent_by_entity: &BTreeMap<EntityId, Option<EntityId>>,
) -> Option<EntityId> {
    let candidate_names = pose_target_names(bone_name);
    for candidate in &candidate_names {
        if let Some(node) = nodes.iter().find(|node| {
            node.name == *candidate
                && node.id != root
                && is_descendant_of(node.id, root, parent_by_entity)
        }) {
            return Some(node.id);
        }
    }

    for candidate in &candidate_names {
        if let Some(node) = nodes.iter().find(|node| {
            short_node_name(&node.name) == *candidate
                && node.id != root
                && is_descendant_of(node.id, root, parent_by_entity)
        }) {
            return Some(node.id);
        }
    }

    None
}

fn pose_target_names(bone_name: &str) -> Vec<&str> {
    let trimmed = bone_name.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let path_tail = trimmed.rsplit('/').next().unwrap_or(trimmed);
    let short_name = short_node_name(path_tail);
    let mut names = vec![trimmed, path_tail, short_name];
    names.dedup();
    names
}

fn short_node_name(name: &str) -> &str {
    name.rsplit_once(':')
        .map(|(_, short)| short.trim())
        .unwrap_or(name.trim())
}

fn is_descendant_of(
    entity: EntityId,
    root: EntityId,
    parent_by_entity: &BTreeMap<EntityId, Option<EntityId>>,
) -> bool {
    let mut current = Some(entity);
    let mut depth = 0usize;
    while let Some(entity) = current {
        if entity == root {
            return true;
        }
        depth += 1;
        if depth > parent_by_entity.len() {
            return false;
        }
        current = parent_by_entity.get(&entity).copied().flatten();
    }
    false
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zircon_runtime::core::framework::animation::{
        AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
    };
    use zircon_runtime::core::math::{Transform, Vec3};
    use zircon_runtime::scene::components::NodeKind;
    use zircon_runtime::scene::World;

    use super::node_pose_transform_updates;

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

        let updates = node_pose_transform_updates(&world, &poses);
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
}
