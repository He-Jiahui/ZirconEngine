use crate::scene::World;
use crate::scene::ecs::{
    BoxedSceneSystem, FunctionSceneSystem, SceneSystemMetadata, SystemParam, SystemParamError,
    WorldlessFunctionSceneSystem, WorldlessSystemParam,
};

pub trait IntoSceneSystem<P>
where
    P: SystemParam,
{
    fn into_scene_system(
        self,
        metadata: SceneSystemMetadata,
        world: &mut World,
    ) -> Result<BoxedSceneSystem, SystemParamError>;
}

pub trait IntoWorldlessSceneSystem<P>
where
    P: WorldlessSystemParam,
{
    fn into_worldless_scene_system(
        self,
        metadata: SceneSystemMetadata,
        world: &mut World,
    ) -> Result<BoxedSceneSystem, SystemParamError>;
}

impl<P, F> IntoSceneSystem<P> for F
where
    P: SystemParam + 'static,
    P::State: Send,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn into_scene_system(
        self,
        metadata: SceneSystemMetadata,
        world: &mut World,
    ) -> Result<BoxedSceneSystem, SystemParamError> {
        Ok(Box::new(FunctionSceneSystem::<P, F>::new(
            metadata, world, self,
        )?))
    }
}

impl<P, F> IntoWorldlessSceneSystem<P> for F
where
    P: WorldlessSystemParam + 'static,
    P::State: Send,
    F: for<'world> FnMut(P::Item<'world>) + Send + 'static,
{
    fn into_worldless_scene_system(
        self,
        metadata: SceneSystemMetadata,
        world: &mut World,
    ) -> Result<BoxedSceneSystem, SystemParamError> {
        Ok(Box::new(WorldlessFunctionSceneSystem::<P, F>::new(
            metadata, world, self,
        )?))
    }
}
