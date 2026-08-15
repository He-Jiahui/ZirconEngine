use super::super::QueuedStructuralCommand;
use super::super::{DeferredStructuralKind, DeferredStructuralMetadata};
use crate::scene::ecs::{
    Bundle, Command, CommandQueue, Component, DeferredCommandError, DeferredCommandOperation,
    DeferredEntity, DeferredEntityRef, DeferredSpawnToken, DeferredSystemKey, FnCommand, Resource,
};
use crate::scene::{EntityId, SceneError, World};

use super::entity_commands::EntityCommands;

pub struct Commands<'world> {
    queue: &'world mut CommandQueue,
    key: DeferredSystemKey,
    spawn_generation: u64,
    next_spawn_ordinal: &'world mut u32,
}

impl<'world> Commands<'world> {
    pub(crate) fn new(
        queue: &'world mut CommandQueue,
        key: DeferredSystemKey,
        spawn_generation: u64,
        next_spawn_ordinal: &'world mut u32,
    ) -> Self {
        Self {
            queue,
            key,
            spawn_generation,
            next_spawn_ordinal,
        }
    }

    pub fn queue(&mut self, command: impl Command) {
        self.queue.push(command);
    }

    pub fn queue_fn(&mut self, command: impl FnOnce(&mut World) + Send + 'static) {
        self.queue.push(FnCommand::new(command));
    }

    pub fn spawn_empty(&mut self) -> EntityCommands<'_> {
        let token = self.next_spawn_token();
        self.queue.push_structural(SpawnEmptyCommand {
            token: token.clone(),
        });
        self.spawned_entity_commands(token)
    }

    pub fn spawn<B>(&mut self, bundle: B) -> EntityCommands<'_>
    where
        B: Bundle,
    {
        let token = self.next_spawn_token();
        self.queue.push_structural(SpawnBundleCommand {
            token: token.clone(),
            bundle,
        });
        self.spawned_entity_commands(token)
    }

    pub fn entity(&mut self, entity: EntityId) -> EntityCommands<'_> {
        EntityCommands::existing(entity, self.reborrow())
    }

    pub fn entity_deferred(&mut self, entity: &DeferredEntity) -> EntityCommands<'_> {
        EntityCommands::spawned(entity.clone(), self.reborrow())
    }

    pub fn despawn(&mut self, entity: EntityId) {
        self.queue.push_structural(DespawnCommand {
            target: DeferredEntityRef::existing(entity),
        });
    }

    pub fn insert<T>(&mut self, entity: EntityId, component: T)
    where
        T: Component,
    {
        self.queue.push_structural(InsertBundleCommand {
            target: DeferredEntityRef::existing(entity),
            bundle: (component,),
            operation: DeferredCommandOperation::Insert,
        });
    }

    pub fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B)
    where
        B: Bundle,
    {
        self.queue_insert_bundle(DeferredEntityRef::existing(entity), bundle);
    }

    pub fn remove<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        self.queue_remove::<T>(DeferredEntityRef::existing(entity));
    }

    pub fn insert_resource<T>(&mut self, resource: T)
    where
        T: Resource,
    {
        self.queue_fn(move |world| {
            world.insert_resource(resource);
        });
    }

    pub fn remove_resource<T>(&mut self)
    where
        T: Resource,
    {
        self.queue_fn(move |world| {
            let _ = world.remove_resource::<T>();
        });
    }

    pub(super) fn queue_insert_bundle<B>(&mut self, target: DeferredEntityRef, bundle: B)
    where
        B: Bundle,
    {
        self.queue.push_structural(InsertBundleCommand {
            target,
            bundle,
            operation: DeferredCommandOperation::InsertBundle,
        });
    }

    pub(super) fn queue_remove<T>(&mut self, target: DeferredEntityRef)
    where
        T: Component,
    {
        self.queue.push_structural(RemoveComponentCommand::<T> {
            target,
            marker: std::marker::PhantomData,
        });
    }

    pub(super) fn queue_despawn(&mut self, target: DeferredEntityRef) {
        self.queue.push_structural(DespawnCommand { target });
    }

    fn next_spawn_token(&mut self) -> DeferredSpawnToken {
        let ordinal = *self.next_spawn_ordinal;
        *self.next_spawn_ordinal = self
            .next_spawn_ordinal
            .checked_add(1)
            .expect("deferred spawn ordinal exhausted");
        DeferredSpawnToken::new(self.key.clone(), self.spawn_generation, ordinal)
    }

    fn spawned_entity_commands(&mut self, token: DeferredSpawnToken) -> EntityCommands<'_> {
        EntityCommands::spawned(DeferredEntity::new(token), self.reborrow())
    }

    fn reborrow(&mut self) -> Commands<'_> {
        Commands::new(
            self.queue,
            self.key.clone(),
            self.spawn_generation,
            self.next_spawn_ordinal,
        )
    }
}

impl std::fmt::Debug for Commands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Commands").finish_non_exhaustive()
    }
}

struct SpawnEmptyCommand {
    token: DeferredSpawnToken,
}

impl Command for SpawnEmptyCommand {
    fn apply(self, world: &mut World) {
        let target = DeferredEntityRef::spawned(self.token.clone());
        let Some(entity) = world.resolve_deferred_entity_ref(&target) else {
            world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Spawn,
                world.deferred_command_target(&target),
                SceneError::EntityIdExhausted { entity: u64::MAX },
            ));
            return;
        };
        match world.spawn_empty_at(entity) {
            Ok(true) => world.mark_deferred_spawn_published(self.token),
            Ok(false) => world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Spawn,
                world.deferred_command_target(&target),
                SceneError::DuplicateEntity { entity },
            )),
            Err(error) => world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Spawn,
                world.deferred_command_target(&target),
                error,
            )),
        }
    }
}

