use crate::scene::EntityId;
use crate::scene::ecs::{Bundle, Component, DeferredEntity, DeferredEntityRef};

use super::facade::Commands;

pub struct EntityCommands<'world> {
    target: DeferredEntityRef,
    deferred_entity: Option<DeferredEntity>,
    commands: Commands<'world>,
}

impl<'world> EntityCommands<'world> {
    pub(super) fn existing(entity: EntityId, commands: Commands<'world>) -> Self {
        Self {
            target: DeferredEntityRef::existing(entity),
            deferred_entity: None,
            commands,
        }
    }

    pub(super) fn spawned(entity: DeferredEntity, commands: Commands<'world>) -> Self {
        Self {
            target: DeferredEntityRef::spawned(entity.token().clone()),
            deferred_entity: Some(entity),
            commands,
        }
    }

    /// Consumes a spawned command builder and returns its explicit deferred
    /// handle. Existing-entity builders do not have a pending spawn handle.
    pub fn into_deferred_entity(self) -> DeferredEntity {
        self.deferred_entity
            .expect("only a spawned EntityCommands has a deferred entity handle")
    }

    pub fn insert<B>(&mut self, bundle: B) -> &mut Self
    where
        B: Bundle,
    {
        self.commands
            .queue_insert_bundle(self.target.clone(), bundle);
        self
    }

    pub fn remove<T>(&mut self) -> &mut Self
    where
        T: Component,
    {
        self.commands.queue_remove::<T>(self.target.clone());
        self
    }

    pub fn despawn(&mut self) -> &mut Self {
        self.commands.queue_despawn(self.target.clone());
        self
    }
}

impl std::fmt::Debug for EntityCommands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityCommands")
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}
