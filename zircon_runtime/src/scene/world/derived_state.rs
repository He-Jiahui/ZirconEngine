use std::collections::{BTreeMap, HashMap, HashSet};

use crate::core::math::{Mat4, Transform, transform_to_mat4};

use super::World;
use crate::scene::EntityId;
use crate::scene::components::{
    ActiveInHierarchy, ActiveSelf, AmbientLight, AnimationGraphPlayerComponent,
    AnimationPlayerComponent, AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, MeshRenderer, Name, NodeKind,
    NodeRecord, PointLight, RectLight, RigidBodyComponent, SceneNode, SpotLight, Sprite2dComponent,
    WorldMatrix,
};
use crate::scene::ecs::{InternalSceneSystem, SystemStage};

pub(super) const NODE_KIND_ORDINAL_COUNT: usize = 9;

/// Incremental parent-to-children projection used by affected-row mutations.
/// Dense component rows remain the hierarchy authority; this index only avoids
/// rebuilding a whole-world traversal for subtree-local work.
#[derive(Debug, Default)]
pub(super) struct HierarchyMutationIndex {
    roots: BTreeMap<usize, EntityId>,
    children_by_parent: HashMap<EntityId, BTreeMap<usize, EntityId>>,
    indexed_entities: HashSet<EntityId>,
    dirty: bool,
}

impl PartialEq for HierarchyMutationIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl HierarchyMutationIndex {
    fn update_parent(
        &mut self,
        entity: EntityId,
        stable_order: usize,
        previous_parent: Option<EntityId>,
        current_parent: Option<EntityId>,
    ) {
        if previous_parent != current_parent {
            if let Some(previous_parent) = previous_parent {
                self.remove_child(previous_parent, stable_order, entity);
            } else {
                self.roots.remove(&stable_order);
            }
            if let Some(current_parent) = current_parent {
                let replaced = self
                    .children_by_parent
                    .entry(current_parent)
                    .or_default()
                    .insert(stable_order, entity);
                debug_assert!(replaced.is_none() || replaced == Some(entity));
            } else {
                let replaced = self.roots.insert(stable_order, entity);
                debug_assert!(replaced.is_none() || replaced == Some(entity));
            }
        } else if !self.indexed_entities.contains(&entity) {
            if let Some(current_parent) = current_parent {
                let replaced = self
                    .children_by_parent
                    .entry(current_parent)
                    .or_default()
                    .insert(stable_order, entity);
                debug_assert!(replaced.is_none() || replaced == Some(entity));
            } else {
                let replaced = self.roots.insert(stable_order, entity);
                debug_assert!(replaced.is_none() || replaced == Some(entity));
            }
        }
        self.indexed_entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: EntityId, stable_order: usize, parent: Option<EntityId>) {
        if let Some(parent) = parent {
            self.remove_child(parent, stable_order, entity);
        } else {
            self.roots.remove(&stable_order);
        }
        self.indexed_entities.remove(&entity);
        self.children_by_parent.remove(&entity);
    }

    fn remove_child(&mut self, parent: EntityId, stable_order: usize, entity: EntityId) {
        let remove_bucket = if let Some(children) = self.children_by_parent.get_mut(&parent) {
            let removed = children.remove(&stable_order);
            debug_assert!(removed.is_none() || removed == Some(entity));
            children.is_empty()
        } else {
            false
        };
        if remove_bucket {
            self.children_by_parent.remove(&parent);
        }
    }

    fn children_of(&self, parent: EntityId) -> impl DoubleEndedIterator<Item = EntityId> + '_ {
        self.children_by_parent
            .get(&parent)
            .into_iter()
            .flat_map(|children| children.values().copied())
    }

    fn roots(&self) -> impl DoubleEndedIterator<Item = EntityId> + '_ {
        self.roots.values().copied()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_current(&mut self) {
        self.dirty = false;
    }

    fn is_current_for_entity_count(&self, entity_count: usize) -> bool {
        !self.dirty && self.indexed_entities.len() == entity_count
    }

    fn rebuild(&mut self, rows: impl IntoIterator<Item = (EntityId, usize, Option<EntityId>)>) {
        self.roots.clear();
        self.children_by_parent.clear();
        self.indexed_entities.clear();
        for (entity, stable_order, parent) in rows {
            self.update_parent(entity, stable_order, None, parent);
        }
        self.dirty = false;
    }
}

