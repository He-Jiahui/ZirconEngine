use crate::scene::ecs::Bundle;
use crate::scene::{EntityId, NodeKind};
use zircon_runtime_interface::world_sync::WorldFact;

use super::{BundleInsertionTransaction, SceneError, SceneResult, World};

impl World {
    pub fn spawn<B>(&mut self, bundle: B) -> SceneResult<EntityId>
    where
        B: Bundle,
    {
        let entity = self.entity_id_allocator.next_available()?;
        let mut transaction = self.begin_bundle_spawn(entity, NodeKind::Mesh)?;
        bundle.stage_into(&mut transaction)?;
        transaction.finish()?;
        Ok(entity)
    }

    pub(crate) fn spawn_empty_at(&mut self, entity: EntityId) -> SceneResult<bool> {
        if self.contains_entity(entity) {
            return Ok(false);
        }
        let next_id = self.entity_id_allocator.next_after(entity)?;
        self.register_stable_entity(entity)?;
        self.entity_id_allocator
            .replace_next(next_id)
            .expect("prevalidated explicit entity spawn must retain a valid allocator state");
        self.append_entity_to_dense_storage(entity);
        self.kinds.insert(entity, NodeKind::Empty);
        self.record_node_kind_added(NodeKind::Empty);
        self.update_hierarchy_mutation_index(entity, None, None);
        self.bump_lifecycle_visibility_revision();
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        self.advance_scene_binding_generations_for_new_descendant(entity);
        self.record_world_fact(WorldFact::Spawned(entity));
        Ok(true)
    }

    pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<EntityId>
    where
        B: Bundle,
    {
        let mut transaction = self.begin_bundle_spawn(entity, NodeKind::Empty)?;
        bundle.stage_into(&mut transaction)?;
        transaction.finish()?;
        Ok(entity)
    }

    pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<()>
    where
        B: Bundle,
    {
        let mut transaction = self.begin_bundle_insertion(entity)?;
        bundle.stage_into(&mut transaction)?;
        transaction.finish()
    }

    pub(crate) fn begin_bundle_insertion(
        &mut self,
        entity: EntityId,
    ) -> SceneResult<BundleInsertionTransaction<'_>> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("insert component on", entity));
        }
        let internal_entity = self
            .internal_entity(entity)
            .ok_or_else(|| SceneError::missing_entity("insert component on", entity))?;
        Ok(BundleInsertionTransaction::new(
            self,
            entity,
            internal_entity,
        ))
    }

    pub(crate) fn begin_deferred_bundle_insertion(
        &mut self,
        entity: EntityId,
    ) -> SceneResult<BundleInsertionTransaction<'_>> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("insert component on", entity));
        }
        let internal_entity = self
            .internal_entity(entity)
            .ok_or_else(|| SceneError::missing_entity("insert component on", entity))?;
        Ok(BundleInsertionTransaction::new_deferred_existing(
            self,
            entity,
            internal_entity,
        ))
    }

    pub(crate) fn begin_deferred_bundle_spawn(
        &mut self,
        entity: EntityId,
        include_default_components: bool,
    ) -> SceneResult<BundleInsertionTransaction<'_>> {
        let record = self.default_node_record(entity, NodeKind::Empty);
        self.validate_owned_node_records(std::slice::from_ref(&record))?;
        BundleInsertionTransaction::new_deferred_spawn(self, record, include_default_components)
    }

    pub(crate) fn preflight_deferred_despawn(&self, entity: EntityId) -> SceneResult<()> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("despawn", entity));
        }
        for child in self.direct_child_entity_ids(entity) {
            let mobility = self.mobility(child).unwrap_or_default();
            self.validate_bundle_mobility_state(child, None, mobility)?;
        }
        Ok(())
    }

    fn begin_bundle_spawn(
        &mut self,
        entity: EntityId,
        kind: NodeKind,
    ) -> SceneResult<BundleInsertionTransaction<'_>> {
        let record = self.default_node_record(entity, kind);
        self.validate_owned_node_records(std::slice::from_ref(&record))?;
        BundleInsertionTransaction::new_spawn(self, record)
    }
}
