use std::collections::{HashMap, HashSet};

use crate::core::math::{transform_to_mat4, Mat4, Transform};

use super::World;
use crate::scene::components::{ActiveInHierarchy, NodeKind, NodeRecord, SceneNode, WorldMatrix};
use crate::scene::ecs::{InternalSceneSystem, SystemStage};
use crate::scene::EntityId;

impl World {
    pub(super) fn ordinal_for(&self, kind: NodeKind) -> usize {
        let mut ordinal = 1;
        for entity in self.entities.iter().copied() {
            if self.kinds.get(&entity) == Some(&kind) {
                ordinal += 1;
            }
        }
        ordinal
    }

    pub(super) fn node_kind(&self, entity: EntityId) -> Option<NodeKind> {
        self.kinds.get(&entity).copied()
    }

    pub(crate) fn run_internal_scene_system(&mut self, system: InternalSceneSystem) {
        if system == InternalSceneSystem::ApplyDeferred {
            self.apply_deferred();
            return;
        }
        if system == InternalSceneSystem::UpdateEvents {
            self.update_all_events();
            return;
        }
        if !self.derived_state_dirty.should_run(system) {
            return;
        }
        match system {
            InternalSceneSystem::ApplyDeferred => unreachable!("ApplyDeferred is handled above"),
            InternalSceneSystem::UpdateEvents => unreachable!("UpdateEvents is handled above"),
            InternalSceneSystem::HierarchyValidity => self.rebuild_hierarchy_validity(),
            InternalSceneSystem::ActiveHierarchy => self.rebuild_active_in_hierarchy(),
            InternalSceneSystem::WorldTransform => self.rebuild_world_matrices(),
            InternalSceneSystem::NodeCache => self.refresh_node_cache(),
            InternalSceneSystem::RenderExtractPrepare => self.prepare_render_extract(),
        }
        self.derived_state_dirty.clear(system);
    }

    pub(crate) fn run_internal_scene_systems_for_stage(&mut self, stage: SystemStage) {
        let stage_plan = self.schedule.stage_plan();
        for system in stage_plan.internal_systems_for_stage(stage) {
            self.run_internal_scene_system(system.system());
        }
    }

    pub(crate) fn flush_pending_scene_systems_for_stage(&mut self, stage: SystemStage) {
        if !self.derived_state_dirty.has_pending() {
            return;
        }
        let stage_plan = self.schedule.stage_plan();
        for system in stage_plan.internal_systems_for_stage(stage) {
            let system = system.system();
            if self.derived_state_dirty.should_run(system) {
                self.run_internal_scene_system(system);
            }
        }
    }

    pub(crate) fn flush_pending_scene_systems(&mut self) {
        if !self.derived_state_dirty.has_pending() {
            return;
        }
        let stage_plan = self.schedule.stage_plan();
        for stage in stage_plan.stages().iter().copied() {
            for system in stage_plan.internal_systems_for_stage(stage) {
                self.run_internal_scene_system(system.system());
            }
        }
    }

    pub(crate) fn set_scene_system_flush_deferred(&mut self, defer_flush: bool) {
        self.derived_state_dirty.set_defer_flush(defer_flush);
    }

    pub(super) fn flush_scene_systems_now(&mut self) {
        self.flush_pending_scene_systems();
    }

    pub(super) fn project_active_in_hierarchy_for_read(&self, entity: EntityId) -> Option<bool> {
        if !self.derived_state_dirty.active_pending() {
            let Some(active) = self.active_in_hierarchy.get(&entity) else {
                return None;
            };

            return Some(active.0);
        }
        if !self.contains_entity(entity) {
            return None;
        }

        Some(self.active_self_chain_value(entity, &mut HashSet::new()))
    }

    #[cfg(test)]
    pub(crate) fn has_pending_scene_systems(&self) -> bool {
        self.derived_state_dirty.has_pending()
    }

    pub(super) fn mark_derived_state_dirty(&mut self) {
        self.derived_state_dirty.mark_hierarchy();
    }

    pub(super) fn mark_hierarchy_dirty(&mut self) {
        self.derived_state_dirty.mark_hierarchy();
    }

    pub(super) fn mark_active_state_dirty(&mut self) {
        self.derived_state_dirty.mark_active();
    }

    pub(super) fn mark_transform_dirty(&mut self) {
        self.derived_state_dirty.mark_transform();
    }

    pub(super) fn mark_node_cache_dirty(&mut self) {
        self.derived_state_dirty.mark_node_cache();
    }

    pub(super) fn collect_subtree_records(&self, entity: EntityId, records: &mut Vec<NodeRecord>) {
        let traversal = self.hierarchy_traversal_index();
        self.collect_subtree_records_with_traversal(entity, records, &traversal);
    }

