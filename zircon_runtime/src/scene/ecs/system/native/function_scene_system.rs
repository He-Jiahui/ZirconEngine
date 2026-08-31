use crate::scene::World;
use crate::scene::ecs::{
    DeferredSystemKey, SceneSystem, SceneSystemMetadata, SystemParam, SystemParamAccess,
    SystemParamError, SystemState, WorkerCommandBuffer, WorldlessSystemParam,
};
use std::marker::PhantomData;

pub struct FunctionSceneSystem<P, F>
where
    P: SystemParam,
{
    metadata: SceneSystemMetadata,
    state: SystemState<P>,
    system: F,
    _marker: PhantomData<fn() -> P>,
}

impl<P, F> FunctionSceneSystem<P, F>
where
    P: SystemParam,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    pub fn new(
        metadata: SceneSystemMetadata,
        world: &mut World,
        system: F,
    ) -> Result<Self, SystemParamError> {
        let state = SystemState::<P>::new(world)?;
        Ok(Self {
            metadata,
            state,
            system,
            _marker: PhantomData,
        })
    }
}

impl<P, F> SceneSystem for FunctionSceneSystem<P, F>
where
    P: SystemParam + 'static,
    P::State: Send,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn metadata(&self) -> &SceneSystemMetadata {
        &self.metadata
    }

    fn access(&self) -> &SystemParamAccess {
        self.state.access()
    }

    fn run(&mut self, world: &mut World) {
        self.state.run(world, |params| (self.system)(params));
    }

    fn retire(&mut self, world: &mut World) {
        self.state.retire(world);
    }

    fn bind_deferred_system_key(&mut self, key: DeferredSystemKey) {
        if let Some(buffer) = self.state.deferred_command_buffer_mut() {
            buffer.bind_compiled_key(key);
        }
    }
}

/// A typed system whose parameter composition can execute without borrowing
/// World. Only the explicit worldless registration path constructs it.
pub struct WorldlessFunctionSceneSystem<P, F>
where
    P: WorldlessSystemParam,
{
    metadata: SceneSystemMetadata,
    state: SystemState<P>,
    system: F,
    _marker: PhantomData<fn() -> P>,
}

impl<P, F> WorldlessFunctionSceneSystem<P, F>
where
    P: WorldlessSystemParam,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    pub fn new(
        metadata: SceneSystemMetadata,
        world: &mut World,
        system: F,
    ) -> Result<Self, SystemParamError> {
        Ok(Self {
            metadata,
            state: SystemState::<P>::new(world)?,
            system,
            _marker: PhantomData,
        })
    }
}

impl<P, F> SceneSystem for WorldlessFunctionSceneSystem<P, F>
where
    P: WorldlessSystemParam + 'static,
    P::State: Send,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn metadata(&self) -> &SceneSystemMetadata {
        &self.metadata
    }

    fn access(&self) -> &SystemParamAccess {
        self.state.access()
    }

    fn run(&mut self, world: &mut World) {
        self.state.run(world, |params| (self.system)(params));
    }

    fn retire(&mut self, world: &mut World) {
        self.state.retire(world);
    }

    fn run_without_world(&mut self) {
        self.state.run_without_world(|params| (self.system)(params));
    }

    fn supports_worldless_execution(&self) -> bool {
        true
    }

    fn worker_command_buffer_mut(&mut self) -> Option<&mut WorkerCommandBuffer> {
        self.state.deferred_command_buffer_mut()
    }

    fn bind_deferred_system_key(&mut self, key: DeferredSystemKey) {
        if let Some(buffer) = self.state.deferred_command_buffer_mut() {
            buffer.bind_compiled_key(key);
        }
    }
}