const fn node_kind_ordinal_index(kind: NodeKind) -> usize {
    match kind {
        NodeKind::Empty => 0,
        NodeKind::Camera => 1,
        NodeKind::Cube => 2,
        NodeKind::Mesh => 3,
        NodeKind::AmbientLight => 4,
        NodeKind::DirectionalLight => 5,
        NodeKind::PointLight => 6,
        NodeKind::RectLight => 7,
        NodeKind::SpotLight => 8,
    }
}

impl World {
    pub(super) fn ordinal_for(&self, kind: NodeKind) -> usize {
        self.node_kind_ordinals[node_kind_ordinal_index(kind)].saturating_add(1)
    }

    pub(super) fn record_node_kind_added(&mut self, kind: NodeKind) {
        let ordinal = &mut self.node_kind_ordinals[node_kind_ordinal_index(kind)];
        *ordinal = ordinal.saturating_add(1);
    }

    pub(super) fn record_node_kind_removed(&mut self, kind: NodeKind) {
        let ordinal = &mut self.node_kind_ordinals[node_kind_ordinal_index(kind)];
        *ordinal = ordinal.saturating_sub(1);
    }

    pub(super) fn rebuild_node_kind_ordinals(&mut self) {
        let mut ordinals = [0_usize; NODE_KIND_ORDINAL_COUNT];
        for kind in self.kinds.values().copied() {
            let ordinal = &mut ordinals[node_kind_ordinal_index(kind)];
            *ordinal = ordinal.saturating_add(1);
        }
        self.node_kind_ordinals = ordinals;
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
            self.advance_messages();
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
            let Some(active) = self.get::<ActiveInHierarchy>(entity) else {
                return None;
            };

            return Some(active.0);
        }
        if !self.contains_entity(entity) {
            return None;
        }

        Some(self.active_self_chain_value(entity))
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

    pub(super) fn subtree_entity_ids(&self, root: EntityId) -> Vec<EntityId> {
        if !self.contains_entity(root) {
            return Vec::new();
        }

        if !self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len())
        {
            let traversal = self.hierarchy_traversal_index();
            let mut entities = Vec::new();
            let mut stack = vec![root];
            while let Some(entity) = stack.pop() {
                entities.push(entity);
                stack.extend(traversal.children_of(entity).iter().rev().copied());
            }
            return entities;
        }

