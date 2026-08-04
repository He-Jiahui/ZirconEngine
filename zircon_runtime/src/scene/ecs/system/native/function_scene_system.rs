use std::marker::PhantomData;

use crate::scene::ecs::{
    SceneSystem, SceneSystemMetadata, SystemParam, SystemParamAccess, SystemParamError, SystemState,
};
use crate::scene::World;

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
}
