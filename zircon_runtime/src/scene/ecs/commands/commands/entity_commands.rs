use crate::scene::ecs::{Bundle, Component, DeferredCommandError, DeferredCommandOperation};
use crate::scene::EntityId;

use super::facade::Commands;

pub struct EntityCommands<'world> {
    entity: EntityId,
    commands: Commands<'world>,
}

impl<'world> EntityCommands<'world> {
    pub(super) fn new(entity: EntityId, commands: Commands<'world>) -> Self {
        Self { entity, commands }
    }

    pub fn id(&self) -> EntityId {
        self.entity
    }

    pub fn insert<B>(&mut self, bundle: B) -> &mut Self
    where
        B: Bundle,
    {
        let entity = self.entity;
        self.commands.queue_fn(move |world| {
            if let Err(error) = world.insert_bundle(entity, bundle) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::InsertBundle,
                    entity,
                    error,
                ));
            }
        });
        self
    }

    pub fn remove<T>(&mut self) -> &mut Self
    where
        T: Component,
    {
        let entity = self.entity;
        self.commands.queue_fn(move |world| {
            if let Err(error) = world.remove::<T>(entity) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::Remove,
                    entity,
                    error,
                ));
            }
        });
        self
    }

    pub fn despawn(&mut self) -> &mut Self {
        let entity = self.entity;
        self.commands.queue_fn(move |world| {
            if !world.remove_entity(entity) {
                world.record_deferred_command_error(DeferredCommandError::new(
                    DeferredCommandOperation::Despawn,
                    entity,
                    format!("cannot despawn missing entity {entity}"),
                ));
            }
        });
        self
    }
}

impl std::fmt::Debug for EntityCommands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityCommands")
            .field("entity", &self.entity)
            .finish_non_exhaustive()
    }
}
