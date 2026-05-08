use crate::scene::ecs::{
    Bundle, ChangeTickWindow, CommandQueue, Component, Resource, SystemParam, SystemParamAccess,
    SystemParamError,
};
use crate::scene::{EntityId, World};

pub struct Commands<'world> {
    queue: &'world mut CommandQueue,
}

pub struct CommandsParam;

impl<'world> Commands<'world> {
    pub(crate) fn new(queue: &'world mut CommandQueue) -> Self {
        Self { queue }
    }

    pub fn queue(&mut self, command: impl FnOnce(&mut World) + Send + 'static) {
        self.queue.push(command);
    }

    pub fn spawn<B>(&mut self, bundle: B)
    where
        B: Bundle,
    {
        self.queue(|world| {
            let _ = world.spawn(bundle);
        });
    }

    pub fn despawn(&mut self, entity: EntityId) {
        self.queue(move |world| {
            let _ = world.remove_entity(entity);
        });
    }

    pub fn insert<T>(&mut self, entity: EntityId, component: T)
    where
        T: Component,
    {
        self.queue(move |world| {
            let _ = world.insert(entity, component);
        });
    }

    pub fn remove<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        self.queue(move |world| {
            let _ = world.remove::<T>(entity);
        });
    }

    pub fn insert_resource<T>(&mut self, resource: T)
    where
        T: Resource,
    {
        self.queue(move |world| {
            world.insert_resource(resource);
        });
    }

    pub fn remove_resource<T>(&mut self)
    where
        T: Resource,
    {
        self.queue(move |world| {
            let _ = world.remove_resource::<T>();
        });
    }
}

impl std::fmt::Debug for Commands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Commands").finish_non_exhaustive()
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
        Commands::new(world.command_queue_mut())
    }
}