impl QueuedStructuralCommand for SpawnEmptyCommand {
    fn structural_metadata(&self) -> DeferredStructuralMetadata {
        DeferredStructuralMetadata::new(
            DeferredEntityRef::spawned(self.token.clone()),
            DeferredStructuralKind::SpawnEmpty,
            DeferredCommandOperation::Spawn,
        )
    }

    fn stage_into_batch(
        self,
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut World,
    ) {
        batch.stage_empty_spawn(world, self.structural_metadata());
    }
}

struct SpawnBundleCommand<B> {
    token: DeferredSpawnToken,
    bundle: B,
}

impl<B> Command for SpawnBundleCommand<B>
where
    B: Bundle,
{
    fn apply(self, world: &mut World) {
        let target = DeferredEntityRef::spawned(self.token.clone());
        let Some(entity) = world.resolve_deferred_entity_ref(&target) else {
            world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Spawn,
                world.deferred_command_target(&target),
                SceneError::EntityIdExhausted { entity: u64::MAX },
            ));
            return;
        };
        match world.spawn_at(entity, self.bundle) {
            Ok(_) => world.mark_deferred_spawn_published(self.token),
            Err(error) => world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Spawn,
                world.deferred_command_target(&target),
                error,
            )),
        }
    }
}

impl<B> QueuedStructuralCommand for SpawnBundleCommand<B>
where
    B: Bundle,
{
    fn structural_metadata(&self) -> DeferredStructuralMetadata {
        DeferredStructuralMetadata::new(
            DeferredEntityRef::spawned(self.token.clone()),
            DeferredStructuralKind::SpawnBundle,
            DeferredCommandOperation::Spawn,
        )
    }

    fn stage_into_batch(
        self,
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut World,
    ) {
        let metadata = DeferredStructuralMetadata::new(
            DeferredEntityRef::spawned(self.token),
            DeferredStructuralKind::SpawnBundle,
            DeferredCommandOperation::Spawn,
        );
        batch.stage_bundle(world, metadata, self.bundle);
    }
}

struct InsertBundleCommand<B> {
    target: DeferredEntityRef,
    bundle: B,
    operation: DeferredCommandOperation,
}

impl<B> Command for InsertBundleCommand<B>
where
    B: Bundle,
{
    fn apply(self, world: &mut World) {
        let Some(entity) = world.resolve_deferred_entity_ref(&self.target) else {
            return;
        };
        if let Err(error) = world.insert_bundle(entity, self.bundle) {
            world.record_deferred_command_error(DeferredCommandError::new(
                self.operation,
                world.deferred_command_target(&self.target),
                error,
            ));
        }
    }
}

impl<B> QueuedStructuralCommand for InsertBundleCommand<B>
where
    B: Bundle,
{
    fn structural_metadata(&self) -> DeferredStructuralMetadata {
        DeferredStructuralMetadata::new(
            self.target.clone(),
            DeferredStructuralKind::InsertBundle,
            self.operation,
        )
    }

    fn stage_into_batch(
        self,
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut World,
    ) {
        let metadata = DeferredStructuralMetadata::new(
            self.target,
            DeferredStructuralKind::InsertBundle,
            self.operation,
        );
        batch.stage_bundle(world, metadata, self.bundle);
    }
}

struct RemoveComponentCommand<T> {
    target: DeferredEntityRef,
    marker: std::marker::PhantomData<T>,
}

impl<T> Command for RemoveComponentCommand<T>
where
    T: Component,
{
    fn apply(self, world: &mut World) {
        let Some(entity) = world.resolve_deferred_entity_ref(&self.target) else {
            return;
        };
        if let Err(error) = world.remove::<T>(entity) {
            world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Remove,
                world.deferred_command_target(&self.target),
                error,
            ));
        }
    }
}

impl<T> QueuedStructuralCommand for RemoveComponentCommand<T>
where
    T: Component,
{
    fn structural_metadata(&self) -> DeferredStructuralMetadata {
        DeferredStructuralMetadata::new(
            self.target.clone(),
            DeferredStructuralKind::RemoveComponent,
            DeferredCommandOperation::Remove,
        )
    }

    fn stage_into_batch(
        self,
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut World,
    ) {
        let metadata = self.structural_metadata();
        batch.stage_remove::<T>(world, metadata);
    }
}

struct DespawnCommand {
    target: DeferredEntityRef,
}

impl Command for DespawnCommand {
    fn apply(self, world: &mut World) {
        let Some(entity) = world.resolve_deferred_entity_ref(&self.target) else {
            return;
        };
        if let Err(error) = world.remove_entity(entity) {
            world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Despawn,
                world.deferred_command_target(&self.target),
                error,
            ));
        }
    }
}

impl QueuedStructuralCommand for DespawnCommand {
    fn structural_metadata(&self) -> DeferredStructuralMetadata {
        DeferredStructuralMetadata::new(
            self.target.clone(),
            DeferredStructuralKind::Despawn,
            DeferredCommandOperation::Despawn,
        )
    }

    fn stage_into_batch(
        self,
        batch: &mut crate::scene::world::DeferredStructuralBatch,
        world: &mut World,
    ) {
        batch.stage_despawn(world, self.structural_metadata());
    }
}
