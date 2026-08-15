use crate::scene::ecs::{
    Bundle, DeferredCommandError, DeferredCommandOperation, DeferredCommandTarget,
    DeferredEntityRef, DeferredSpawnToken, DeferredStructuralMetadata,
};
use crate::scene::{EntityId, SceneError, SceneResult, World};

use super::typed_api::{BundleInsertionTransaction, DeferredBundleTransactionArtifact};

/// Owns one maximal contiguous structural barrier. Its transactions detach
/// from World between individual commands, so every target can be preflighted
/// before this barrier publishes its first final row.
pub(crate) struct DeferredStructuralBatch {
    segments: Vec<DeferredStructuralSegment>,
    next_sequence: usize,
}

struct DeferredStructuralSegment {
    target: DeferredEntityRef,
    deferred_target: DeferredCommandTarget,
    spawn_token: Option<DeferredSpawnToken>,
    artifact: Option<DeferredBundleTransactionArtifact>,
    errors: Vec<(usize, DeferredCommandError)>,
    last_operation: DeferredCommandOperation,
    last_sequence: usize,
    aborted: bool,
    despawned: bool,
}

impl DeferredStructuralBatch {
    pub(crate) fn new() -> Self {
        Self {
            segments: Vec::new(),
            next_sequence: 0,
        }
    }

    pub(crate) fn stage_empty_spawn(
        &mut self,
        world: &mut World,
        metadata: DeferredStructuralMetadata,
    ) {
        let sequence = self.next_sequence();
        self.segment_mut(world, &metadata, sequence)
            .stage_empty_spawn(metadata, sequence);
    }

    pub(crate) fn stage_bundle<B>(
        &mut self,
        world: &mut World,
        metadata: DeferredStructuralMetadata,
        bundle: B,
    ) where
        B: Bundle,
    {
        let sequence = self.next_sequence();
        self.segment_mut(world, &metadata, sequence)
            .stage_bundle(world, metadata, bundle, sequence);
    }

    pub(crate) fn stage_remove<T>(
        &mut self,
        world: &mut World,
        metadata: DeferredStructuralMetadata,
    ) where
        T: crate::scene::ecs::Component,
    {
        let sequence = self.next_sequence();
        self.segment_mut(world, &metadata, sequence)
            .stage_remove::<T>(world, metadata, sequence);
    }

    pub(crate) fn stage_despawn(
        &mut self,
        world: &mut World,
        metadata: DeferredStructuralMetadata,
    ) {
        let sequence = self.next_sequence();
        self.segment_mut(world, &metadata, sequence)
            .stage_despawn(metadata, sequence);
    }

    /// Performs the complete segment preflight before descriptor materialization
    /// or entity publication. A failure drops every staged payload in the
    /// barrier and leaves World rows, lifecycle, and derived state untouched.
    pub(crate) fn finish(mut self, world: &mut World) -> Vec<DeferredCommandError> {
        for segment in &mut self.segments {
            segment.preflight(world);
        }
        let despawned_entities = self
            .segments
            .iter()
            .filter(|segment| segment.despawned && !segment.aborted)
            .filter_map(DeferredStructuralSegment::resolved_entity_if_existing)
            .collect::<std::collections::BTreeSet<_>>();
        for segment in &mut self.segments {
            segment.preflight_batch_relationships(&despawned_entities);
        }
        if self.has_errors() {
            return self.take_errors();
        }

        // Materializing descriptors is deliberately after full preflight. It
        // cannot fail because the artifacts retain the exact typed registrar
        // and will only publish values they already validated.
        for segment in &mut self.segments {
            segment.materialize_component_reservations(world);
        }
        for segment in &mut self.segments {
            segment.publish(world);
        }
        self.take_errors()
    }

    fn next_sequence(&mut self) -> usize {
        let sequence = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .expect("deferred structural command sequence exhausted");
        sequence
    }