        let mut entities = Vec::new();
        let mut stack = vec![root];
        while let Some(entity) = stack.pop() {
            entities.push(entity);
            stack.extend(self.hierarchy_mutation_index.children_of(entity).rev());
        }
        entities
    }

    pub(super) fn direct_child_entity_ids(&self, parent: EntityId) -> Vec<EntityId> {
        if !self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len())
        {
            return self
                .hierarchy_traversal_index()
                .children_of(parent)
                .to_vec();
        }
        self.hierarchy_mutation_index.children_of(parent).collect()
    }

    pub(super) fn has_direct_child_matching(
        &self,
        parent: EntityId,
        mut predicate: impl FnMut(EntityId) -> bool,
    ) -> bool {
        if self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len())
        {
            return self
                .hierarchy_mutation_index
                .children_of(parent)
                .any(predicate);
        }

        self.stable_entity_ids()
            .any(|child| self.parent_of(child) == Some(parent) && predicate(child))
    }

    pub(super) fn update_hierarchy_mutation_index(
        &mut self,
        entity: EntityId,
        previous_parent: Option<EntityId>,
        current_parent: Option<EntityId>,
    ) {
        let stable_order = self
            .stable_entity_order(entity)
            .expect("hierarchy index entity must retain stable order");
        self.hierarchy_mutation_index.update_parent(
            entity,
            stable_order,
            previous_parent,
            current_parent,
        );
    }

    pub(super) fn remove_hierarchy_mutation_index_entry(
        &mut self,
        entity: EntityId,
        stable_order: usize,
        parent: Option<EntityId>,
    ) {
        self.hierarchy_mutation_index
            .remove_entity(entity, stable_order, parent);
    }

    pub(super) fn mark_hierarchy_mutation_index_dirty(&mut self) {
        self.hierarchy_mutation_index.mark_dirty();
    }

    pub(super) fn rebuild_hierarchy_mutation_index(&mut self) {
        let rows = self
            .stable_entity_ids()
            .map(|entity| {
                (
                    entity,
                    self.stable_entity_order(entity)
                        .expect("stable entity must retain order while rebuilding hierarchy index"),
                    self.parent_of(entity),
                )
            })
            .collect::<Vec<_>>();
        self.hierarchy_mutation_index.rebuild(rows);
    }

    pub(super) fn ensure_hierarchy_mutation_index_current(&mut self) -> usize {
        if !self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len())
        {
            let visited = self.entities.len();
            self.rebuild_hierarchy_mutation_index();
            self.record_derived_state_hierarchy_topology_rebuild(visited);
            return visited;
        }
        0
    }

    fn collect_subtree_records_with_traversal(
        &self,
        entity: EntityId,
        records: &mut Vec<NodeRecord>,
        traversal: &HierarchyTraversalIndex,
    ) {
        let mut stack = vec![entity];
        while let Some(current) = stack.pop() {
            let Some(record) = self.node_record(current) else {
                continue;
            };
            records.push(record);
            stack.extend(traversal.children_of(current).iter().rev().copied());
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
            let Some(world) = self.get::<WorldMatrix>(entity) else {
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
        let Some(name) = self.get::<Name>(entity) else {
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
            camera: self.get::<CameraComponent>(entity).cloned(),
            mesh: self.get::<MeshRenderer>(entity).cloned(),
            sprite_2d: self.get::<Sprite2dComponent>(entity).cloned(),
            mesh_2d: self.get::<Mesh2dComponent>(entity).cloned(),
            ambient_light: self.get::<AmbientLight>(entity).cloned(),
            directional_light: self.get::<DirectionalLight>(entity).cloned(),
            point_light: self.get::<PointLight>(entity).cloned(),
            rect_light: self.get::<RectLight>(entity).cloned(),
            spot_light: self.get::<SpotLight>(entity).cloned(),
            rigid_body: self.get::<RigidBodyComponent>(entity).cloned(),
            collider: self.get::<ColliderComponent>(entity).cloned(),
            joint: self.get::<JointComponent>(entity).cloned(),
            animation_skeleton: self.get::<AnimationSkeletonComponent>(entity).cloned(),
            animation_player: self.get::<AnimationPlayerComponent>(entity).cloned(),
            animation_sequence_player: self
                .get::<AnimationSequencePlayerComponent>(entity)
                .cloned(),
            animation_graph_player: self.get::<AnimationGraphPlayerComponent>(entity).cloned(),
            animation_state_machine_player: self
                .get::<AnimationStateMachinePlayerComponent>(entity)
                .cloned(),
        })
    }

    pub(super) fn project_world_matrix_for_read(&self, entity: EntityId) -> Option<Mat4> {
        let mut lineage = Vec::new();
        let mut seen = HashSet::new();
        let mut current = entity;
        loop {
            if !self.contains_entity(current) || !seen.insert(current) {
                return None;
            }
            lineage.push(current);
            let Some(parent) = self.parent_for_read(current) else {
                break;
            };
            current = parent;
        }

        let mut world = Mat4::IDENTITY;
        for current in lineage.iter().rev().copied() {
            world = world * transform_to_mat4(self.local_transform_value(current));
        }
        Some(world)
    }

    fn parent_for_read(&self, entity: EntityId) -> Option<EntityId> {
        let Some(hierarchy) = self.get::<Hierarchy>(entity) else {
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

    fn active_self_chain_value(&self, entity: EntityId) -> bool {
        let mut seen = HashSet::new();
        let mut current = entity;
        loop {
            if !seen.insert(current) || !self.active_self_value(current) {
                return false;
            }
            let Some(parent) = self.parent_for_read(current) else {
                return true;
            };
            current = parent;
        }
    }

    fn rebuild_hierarchy_validity(&mut self) {
        let parents = self.hierarchy_parent_snapshot();
        let mut seen = HashSet::new();
        let mut hierarchy_updates = Vec::new();
        let mut parent_chain_steps: usize = 0;
        let hierarchy_index_was_current = self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len());

        let entities = self.stable_entity_ids().collect::<Vec<_>>();
        for entity in entities {
            let Some(hierarchy) = self.get::<Hierarchy>(entity) else {
                continue;
            };
            let previous_parent = hierarchy.parent;
            let current_parent = previous_parent.filter(|parent| {
                let parent_exists = *parent != entity && parents.contains_key(parent);
                let current_parent_is_valid =
                    parent_exists && !parent_chain_is_invalid(*parent, entity, &parents, &mut seen);
                if parent_exists {
                    parent_chain_steps =
                        parent_chain_steps.saturating_add(seen.len().saturating_sub(1));
                }
                current_parent_is_valid
            });
            if previous_parent != current_parent {
                hierarchy_updates.push((entity, previous_parent, current_parent));
            }
        }
        for (entity, previous_parent, current_parent) in hierarchy_updates.iter().copied() {
            let updated = if let Some(hierarchy) = self.get_mut::<Hierarchy>(entity) {
                hierarchy.parent = current_parent;
                true
            } else {
                false
            };
            if updated && hierarchy_index_was_current {
                self.update_hierarchy_mutation_index(entity, previous_parent, current_parent);
            }
        }
        if hierarchy_index_was_current {
            self.hierarchy_mutation_index.mark_current();
        }
        self.record_derived_state_hierarchy_validity(
            parents.len(),
            self.entities.len(),
            parent_chain_steps,
        );
    }

    fn hierarchy_parent_snapshot(&self) -> HashMap<EntityId, Option<EntityId>> {
        let mut parents = HashMap::with_capacity(self.entities.len());
        for entity in self.stable_entity_ids() {
            let parent = match self.get::<Hierarchy>(entity) {
                Some(hierarchy) => hierarchy.parent,
                None => None,
            };
            parents.insert(entity, parent);
        }
        parents
    }

    fn rebuild_active_in_hierarchy(&mut self) {
        self.ensure_hierarchy_mutation_index_current();
        let traversal = std::mem::take(&mut self.hierarchy_mutation_index);
        let mut propagated_entities: usize = 0;
        for root in traversal.roots() {
            propagated_entities = propagated_entities
                .saturating_add(self.propagate_active_state(root, true, &traversal));
        }
        self.hierarchy_mutation_index = traversal;
        self.record_derived_state_active_propagation(propagated_entities);
    }

    fn rebuild_world_matrices(&mut self) {
        self.ensure_hierarchy_mutation_index_current();
        let traversal = std::mem::take(&mut self.hierarchy_mutation_index);
        let mut propagated_entities: usize = 0;
        for root in traversal.roots() {
            propagated_entities = propagated_entities.saturating_add(self.propagate_world_matrix(
                root,
                Mat4::IDENTITY,
                &traversal,
            ));
        }
        self.hierarchy_mutation_index = traversal;
        self.record_derived_state_world_matrix_propagation(propagated_entities);
    }

    fn propagate_active_state(
        &mut self,
        entity: EntityId,
        parent_active: bool,
        traversal: &HierarchyMutationIndex,
    ) -> usize {
        let mut stack = vec![(entity, parent_active)];
        let mut propagated_entities: usize = 0;
        while let Some((current, inherited_active)) = stack.pop() {
            let active = inherited_active && self.active_self_value(current);
            self.replace_derived_component(current, ActiveInHierarchy(active));
            propagated_entities = propagated_entities.saturating_add(1);
            stack.extend(
                traversal
                    .children_of(current)
                    .rev()
                    .map(|child| (child, active)),
            );
        }
        propagated_entities
    }

    fn propagate_world_matrix(
        &mut self,
        entity: EntityId,
        parent_world: Mat4,
        traversal: &HierarchyMutationIndex,
    ) -> usize {
        let mut stack = vec![(entity, parent_world)];
        let mut propagated_entities: usize = 0;
        while let Some((current, inherited_world)) = stack.pop() {
            let local = self.local_transform_value(current);
            let local_matrix = transform_to_mat4(local);
            let world = if self.parent_of(current).is_some() {
                inherited_world * local_matrix
            } else {
                local_matrix
            };
            self.replace_derived_component(current, WorldMatrix(world));
            propagated_entities = propagated_entities.saturating_add(1);
            stack.extend(
                traversal
                    .children_of(current)
                    .rev()
                    .map(|child| (child, world)),
            );
        }
        propagated_entities
    }

    fn hierarchy_traversal_index(&self) -> HierarchyTraversalIndex {
        let mut index = HierarchyTraversalIndex::with_entity_capacity(self.entities.len());
        for entity in self.stable_entity_ids() {
            if let Some(parent) = self.parent_of(entity) {
                index.push_child(parent, entity);
            } else {
                index.push_root(entity);
            }
        }
        index
    }

    fn local_transform_value(&self, entity: EntityId) -> Transform {
        let Some(local) = self.get::<LocalTransform>(entity) else {
            return Transform::default();
        };

        local.transform
    }

    fn active_self_value(&self, entity: EntityId) -> bool {
        let Some(active) = self.get::<ActiveSelf>(entity) else {
            return true;
        };

        active.0
    }

    pub(super) fn refresh_node_cache(&mut self) {
        self.node_cache.clear();
        self.node_cache.reserve(self.entities.len());
        let entities = self.stable_entity_ids().collect::<Vec<_>>();
        let mut rebuilt_entities: usize = 0;
        for entity in entities {
            let Some(name) = self.get::<Name>(entity) else {
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
                camera: self.get::<CameraComponent>(entity).cloned(),
                mesh: self.get::<MeshRenderer>(entity).cloned(),
                sprite_2d: self.get::<Sprite2dComponent>(entity).cloned(),
                mesh_2d: self.get::<Mesh2dComponent>(entity).cloned(),
                ambient_light: self.get::<AmbientLight>(entity).cloned(),
                directional_light: self.get::<DirectionalLight>(entity).cloned(),
                point_light: self.get::<PointLight>(entity).cloned(),
                rect_light: self.get::<RectLight>(entity).cloned(),
                spot_light: self.get::<SpotLight>(entity).cloned(),
                rigid_body: self.get::<RigidBodyComponent>(entity).cloned(),
                collider: self.get::<ColliderComponent>(entity).cloned(),
                joint: self.get::<JointComponent>(entity).cloned(),
                animation_skeleton: self.get::<AnimationSkeletonComponent>(entity).cloned(),
                animation_player: self.get::<AnimationPlayerComponent>(entity).cloned(),
                animation_sequence_player: self
                    .get::<AnimationSequencePlayerComponent>(entity)
                    .cloned(),
                animation_graph_player: self.get::<AnimationGraphPlayerComponent>(entity).cloned(),
                animation_state_machine_player: self
                    .get::<AnimationStateMachinePlayerComponent>(entity)
                    .cloned(),
            });
            rebuilt_entities = rebuilt_entities.saturating_add(1);
        }
        self.record_derived_state_node_cache_rebuild(rebuilt_entities);
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
    seen: &mut HashSet<EntityId>,
) -> bool {
    seen.clear();
    seen.insert(entity);
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