    fn collect_subtree_records_with_traversal(
        &self,
        entity: EntityId,
        records: &mut Vec<NodeRecord>,
        traversal: &HierarchyTraversalIndex,
    ) {
        let Some(record) = self.node_record(entity) else {
            return;
        };
        records.push(record);
        for child in traversal.children_of(entity) {
            self.collect_subtree_records_with_traversal(*child, records, traversal);
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
        if !self.derived_state_dirty.hierarchy_or_transform_pending() {
            let Some(world) = self.world_matrices.get(&entity) else {
                return None;
            };

            return Some(matrix_to_transform(world.0));
        }
        let Some(world_matrix) = self.project_world_matrix_for_read(entity) else {
            return None;
        };

        Some(matrix_to_transform(world_matrix))
    }

    pub(super) fn project_node_for_read(&self, entity: EntityId) -> Option<SceneNode> {
        let Some(name) = self.names.get(&entity) else {
            return None;
        };
        let Some(kind) = self.node_kind(entity) else {
            return None;
        };
        Some(SceneNode {
            id: entity,
            name: name.0.clone(),
            kind,
            parent: self.parent_for_read(entity),
            transform: self.local_transform_value(entity),
            camera: self.cameras.get(&entity).cloned(),
            mesh: self.mesh_renderers.get(&entity).cloned(),
            sprite_2d: self.sprite_2d.get(&entity).cloned(),
            mesh_2d: self.mesh_2d.get(&entity).cloned(),
            ambient_light: self.ambient_lights.get(&entity).cloned(),
            directional_light: self.directional_lights.get(&entity).cloned(),
            point_light: self.point_lights.get(&entity).cloned(),
            rect_light: self.rect_lights.get(&entity).cloned(),
            spot_light: self.spot_lights.get(&entity).cloned(),
            rigid_body: self.rigid_bodies.get(&entity).cloned(),
            collider: self.colliders.get(&entity).cloned(),
            joint: self.joints.get(&entity).cloned(),
            animation_skeleton: self.animation_skeletons.get(&entity).cloned(),
            animation_player: self.animation_players.get(&entity).cloned(),
            animation_sequence_player: self.animation_sequence_players.get(&entity).cloned(),
            animation_graph_player: self.animation_graph_players.get(&entity).cloned(),
            animation_state_machine_player: self
                .animation_state_machine_players
                .get(&entity)
                .cloned(),
        })
    }

    pub(super) fn project_world_matrix_for_read(&self, entity: EntityId) -> Option<Mat4> {
        self.project_world_matrix_for_read_inner(entity, &mut HashSet::new())
    }

    fn project_world_matrix_for_read_inner(
        &self,
        entity: EntityId,
        seen: &mut HashSet<EntityId>,
    ) -> Option<Mat4> {
        if !self.contains_entity(entity) || !seen.insert(entity) {
            return None;
        }
        let local = self.local_transform_value(entity);
        let local_matrix = transform_to_mat4(local);
        let Some(parent) = self.parent_for_read(entity) else {
            return Some(local_matrix);
        };
        let Some(parent_matrix) = self.project_world_matrix_for_read_inner(parent, seen) else {
            return None;
        };

        Some(parent_matrix * local_matrix)
    }

    fn parent_for_read(&self, entity: EntityId) -> Option<EntityId> {
        let Some(hierarchy) = self.hierarchy.get(&entity) else {
            return None;
        };
        let Some(parent) = hierarchy.parent else {
            return None;
        };
        if parent == entity || !self.contains_entity(parent) {
            return None;
        }

        Some(parent)
    }

    fn active_self_chain_value(&self, entity: EntityId, seen: &mut HashSet<EntityId>) -> bool {
        if !seen.insert(entity) {
            return false;
        }
        if let Some(parent) = self.parent_for_read(entity) {
            if !self.active_self_chain_value(parent, seen) {
                return false;
            }
        }

        self.active_self_value(entity)
    }

    fn rebuild_hierarchy_validity(&mut self) {
        let parents = self.hierarchy_parent_snapshot();

        for entity_index in 0..self.entities.len() {
            let entity = self.entities[entity_index];
            let Some(hierarchy) = self.hierarchy.get_mut(&entity) else {
                continue;
            };
            let parent = hierarchy.parent;
            hierarchy.parent = parent.filter(|parent| {
                *parent != entity
                    && parents.contains_key(parent)
                    && !parent_chain_is_invalid(*parent, entity, &parents)
            });
        }
    }

    fn hierarchy_parent_snapshot(&self) -> HashMap<EntityId, Option<EntityId>> {
        let mut parents = HashMap::with_capacity(self.entities.len());
        for entity in self.entities.iter().copied() {
            let parent = match self.hierarchy.get(&entity) {
                Some(hierarchy) => hierarchy.parent,
                None => None,
            };
            parents.insert(entity, parent);
        }
        parents
    }

    fn rebuild_active_in_hierarchy(&mut self) {
        self.active_in_hierarchy.clear();
        let traversal = self.hierarchy_traversal_index();
        for root in traversal.roots() {
            self.propagate_active_state(*root, true, &traversal);
        }
    }

    fn rebuild_world_matrices(&mut self) {
        self.world_matrices.clear();
        let traversal = self.hierarchy_traversal_index();
        for root in traversal.roots() {
            self.propagate_world_matrix(*root, Mat4::IDENTITY, &traversal);
        }
    }

    fn propagate_active_state(
        &mut self,
        entity: EntityId,
        parent_active: bool,
        traversal: &HierarchyTraversalIndex,
    ) {
        let active = parent_active && self.active_self_value(entity);
        self.active_in_hierarchy
            .insert(entity, ActiveInHierarchy(active));
        for child in traversal.children_of(entity) {
            self.propagate_active_state(*child, active, traversal);
        }
    }

    fn propagate_world_matrix(
        &mut self,
        entity: EntityId,
        parent_world: Mat4,
        traversal: &HierarchyTraversalIndex,
    ) {
        let local = self.local_transform_value(entity);
        let local_matrix = transform_to_mat4(local);
        let world = if self.parent_of(entity).is_some() {
            parent_world * local_matrix
        } else {
            local_matrix
        };
        self.world_matrices.insert(entity, WorldMatrix(world));
        for child in traversal.children_of(entity) {
            self.propagate_world_matrix(*child, world, traversal);
        }
    }

    fn hierarchy_traversal_index(&self) -> HierarchyTraversalIndex {
        let mut index = HierarchyTraversalIndex::with_entity_capacity(self.entities.len());
        for entity in self.entities.iter().copied() {
            if let Some(parent) = self.parent_of(entity) {
                index.push_child(parent, entity);
            } else {
                index.push_root(entity);
            }
        }
        index
    }

    fn local_transform_value(&self, entity: EntityId) -> Transform {
        let Some(local) = self.local_transforms.get(&entity) else {
            return Transform::default();
        };

        local.transform
    }

    fn active_self_value(&self, entity: EntityId) -> bool {
        let Some(active) = self.active_self.get(&entity) else {
            return true;
        };

        active.0
    }

    pub(super) fn refresh_node_cache(&mut self) {
        self.node_cache.clear();
        self.node_cache.reserve(self.entities.len());
        for entity in self.entities.iter().copied() {
            let Some(name) = self.names.get(&entity) else {
                continue;
            };
            let Some(kind) = self.node_kind(entity) else {
                continue;
            };
            self.node_cache.push(SceneNode {
                id: entity,
                name: name.0.clone(),
                kind,
                parent: self.parent_of(entity),
                transform: self.local_transform_value(entity),
                camera: self.cameras.get(&entity).cloned(),
                mesh: self.mesh_renderers.get(&entity).cloned(),
                sprite_2d: self.sprite_2d.get(&entity).cloned(),
                mesh_2d: self.mesh_2d.get(&entity).cloned(),
                ambient_light: self.ambient_lights.get(&entity).cloned(),
                directional_light: self.directional_lights.get(&entity).cloned(),
                point_light: self.point_lights.get(&entity).cloned(),
                rect_light: self.rect_lights.get(&entity).cloned(),
                spot_light: self.spot_lights.get(&entity).cloned(),
                rigid_body: self.rigid_bodies.get(&entity).cloned(),
                collider: self.colliders.get(&entity).cloned(),
                joint: self.joints.get(&entity).cloned(),
                animation_skeleton: self.animation_skeletons.get(&entity).cloned(),
                animation_player: self.animation_players.get(&entity).cloned(),
                animation_sequence_player: self.animation_sequence_players.get(&entity).cloned(),
                animation_graph_player: self.animation_graph_players.get(&entity).cloned(),
                animation_state_machine_player: self
                    .animation_state_machine_players
                    .get(&entity)
                    .cloned(),
            });
        }
    }

    fn prepare_render_extract(&mut self) {
        for system in [
            InternalSceneSystem::HierarchyValidity,
            InternalSceneSystem::ActiveHierarchy,
            InternalSceneSystem::WorldTransform,
            InternalSceneSystem::NodeCache,
        ] {
            self.run_internal_scene_system(system);
        }
    }
}

struct HierarchyTraversalIndex {
    roots: Vec<EntityId>,
    children_by_parent: HashMap<EntityId, Vec<EntityId>>,
}

impl HierarchyTraversalIndex {
    fn with_entity_capacity(entity_count: usize) -> Self {
        Self {
            roots: Vec::with_capacity(entity_count),
            children_by_parent: HashMap::with_capacity(entity_count),
        }
    }

    fn push_root(&mut self, entity: EntityId) {
        self.roots.push(entity);
    }

    fn push_child(&mut self, parent: EntityId, child: EntityId) {
        self.children_by_parent
            .entry(parent)
            .or_default()
            .push(child);
    }

    fn roots(&self) -> &[EntityId] {
        &self.roots
    }

    fn children_of(&self, parent: EntityId) -> &[EntityId] {
        match self.children_by_parent.get(&parent) {
            Some(children) => children.as_slice(),
            None => &[],
        }
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
