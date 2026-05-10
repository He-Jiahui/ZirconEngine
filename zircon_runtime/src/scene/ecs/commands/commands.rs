use crate::scene::ecs::{
    Bundle, ChangeTickWindow, Command, CommandQueue, Component, FnCommand, Resource, SystemParam,
    SystemParamAccess, SystemParamError,
};
use crate::scene::{EntityId, World};

pub struct Commands<'world> {
    queue: &'world mut CommandQueue,
    next_entity: &'world mut EntityId,
}

pub struct EntityCommands<'world> {
    entity: EntityId,
    commands: Commands<'world>,
}

pub struct CommandsParam;

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
        self.queue_fn(move |world| {
            world.spawn_empty_at(entity);
        });
        self.entity(entity)
    }

    pub fn spawn<B>(&mut self, bundle: B) -> EntityCommands<'_>
    where
        B: Bundle,
    {
        let entity = self.queue_reserved_entity();
        self.queue_fn(move |world| {
            let _ = world.spawn_at(entity, bundle);
        });
        self.entity(entity)
    }

    pub fn entity(&mut self, entity: EntityId) -> EntityCommands<'_> {
        EntityCommands {
            entity,
            commands: self.reborrow(),
        }
    }

    pub fn entity_or_spawn(&mut self, entity: EntityId) -> EntityCommands<'_> {
        self.queue_fn(move |world| {
            world.spawn_empty_at(entity);
        });
        self.entity(entity)
    }

    pub fn despawn(&mut self, entity: EntityId) {
        self.queue_fn(move |world| {
            let _ = world.remove_entity(entity);
        });
    }

    pub fn insert<T>(&mut self, entity: EntityId, component: T)
    where
        T: Component,
    {
        self.queue_fn(move |world| {
            let _ = world.insert(entity, component);
        });
    }

    pub fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B)
    where
        B: Bundle,
    {
        self.queue_fn(move |world| {
            let _ = world.insert_bundle(entity, bundle);
        });
    }

    pub fn remove<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        self.queue_fn(move |world| {
            let _ = world.remove::<T>(entity);
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

impl EntityCommands<'_> {
    pub fn id(&self) -> EntityId {
        self.entity
    }

    pub fn insert<B>(&mut self, bundle: B) -> &mut Self
    where
        B: Bundle,
    {
        let entity = self.entity;
        self.commands.queue_fn(move |world| {
            let _ = world.insert_bundle(entity, bundle);
        });
        self
    }

    pub fn remove<T>(&mut self) -> &mut Self
    where
        T: Component,
    {
        let entity = self.entity;
        self.commands.queue_fn(move |world| {
            let _ = world.remove::<T>(entity);
        });
        self
    }

    pub fn despawn(&mut self) -> &mut Self {
        let entity = self.entity;
        self.commands.queue_fn(move |world| {
            let _ = world.remove_entity(entity);
        });
        self
    }
}

impl std::fmt::Debug for Commands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Commands").finish_non_exhaustive()
    }
}

impl std::fmt::Debug for EntityCommands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityCommands")
            .field("entity", &self.entity)
            .finish_non_exhaustive()
    }
}

impl Default for CommandsParam {
    fn default() -> Self {
        Self
    }
}

impl SystemParam for CommandsParam {
    type State = ();
    type Item<'world> = Commands<'world>;

    fn init_state(
        _world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_deferred_commands();
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &mut *world;
        let (queue, next_entity) = world.command_state_mut();
        Commands::new(queue, next_entity)
    }
}
