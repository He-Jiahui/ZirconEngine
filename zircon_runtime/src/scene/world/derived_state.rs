use std::collections::{HashMap, HashSet};

use crate::core::math::{transform_to_mat4, Mat4, Transform};

use super::World;
use crate::scene::components::{ActiveInHierarchy, NodeKind, NodeRecord, SceneNode, WorldMatrix};
use crate::scene::ecs::InternalSceneSystem;
use crate::scene::EntityId;

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

    pub(crate) fn run_internal_scene_system(&mut self, system: InternalSceneSystem) {
        if system == InternalSceneSystem::ApplyDeferred {
            self.apply_deferred();
            return;
        }
        if !self.derived_state_dirty.should_run(system) {
            return;
        }
        match system {
            InternalSceneSystem::ApplyDeferred => unreachable!("ApplyDeferred is handled above"),
            InternalSceneSystem::HierarchyValidity => self.rebuild_hierarchy_validity(),
            InternalSceneSystem::ActiveHierarchy => self.rebuild_active_in_hierarchy(),
            InternalSceneSystem::WorldTransform => self.rebuild_world_matrices(),
            InternalSceneSystem::NodeCache => self.refresh_node_cache(),
            InternalSceneSystem::RenderExtractPrepare => self.prepare_render_extract(),
        }
        self.derived_state_dirty.clear(system);
    }

    pub(crate) fn run_internal_scene_systems_for_stage(
        &mut self,
        stage: crate::scene::SystemStage,
    ) {
        let systems = self
            .schedule
            .systems_for_stage(stage)
            .cloned()
            .collect::<Vec<_>>();
        for system in systems {
            self.run_internal_scene_system(system.system());
        }
    }

    pub(crate) fn flush_pending_scene_systems(&mut self) {
        if !self.derived_state_dirty.has_pending() {
            return;
        }
        let systems = self.schedule.systems().to_vec();
        for system in systems {
            self.run_internal_scene_system(system.system());
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
            return self
                .active_in_hierarchy
                .get(&entity)
                .copied()
                .map(|active| active.0);
        }
        self.contains_entity(entity)
            .then(|| self.active_self_chain_value(entity, &mut HashSet::new()))
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
        if !self.derived_state_dirty.hierarchy_or_transform_pending() {
            return self
                .world_matrices
                .get(&entity)
                .copied()
                .map(|world| matrix_to_transform(world.0));
        }
        self.project_world_matrix_for_read(entity)
            .map(matrix_to_transform)
    }

    pub(super) fn project_node_for_read(&self, entity: EntityId) -> Option<SceneNode> {
        let name = self.names.get(&entity)?.0.clone();
        let kind = self.node_kind(entity)?;
        Some(SceneNode {
            id: entity,
            name,
            kind,
            parent: self.parent_for_read(entity),
            transform: self
                .local_transforms
                .get(&entity)
                .copied()
                .unwrap_or_default()
                .transform,
            camera: self.cameras.get(&entity).cloned(),
            mesh: self.mesh_renderers.get(&entity).cloned(),
            sprite_2d: self.sprite_2d.get(&entity).cloned(),
            mesh_2d: self.mesh_2d.get(&entity).cloned(),
            directional_light: self.directional_lights.get(&entity).cloned(),
            point_light: self.point_lights.get(&entity).cloned(),
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

    fn project_world_matrix_for_read(&self, entity: EntityId) -> Option<Mat4> {
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
        let local = self
            .local_transforms
            .get(&entity)
            .copied()
            .unwrap_or_default()
            .transform;
        let local_matrix = transform_to_mat4(local);
        self.parent_for_read(entity)
            .map(|parent| {
                self.project_world_matrix_for_read_inner(parent, seen)
                    .map(|parent| parent * local_matrix)
            })
            .unwrap_or(Some(local_matrix))
    }

    fn parent_for_read(&self, entity: EntityId) -> Option<EntityId> {
        self.hierarchy
            .get(&entity)
            .and_then(|hierarchy| hierarchy.parent)
            .filter(|parent| *parent != entity && self.contains_entity(*parent))
    }

    fn active_self_chain_value(&self, entity: EntityId, seen: &mut HashSet<EntityId>) -> bool {
        if !seen.insert(entity) {
            return false;
        }
        self.parent_for_read(entity)
            .map(|parent| self.active_self_chain_value(parent, seen))
            .unwrap_or(true)
            && self.active_self_value(entity)
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
                    sprite_2d: self.sprite_2d.get(&entity).cloned(),
                    mesh_2d: self.mesh_2d.get(&entity).cloned(),
                    directional_light: self.directional_lights.get(&entity).cloned(),
                    point_light: self.point_lights.get(&entity).cloned(),
                    spot_light: self.spot_lights.get(&entity).cloned(),
                    rigid_body: self.rigid_bodies.get(&entity).cloned(),
                    collider: self.colliders.get(&entity).cloned(),
                    joint: self.joints.get(&entity).cloned(),
                    animation_skeleton: self.animation_skeletons.get(&entity).cloned(),
                    animation_player: self.animation_players.get(&entity).cloned(),
                    animation_sequence_player: self
                        .animation_sequence_players
                        .get(&entity)
                        .cloned(),
                    animation_graph_player: self.animation_graph_players.get(&entity).cloned(),
                    animation_state_machine_player: self
                        .animation_state_machine_players
                        .get(&entity)
                        .cloned(),
                })
            })
            .collect();
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
