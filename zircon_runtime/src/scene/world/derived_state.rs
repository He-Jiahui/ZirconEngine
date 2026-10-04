use std::collections::{HashMap, HashSet};

use crate::core::math::{transform_to_mat4, Mat4, Transform};

use super::{
    dirty_state::DerivedStateFrontier, hierarchy_topology::HierarchyTopology, SceneError,
    SceneResult, World,
};
use crate::scene::components::{
    ActiveInHierarchy, ActiveSelf, AmbientLight, AnimationGraphPlayerComponent,
    AnimationPlayerComponent, AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, MeshRenderer, Name, NodeKind,
    NodeRecord, PointLight, RectLight, RigidBodyComponent, SceneNode, SpotLight, Sprite2dComponent,
    WorldMatrix,
};
use crate::scene::ecs::{Component, InternalSceneSystem, SystemStage};
use crate::scene::EntityId;

pub(super) const NODE_KIND_ORDINAL_COUNT: usize = 9;

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
        self.flush_deferred_component_mutations();
        if system == InternalSceneSystem::ApplyDeferred {
            self.apply_deferred();
            return;
        }
        if system == InternalSceneSystem::UpdateEvents {
            self.advance_removed_component_events();
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
            InternalSceneSystem::RenderExtractPrepare => {
                self.prepare_render_extract();
                let source_world_generation = self.world_generation();
                let source_change_tick = self.read_change_tick();
                self.derived_state_dirty
                    .publish_render_dirty_journal(source_world_generation, source_change_tick);
                self.publish_render_component_changes();
            }
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
        self.flush_deferred_component_mutations();
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
        self.flush_deferred_component_mutations();
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

    pub(super) fn flush_deferred_component_mutations(&mut self) {
        for mutation in self.derived_state_dirty.take_component_mutations() {
            self.apply_deferred_component_mutation(mutation);
        }
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

    pub(super) fn mark_hierarchy_dirty_at(&mut self, entity: EntityId) {
        self.derived_state_dirty.mark_hierarchy_at(entity);
    }

    pub(super) fn mark_checked_hierarchy_dirty_at(&mut self, entity: EntityId) {
        self.derived_state_dirty.mark_checked_hierarchy_at(entity);
    }

    pub(super) fn mark_active_state_dirty(&mut self) {
        self.derived_state_dirty.mark_active();
    }

    pub(super) fn mark_active_state_dirty_at(&mut self, entity: EntityId) {
        self.derived_state_dirty.mark_active_at(entity);
    }

    pub(super) fn mark_transform_dirty(&mut self) {
        self.derived_state_dirty.mark_transform();
    }

    pub(super) fn mark_transform_dirty_at(&mut self, entity: EntityId) {
        self.derived_state_dirty.mark_transform_at(entity);
    }

    pub(super) fn mark_node_cache_dirty(&mut self) {
        self.derived_state_dirty.mark_node_cache();
    }

    pub(super) fn mark_node_cache_dirty_at(&mut self, entity: EntityId) {
        self.derived_state_dirty.mark_node_cache_at(entity);
    }

    pub(super) fn collect_subtree_records(&self, entity: EntityId, records: &mut Vec<NodeRecord>) {
        if self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len())
        {
            let mut stack = vec![entity];
            while let Some(current) = stack.pop() {
                let Some(record) = self.node_record(current) else {
                    continue;
                };
                records.push(record);
                let children = self.hierarchy_mutation_index.children_of(current).rev();
                stack.extend(children.filter(|child| *child != entity));
            }
            return;
        }
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
                stack.extend(
                    traversal
                        .children_of(entity)
                        .iter()
                        .rev()
                        .copied()
                        .filter(|child| *child != root),
                );
            }
            return entities;
        }

        let mut entities = Vec::new();
        let mut stack = vec![root];
        while let Some(entity) = stack.pop() {
            entities.push(entity);
            stack.extend(
                self.hierarchy_mutation_index
                    .children_of(entity)
                    .rev()
                    .filter(|child| *child != root),
            );
        }
        entities
    }

    /// Counts instances of a registered ECS component in `root` and all descendants.
    ///
    /// This deliberately walks hierarchy ownership rather than projecting node records so
    /// editor preflight remains correct for components added independently of `NodeKind`.
    pub fn subtree_component_count<T>(&self, root: EntityId) -> usize
    where
        T: Component,
    {
        let Some(component_id) = self.registered_component_id::<T>() else {
            return 0;
        };
        if !self.contains_entity(root) {
            return 0;
        }

        let mut count = 0;
        let mut stack = vec![root];
        if self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len())
        {
            while let Some(entity) = stack.pop() {
                count += usize::from(self.contains_component_id(entity, component_id));
                stack.extend(
                    self.hierarchy_mutation_index
                        .children_of(entity)
                        .rev()
                        .filter(|child| *child != root),
                );
            }
            return count;
        }

        let traversal = self.hierarchy_traversal_index();
        while let Some(entity) = stack.pop() {
            count += usize::from(self.contains_component_id(entity, component_id));
            stack.extend(
                traversal
                    .children_of(entity)
                    .iter()
                    .rev()
                    .copied()
                    .filter(|child| *child != root),
            );
        }
        count
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
        if self
            .hierarchy_mutation_index
            .needs_source_rebuild(self.entities.len())
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
            let children = traversal.children_of(current).iter().rev().copied();
            stack.extend(children.filter(|child| *child != entity));
        }
    }

    pub(super) fn is_descendant(&self, entity: EntityId, ancestor: EntityId) -> SceneResult<bool> {
        let mut cursor = Some(entity);
        let mut checkpoint = entity;
        let mut checkpoint_span = 1_usize;
        let mut steps_since_checkpoint = 0_usize;
        while let Some(current) = cursor {
            // Preserve the requested-ancestor rejection before diagnosing an existing cycle.
            if current == ancestor {
                return Ok(true);
            }
            // Brent's spaced checkpoints bound a corrupt chain without depth-sized scratch.
            if steps_since_checkpoint != 0 && current == checkpoint {
                return Err(SceneError::HierarchyParentChainCycle {
                    start: entity,
                    repeated: current,
                });
            }
            if steps_since_checkpoint == checkpoint_span {
                checkpoint = current;
                checkpoint_span = checkpoint_span.saturating_mul(2);
                steps_since_checkpoint = 0;
            }
            cursor = self.parent_of(current);
            steps_since_checkpoint = steps_since_checkpoint.saturating_add(1);
        }
        Ok(false)
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

    // 干净状态优先读已发布矩阵；缓存缺失或脏状态沿作者态父链即时投影，不刷新保留的节点缓存。
    pub(super) fn project_world_matrix_for_read(&self, entity: EntityId) -> Option<Mat4> {
        if !self.derived_state_dirty.hierarchy_or_transform_pending() {
            if let Some(world_matrix) = self.get::<WorldMatrix>(entity) {
                return Some(world_matrix.0);
            }
        }

        let mut world = Mat4::IDENTITY;
        let mut current = entity;
        let mut slow = Some(entity);
        let mut fast = Some(entity);
        loop {
            if !self.contains_entity(current) {
                return None;
            }
            world = transform_to_mat4(self.local_transform_value(current)) * world;
            let Some(parent) = self.parent_for_read(current) else {
                return Some(world);
            };
            current = parent;

            slow = slow.and_then(|ancestor| self.parent_for_read(ancestor));
            fast = fast
                .and_then(|ancestor| self.parent_for_read(ancestor))
                .and_then(|ancestor| self.parent_for_read(ancestor));
            if slow.is_some() && slow == fast {
                return None;
            }
        }
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
        let mut current = entity;
        let mut slow = Some(entity);
        let mut fast = Some(entity);
        loop {
            if !self.active_self_value(current) {
                return false;
            }
            let Some(parent) = self.parent_for_read(current) else {
                return true;
            };
            current = parent;

            slow = slow.and_then(|ancestor| self.parent_for_read(ancestor));
            fast = fast
                .and_then(|ancestor| self.parent_for_read(ancestor))
                .and_then(|ancestor| self.parent_for_read(ancestor));
            if slow.is_some() && slow == fast {
                return false;
            }
        }
    }

    fn rebuild_active_in_hierarchy(&mut self) {
        let frontier = self.derived_state_dirty.take_active_frontier();
        self.ensure_hierarchy_mutation_index_current();
        let traversal = std::mem::take(&mut self.hierarchy_mutation_index);
        let mut visited_entities: usize = 0;
        let mut written_entities: usize = 0;
        for root in self.derived_state_frontier_roots(&frontier, &traversal) {
            let inherited_active = traversal
                .parent_of(root)
                .and_then(|parent| self.get::<ActiveInHierarchy>(parent))
                .map(|active| active.0)
                .unwrap_or(true);
            let (visited, written) =
                self.propagate_active_state(root, inherited_active, &traversal);
            visited_entities = visited_entities.saturating_add(visited);
            written_entities = written_entities.saturating_add(written);
        }
        self.hierarchy_mutation_index = traversal;
        self.record_derived_state_active_propagation(visited_entities, written_entities);
    }

    fn rebuild_world_matrices(&mut self) {
        let frontier = self.derived_state_dirty.take_transform_frontier();
        self.ensure_hierarchy_mutation_index_current();
        let traversal = std::mem::take(&mut self.hierarchy_mutation_index);
        let mut visited_entities: usize = 0;
        let mut written_entities: usize = 0;
        for root in self.derived_state_frontier_roots(&frontier, &traversal) {
            let inherited_world = traversal
                .parent_of(root)
                .and_then(|parent| self.get::<WorldMatrix>(parent))
                .map(|world| world.0)
                .unwrap_or(Mat4::IDENTITY);
            let (visited, written) = self.propagate_world_matrix(root, inherited_world, &traversal);
            visited_entities = visited_entities.saturating_add(visited);
            written_entities = written_entities.saturating_add(written);
        }
        self.hierarchy_mutation_index = traversal;
        self.record_derived_state_world_matrix_propagation(visited_entities, written_entities);
    }

    fn propagate_active_state(
        &mut self,
        entity: EntityId,
        parent_active: bool,
        traversal: &HierarchyTopology,
    ) -> (usize, usize) {
        // Keep an iterator only where a parent has more than one child.
        // Neither wide siblings nor a single-child chain fills this stack.
        let mut pending = Vec::new();
        let mut next = Some((entity, parent_active));
        let mut visited_entities: usize = 0;
        let mut written_entities: usize = 0;
        while let Some((current, inherited_active)) = next {
            let active = inherited_active && self.active_self_value(current);
            let next_active = ActiveInHierarchy(active);
            if self.get::<ActiveInHierarchy>(current) != Some(&next_active) {
                self.replace_derived_component(current, next_active);
                written_entities = written_entities.saturating_add(1);
                self.derived_state_dirty.mark_render_dirty_at(current);
            }
            visited_entities = visited_entities.saturating_add(1);

            let mut children = traversal.children_of(current);
            if let Some(first_child) = children.next() {
                let mut remaining = children.peekable();
                if remaining.peek().is_some() {
                    pending.push((remaining, active));
                }
                next = Some((first_child, active));
                continue;
            }
            next = loop {
                let Some((siblings, inherited_active)) = pending.last_mut() else {
                    break None;
                };
                if let Some(sibling) = siblings.next() {
                    break Some((sibling, *inherited_active));
                }
                pending.pop();
            };
        }
        (visited_entities, written_entities)
    }

    fn propagate_world_matrix(
        &mut self,
        entity: EntityId,
        parent_world: Mat4,
        traversal: &HierarchyTopology,
    ) -> (usize, usize) {
        // Keep inherited matrices only for branching ancestors, not each
        // sibling of a wide parent or each node of a single-child chain.
        let mut pending = Vec::new();
        let mut next = Some((entity, parent_world));
        let mut visited_entities: usize = 0;
        let mut written_entities: usize = 0;
        while let Some((current, inherited_world)) = next {
            let local = self.local_transform_value(current);
            let local_matrix = transform_to_mat4(local);
            let world = if traversal.parent_of(current).is_some() {
                inherited_world * local_matrix
            } else {
                local_matrix
            };
            let next_world = WorldMatrix(world);
            if self.get::<WorldMatrix>(current) != Some(&next_world) {
                self.replace_derived_component(current, next_world);
                written_entities = written_entities.saturating_add(1);
                self.derived_state_dirty.mark_render_dirty_at(current);
            }
            visited_entities = visited_entities.saturating_add(1);

            let mut children = traversal.children_of(current);
            if let Some(first_child) = children.next() {
                let mut remaining = children.peekable();
                if remaining.peek().is_some() {
                    pending.push((remaining, world));
                }
                next = Some((first_child, world));
                continue;
            }
            next = loop {
                let Some((siblings, inherited_world)) = pending.last_mut() else {
                    break None;
                };
                if let Some(sibling) = siblings.next() {
                    break Some((sibling, *inherited_world));
                }
                pending.pop();
            };
        }
        (visited_entities, written_entities)
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

    fn derived_state_frontier_roots(
        &self,
        frontier: &DerivedStateFrontier,
        topology: &HierarchyTopology,
    ) -> Vec<EntityId> {
        if frontier.is_all() {
            return topology.roots().collect();
        }

        let candidates = frontier
            .entities()
            .filter(|entity| self.contains_entity(*entity))
            .collect::<HashSet<_>>();
        let mut roots = candidates
            .iter()
            .copied()
            .filter(|entity| {
                let mut parent = topology.parent_of(*entity);
                while let Some(current) = parent {
                    if frontier.contains(current) {
                        return false;
                    }
                    parent = topology.parent_of(current);
                }
                true
            })
            .collect::<Vec<_>>();
        roots.sort_by_key(|entity| self.stable_entity_order(*entity));
        roots
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
        let frontier = self.derived_state_dirty.take_node_cache_frontier();
        self.ensure_hierarchy_mutation_index_current();
        let topology_generation = self.hierarchy_mutation_index.generation();
        let topology_changed = self.node_cache_topology_generation != topology_generation;
        if frontier.is_all()
            || self.node_cache_rows.len() != self.node_cache.len()
            || (topology_changed && frontier.entities().next().is_none())
        {
            return self.rebuild_node_cache();
        }

        let mut rebuilt_entities: usize = 0;
        for entity in frontier.entities() {
            let Some(row) = self.node_cache_rows.get(&entity).copied() else {
                return self.rebuild_node_cache();
            };
            let Some(node) = self.project_node_for_read(entity) else {
                return self.rebuild_node_cache();
            };
            self.node_cache[row] = node;
            rebuilt_entities = rebuilt_entities.saturating_add(1);
        }
        self.node_cache_topology_generation = topology_generation;
        self.record_derived_state_node_cache_rebuild(rebuilt_entities);
    }

    fn rebuild_node_cache(&mut self) {
        self.node_cache.clear();
        self.node_cache_rows.clear();
        self.node_cache.reserve(self.entities.len());
        self.node_cache_rows.reserve(self.entities.len());
        let entities = self.stable_entity_ids().collect::<Vec<_>>();
        let mut rebuilt_entities: usize = 0;
        for entity in entities {
            let Some(node) = self.project_node_for_read(entity) else {
                continue;
            };
            let row = self.node_cache.len();
            self.node_cache_rows.insert(entity, row);
            self.node_cache.push(node);
            rebuilt_entities = rebuilt_entities.saturating_add(1);
        }
        self.node_cache_topology_generation = self.hierarchy_mutation_index.generation();
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

pub(super) fn matrix_to_transform(matrix: Mat4) -> Transform {
    let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
    Transform {
        translation,
        rotation,
        scale,
    }
}

#[cfg(test)]
#[path = "../tests/derived_state/subtree_cycle_profile.rs"]
mod subtree_cycle_profile;

#[cfg(test)]
#[path = "tests/derived_state_clean_world_matrix_profile.rs"]
mod clean_world_matrix_profile;

#[cfg(all(test, target_os = "windows"))]
#[path = "tests/derived_state_scale_profile.rs"]
mod derived_state_scale_profile;
