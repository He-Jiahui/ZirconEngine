use std::collections::{HashMap, HashSet};

use zircon_math::{Mat4, Transform, transform_to_mat4};

use super::World;
use crate::EntityId;
use crate::components::{ActiveInHierarchy, NodeKind, NodeRecord, SceneNode, WorldMatrix};

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
        self.rebuild_hierarchy_validity();
        self.rebuild_active_in_hierarchy();
        self.rebuild_world_matrices();
        self.refresh_node_cache();
    }

    pub(super) fn collect_subtree_records(&self, entity: EntityId, records: &mut Vec<NodeRecord>) {
        let Some(record) = self.node_record(entity) else {
            return;
        };
        records.push(record);
        for child in self.children_of(entity) {
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

    pub(super) fn project_world_transform(&self, entity: EntityId) -> Option<Transform> {
        self.world_matrices
            .get(&entity)
            .copied()
            .map(|world| matrix_to_transform(world.0))
    }

    fn rebuild_hierarchy_validity(&mut self) {
        let entities: HashSet<_> = self.entities.iter().copied().collect();
        let parents: HashMap<_, _> = self
            .entities
            .iter()
            .copied()
            .map(|entity| {
                (
                    entity,
                    self.hierarchy
                        .get(&entity)
                        .and_then(|hierarchy| hierarchy.parent),
                )
            })
            .collect();

        for entity in self.entities.iter().copied().collect::<Vec<_>>() {
            let Some(hierarchy) = self.hierarchy.get_mut(&entity) else {
                continue;
            };
            let parent = hierarchy.parent;
            hierarchy.parent = parent.filter(|parent| {
                *parent != entity
                    && entities.contains(parent)
                    && !parent_chain_is_invalid(*parent, entity, &parents)
            });
        }
    }

    fn rebuild_active_in_hierarchy(&mut self) {
        self.active_in_hierarchy.clear();
        for root in self.root_entities() {
            self.propagate_active_state(root, true);
        }
    }

    fn rebuild_world_matrices(&mut self) {
        self.world_matrices.clear();
        for root in self.root_entities() {
            self.propagate_world_matrix(root, Mat4::IDENTITY);
        }
    }

    fn propagate_active_state(&mut self, entity: EntityId, parent_active: bool) {
        let active = parent_active && self.active_self_value(entity);
        self.active_in_hierarchy
            .insert(entity, ActiveInHierarchy(active));
        for child in self.children_of(entity) {
            self.propagate_active_state(child, active);
        }
    }

    fn propagate_world_matrix(&mut self, entity: EntityId, parent_world: Mat4) {
        let local = self
            .local_transforms
            .get(&entity)
            .copied()
            .unwrap_or_default()
            .transform;
        let local_matrix = transform_to_mat4(local);
        let world = if self.parent_of(entity).is_some() {
            parent_world * local_matrix
        } else {
            local_matrix
        };
        self.world_matrices.insert(entity, WorldMatrix(world));
        for child in self.children_of(entity) {
            self.propagate_world_matrix(child, world);
        }
    }

    fn root_entities(&self) -> Vec<EntityId> {
        self.entities
            .iter()
            .copied()
            .filter(|entity| self.parent_of(*entity).is_none())
            .collect()
    }

    fn children_of(&self, entity: EntityId) -> Vec<EntityId> {
        self.entities
            .iter()
            .copied()
            .filter(|child| self.parent_of(*child) == Some(entity))
            .collect()
    }

    fn active_self_value(&self, entity: EntityId) -> bool {
        self.active_self.get(&entity).copied().unwrap_or_default().0
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
                    parent: self.parent_of(entity),
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

fn parent_chain_is_invalid(
    start_parent: EntityId,
    entity: EntityId,
    parents: &HashMap<EntityId, Option<EntityId>>,
) -> bool {
    let mut seen = HashSet::from([entity]);
    let mut cursor = Some(start_parent);
    while let Some(current) = cursor {
        if !seen.insert(current) {
            return true;
        }
        cursor = parents.get(&current).copied().flatten();
    }
    false
}

pub(super) fn matrix_to_transform(matrix: Mat4) -> Transform {
    let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
    Transform {
        translation,
        rotation,
        scale,
    }
}