    fn segment_mut(
        &mut self,
        world: &mut World,
        metadata: &DeferredStructuralMetadata,
        sequence: usize,
    ) -> &mut DeferredStructuralSegment {
        if let Some(index) = self.segments.iter().position(|segment| {
            segment.target == *metadata.target()
                && (segment.artifact.is_some() || segment.aborted || segment.despawned)
        }) {
            return &mut self.segments[index];
        }
        self.segments
            .push(DeferredStructuralSegment::begin(world, metadata, sequence));
        self.segments
            .last_mut()
            .expect("new deferred structural segment must be retained")
    }

    fn has_errors(&self) -> bool {
        self.segments
            .iter()
            .any(|segment| !segment.errors.is_empty())
    }

    fn take_errors(&mut self) -> Vec<DeferredCommandError> {
        let mut errors = self
            .segments
            .iter_mut()
            .flat_map(|segment| segment.errors.drain(..))
            .collect::<Vec<_>>();
        errors.sort_by_key(|(sequence, _)| *sequence);
        errors.into_iter().map(|(_, error)| error).collect()
    }
}

impl DeferredStructuralSegment {
    fn begin(world: &mut World, metadata: &DeferredStructuralMetadata, sequence: usize) -> Self {
        let target = metadata.target().clone();
        let deferred_target = world.deferred_command_target(&target);
        let spawn_token = match &target {
            DeferredEntityRef::Existing(_) => None,
            DeferredEntityRef::Spawn(token) => Some(token.clone()),
        };
        let artifact = Self::begin_transaction(world, metadata)
            .map(BundleInsertionTransaction::into_deferred_artifact);
        match artifact {
            Ok(artifact) => Self {
                target,
                deferred_target,
                spawn_token,
                artifact: Some(artifact),
                errors: Vec::new(),
                last_operation: metadata.operation(),
                last_sequence: sequence,
                aborted: false,
                despawned: false,
            },
            Err(error) => Self {
                target,
                deferred_target: deferred_target.clone(),
                spawn_token,
                artifact: None,
                errors: vec![(
                    sequence,
                    DeferredCommandError::new(metadata.operation(), deferred_target, error),
                )],
                last_operation: metadata.operation(),
                last_sequence: sequence,
                // A target that failed to resolve stays per-command so each
                // following operation receives its own ordered error.
                aborted: false,
                despawned: false,
            },
        }
    }

    fn stage_empty_spawn(&mut self, metadata: DeferredStructuralMetadata, sequence: usize) {
        self.last_operation = metadata.operation();
        self.last_sequence = sequence;
        self.reject_after_despawn(&metadata, sequence);
    }

    fn stage_bundle<B>(
        &mut self,
        world: &mut World,
        metadata: DeferredStructuralMetadata,
        bundle: B,
        sequence: usize,
    ) where
        B: Bundle,
    {
        self.last_operation = metadata.operation();
        self.last_sequence = sequence;
        if self.aborted || self.reject_after_despawn(&metadata, sequence) {
            return;
        }
        let Some(artifact) = self.artifact.take() else {
            return;
        };
        let mut transaction = BundleInsertionTransaction::from_deferred_artifact(world, artifact);
        match transaction.stage_deferred_bundle(bundle) {
            Ok(()) => self.artifact = Some(transaction.into_deferred_artifact()),
            Err(error) => {
                self.push_error(sequence, metadata.operation(), error);
                self.aborted = true;
            }
        }
    }

    fn stage_remove<T>(
        &mut self,
        world: &mut World,
        metadata: DeferredStructuralMetadata,
        sequence: usize,
    ) where
        T: crate::scene::ecs::Component,
    {
        self.last_operation = metadata.operation();
        self.last_sequence = sequence;
        if self.aborted || self.reject_after_despawn(&metadata, sequence) {
            return;
        }
        let Some(artifact) = self.artifact.take() else {
            return;
        };
        let mut transaction = BundleInsertionTransaction::from_deferred_artifact(world, artifact);
        match transaction.stage_deferred_remove::<T>() {
            Ok(()) => self.artifact = Some(transaction.into_deferred_artifact()),
            Err(error) => {
                self.push_error(sequence, metadata.operation(), error);
                self.aborted = true;
            }
        }
    }

