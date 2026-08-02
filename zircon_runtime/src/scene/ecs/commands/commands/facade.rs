use crate::scene::ecs::{
    Bundle, Command, CommandQueue, Component, DeferredCommandError, DeferredCommandOperation,
    FnCommand, Resource,
};
use crate::scene::{EntityId, SceneError, World};

use super::entity_commands::EntityCommands;

pub struct Commands<'world> {
    queue: &'world mut CommandQueue,
    next_entity: &'world mut EntityId,
}

impl<'world> Commands<'world> {
    pub(crate) fn new(queue: &'world mut CommandQueue, next_entity: &'world mut EntityId) -> Self {
        Self { queue, next_entity }
    }

    pub fn queue(&mut self, command: impl Command) {
        self.queue.push(command);
    }

    pub fn queue_fn(&mut self, command: impl FnOnce(&mut World) + Send + 'static) {
        self.queue.push(FnCommand::new(command));
    }

    pub fn spawn_empty(&mut self) -> EntityCommands<'_> {
        let entity = self.queue_reserved_entity();
        self.queue_fn(move |world| match world.spawn_empty_at(entity) {
            Ok(true) => {}
            Ok(false) => world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Spawn,
                entity,
                SceneError::DuplicateEntity { entity }.to_string(),
            )),
            Err(error) => world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::Spawn,
                entity,
                error.to_string(),
            )),
        });
        self.entity(entity)
    }

    pub fn spawn<B>(&mut self, bundle: B) -> EntityCommands<'_>
    where
        B: Bundle,
    {
        let entity = self.queue_reserved_entity();
        self.queue_fn(move |world| {
            if let Err(error) = world.spawn_at(entity, bundle) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::Spawn,
                    entity,
                    error.to_string(),
                ));
            }
        });
        self.entity(entity)
    }

    pub fn entity(&mut self, entity: EntityId) -> EntityCommands<'_> {
        EntityCommands::new(entity, self.reborrow())
    }

    pub fn entity_or_spawn(&mut self, entity: EntityId) -> EntityCommands<'_> {
        self.queue_fn(move |world| {
            if let Err(error) = world.spawn_empty_at(entity) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::Spawn,
                    entity,
                    error.to_string(),
                ));
            }
        });
        self.entity(entity)
    }

    pub fn despawn(&mut self, entity: EntityId) {
        self.queue_fn(move |world| {
            if !world.remove_entity(entity) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::Despawn,
                    entity,
                    format!("cannot despawn missing entity {entity}"),
                ));
            }
        });
    }

    pub fn insert<T>(&mut self, entity: EntityId, component: T)
    where
        T: Component,
    {
        self.queue_fn(move |world| {
            if let Err(error) = world.insert(entity, component) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::Insert,
                    entity,
                    error.to_string(),
                ));
            }
        });
    }

    pub fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B)
    where
        B: Bundle,
    {
        self.queue_fn(move |world| {
            if let Err(error) = world.insert_bundle(entity, bundle) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::InsertBundle,
                    entity,
                    error.to_string(),
                ));
            }
        });
    }

    pub fn remove<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        self.queue_fn(move |world| {
            if let Err(error) = world.remove::<T>(entity) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::Remove,
                    entity,
                    error.to_string(),
                ));
            }
        });
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

    fn queue_reserved_entity(&mut self) -> EntityId {
        let entity = *self.next_entity;
        *self.next_entity += 1;
        entity
    }

    fn reborrow(&mut self) -> Commands<'_> {
        Commands::new(self.queue, self.next_entity)
    }
}

impl std::fmt::Debug for Commands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Commands").finish_non_exhaustive()
    }
}
