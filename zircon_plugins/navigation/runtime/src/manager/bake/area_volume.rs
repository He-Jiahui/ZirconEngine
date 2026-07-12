use zircon_runtime::core::framework::navigation::{NavAreaId, NavMeshModifierMode};
use zircon_runtime::core::math::Vec3;
use zircon_runtime::scene::components::{NodeKind, SceneNode};
use zircon_runtime::scene::World;

use super::modifier::direct_modifier;

#[derive(Clone, Copy, Debug)]
pub(super) struct BakeAreaVolume {
    entity: u64,
    center: Vec3,
    half_extents: Vec3,
    area: NavAreaId,
}

impl BakeAreaVolume {
    fn contains(self, entity: u64, position: Vec3) -> bool {
        if self.entity == entity {
            return false;
        }
        let delta = (position - self.center).abs();
        delta.x <= self.half_extents.x
            && delta.y <= self.half_extents.y
            && delta.z <= self.half_extents.z
    }
}

pub(super) fn collect_area_volumes(world: &World, agent_type: &str) -> Vec<BakeAreaVolume> {
    world
        .node_records()
        .into_iter()
        .filter(|node| matches!(node.kind, NodeKind::Empty))
        .filter_map(|node| {
            let modifier = direct_modifier(world, node.id, agent_type)?;
            if !modifier.override_area || matches!(modifier.mode, NavMeshModifierMode::Remove) {
                return None;
            }
            let transform = world.world_transform(node.id).unwrap_or(node.transform);
            Some(BakeAreaVolume {
                entity: node.id,
                center: transform.translation,
                half_extents: transform.scale.abs() * 0.5,
                area: modifier.area,
            })
        })
        .collect()
}

pub(super) fn volume_area_override(
    world: &World,
    node: &SceneNode,
    volumes: &[BakeAreaVolume],
) -> Option<NavAreaId> {
    let position = world
        .world_transform(node.id)
        .map(|transform| transform.translation)
        .unwrap_or(node.transform.translation);
    volumes
        .iter()
        .copied()
        .find(|volume| volume.contains(node.id, position))
        .map(|volume| volume.area)
}