    fn stage_despawn(&mut self, metadata: DeferredStructuralMetadata, sequence: usize) {
        self.last_operation = metadata.operation();
        self.last_sequence = sequence;
        if self.aborted || self.reject_after_despawn(&metadata, sequence) {
            return;
        }
        if self.artifact.is_some() {
            self.despawned = true;
        }
    }

    fn preflight(&mut self, world: &World) {
        if self.aborted || !self.errors.is_empty() {
            return;
        }
        let Some(artifact) = self.artifact.as_mut() else {
            return;
        };
        let result = if self.despawned {
            artifact.preflight_despawn(world)
        } else {
            artifact.preflight(world)
        };
        if let Err(error) = result {
            self.push_error(self.last_sequence, self.last_operation, error);
            self.aborted = true;
        }
    }

    fn materialize_component_reservations(&mut self, world: &mut World) {
        if self.aborted || self.despawned {
            return;
        }
        if let Some(artifact) = self.artifact.as_mut() {
            artifact.materialize_component_reservations(world);
        }
    }

    fn preflight_batch_relationships(
        &mut self,
        despawned_entities: &std::collections::BTreeSet<EntityId>,
    ) {
        if self.aborted || self.despawned || !self.errors.is_empty() {
            return;
        }
        let Some(artifact) = self.artifact.as_ref() else {
            return;
        };
        if let Err(error) = artifact.preflight_batch_relationships(despawned_entities) {
            self.push_error(self.last_sequence, self.last_operation, error);
            self.aborted = true;
        }
    }

    fn publish(&mut self, world: &mut World) {
        if self.aborted {
            return;
        }
        let Some(artifact) = self.artifact.take() else {
            return;
        };
        let result = if self.despawned {
            artifact.publish_despawn(world)
        } else {
            artifact.publish_spawn(world, self.spawn_token.clone())
        };
        // `finish` preflights every artifact before descriptor materialization or
        // the first row publication. A recoverable error here would otherwise
        // report a partial batch after earlier rows already became visible.
        result.expect("a fully preflighted deferred structural row must publish");
    }

    fn reject_after_despawn(
        &mut self,
        metadata: &DeferredStructuralMetadata,
        sequence: usize,
    ) -> bool {
        if !self.despawned {
            return false;
        }
        self.push_error(
            sequence,
            metadata.operation(),
            SceneError::missing_entity("deferred command after despawn", self.resolved_entity()),
        );
        true
    }

    fn resolved_entity(&self) -> EntityId {
        match &self.deferred_target {
            DeferredCommandTarget::Resolved(entity) => *entity,
            DeferredCommandTarget::Pending(_) => EntityId::MAX,
        }
    }

    fn resolved_entity_if_existing(&self) -> Option<EntityId> {
        match &self.target {
            DeferredEntityRef::Existing(entity) => Some(*entity),
            DeferredEntityRef::Spawn(_) => None,
        }
    }

    fn push_error(
        &mut self,
        sequence: usize,
        operation: DeferredCommandOperation,
        error: SceneError,
    ) {
        self.errors.push((
            sequence,
            DeferredCommandError::new(operation, self.deferred_target.clone(), error),
        ));
    }

    fn begin_transaction<'world>(
        world: &'world mut World,
        metadata: &DeferredStructuralMetadata,
    ) -> SceneResult<BundleInsertionTransaction<'world>> {
        let entity = world.resolve_deferred_entity_ref(metadata.target()).ok_or(
            SceneError::EntityIdExhausted {
                entity: EntityId::MAX,
            },
        )?;
        match metadata.kind() {
            crate::scene::ecs::DeferredStructuralKind::SpawnEmpty => {
                world.begin_deferred_bundle_spawn(entity, false)
            }
            crate::scene::ecs::DeferredStructuralKind::SpawnBundle => {
                world.begin_deferred_bundle_spawn(entity, true)
            }
            crate::scene::ecs::DeferredStructuralKind::InsertBundle
            | crate::scene::ecs::DeferredStructuralKind::RemoveComponent
            | crate::scene::ecs::DeferredStructuralKind::Despawn => {
                world.begin_deferred_bundle_insertion(entity)
            }
        }
    }
}
