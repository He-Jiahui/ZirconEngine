use crate::scene::ecs::{
    BoxedSceneSystem, FunctionSceneSystem, SceneSystemMetadata, SystemParam, SystemParamError,
};
use crate::scene::World;

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
