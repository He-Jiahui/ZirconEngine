use zircon_math::Transform;

use super::World;
use crate::components::{NodeKind, NodeRecord, SceneNode, WorldTransform};
use crate::EntityId;

impl World {
    pub(super) fn ordinal_for(&self, kind: NodeKind) -> usize {
        self.entities
            .iter()
            .filter(|entity| self.node_kind(**entity) == Some(kind.clone()))
            .count()
            + 1
    }

    pub(super) fn node_kind(&self, entity: EntityId) -> Option<NodeKind> {
        self.kinds.get(&entity).cloned()
    }

    pub(super) fn rebuild_derived_state(&mut self) {
        self.rebuild_world_transforms();
        self.refresh_node_cache();
    }

    pub(super) fn collect_subtree_records(&self, entity: EntityId, records: &mut Vec<NodeRecord>) {
        let Some(record) = self.node_record(entity) else {
            return;
        };
        records.push(record);
        let children: Vec<_> = self
            .hierarchy
            .iter()
            .filter_map(|(child, hierarchy)| (hierarchy.parent == Some(entity)).then_some(*child))
            .collect();
        for child in children {
            self.collect_subtree_records(child, records);
        }
    }

    pub(super) fn is_descendant(&self, entity: EntityId, ancestor: EntityId) -> bool {
        let mut cursor = Some(entity);
        while let Some(current) = cursor {
            if current == ancestor {
                return true;
            }
            cursor = self.parent_of(current);
        }
        false
    }

    fn rebuild_world_transforms(&mut self) {
        self.world_transforms.clear();
        let roots: Vec<_> = self
            .entities
            .iter()
            .copied()
            .filter(|entity| {
                self.hierarchy
                    .get(entity)
                    .and_then(|hierarchy| hierarchy.parent)
                    .is_none_or(|parent| !self.entities.contains(&parent))
            })
            .collect();
        for root in roots {
            self.propagate_world_transform(root, Transform::identity());
        }
    }

    fn propagate_world_transform(&mut self, entity: EntityId, parent_world: Transform) {
        let local = self
            .local_transforms
            .get(&entity)
            .copied()
            .unwrap_or_default()
            .transform;
        let world = if self
            .hierarchy
            .get(&entity)
            .and_then(|hierarchy| hierarchy.parent)
            .is_some()
        {
            combine_transforms(parent_world, local)
        } else {
            local
        };
        self.world_transforms
            .insert(entity, WorldTransform { transform: world });
        let children: Vec<_> = self
            .hierarchy
            .iter()
            .filter_map(|(child, hierarchy)| (hierarchy.parent == Some(entity)).then_some(*child))
            .collect();
        for child in children {
            self.propagate_world_transform(child, world);
        }
    }

    pub(super) fn refresh_node_cache(&mut self) {
        self.node_cache = self
            .entities
            .iter()
            .copied()
            .filter_map(|entity| {
                let name = self.names.get(&entity)?.0.clone();
                let kind = self.node_kind(entity)?;
                Some(SceneNode {
                    id: entity,
                    name,
                    kind,
                    parent: self
                        .hierarchy
                        .get(&entity)
                        .and_then(|hierarchy| hierarchy.parent),
                    transform: self
                        .local_transforms
                        .get(&entity)
                        .copied()
                        .unwrap_or_default()
                        .transform,
                    camera: self.cameras.get(&entity).cloned(),
                    mesh: self.mesh_renderers.get(&entity).cloned(),
                    directional_light: self.directional_lights.get(&entity).cloned(),
                })
            })
            .collect();
    }
}

fn combine_transforms(parent: Transform, local: Transform) -> Transform {
    let matrix = parent.matrix() * local.matrix();
    let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
    Transform {
        translation,
        rotation,
        scale,
    }
}
