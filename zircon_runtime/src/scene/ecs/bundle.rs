use crate::scene::{EntityId, SceneResult, World};

use super::Component;

pub trait Bundle: 'static + Send + Sync {
    fn insert_into(self, world: &mut World, entity: EntityId) -> SceneResult<()>;

    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging;
}

/// Receives one fully preflighted bundle without exposing an intermediate
/// archetype signature to lifecycle observers.
pub trait BundleStaging {
    fn stage<T>(&mut self, component: &T) -> SceneResult<()>
    where
        T: Component;

    fn validate_final_state(&self) -> SceneResult<()>;

    fn commit<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component;
}

macro_rules! tuple_bundle {
    ($($name:ident),*) => {
        impl<$($name),*> Bundle for ($($name,)*)
        where
            $($name: Component,)*
        {
            #[allow(non_snake_case)]
            fn insert_into(self, world: &mut World, entity: EntityId) -> SceneResult<()> {
                let mut transaction = world.begin_bundle_insertion(entity)?;
                self.stage_into(&mut transaction)?;
                transaction.finish()
            }

            #[allow(non_snake_case)]
            fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
            where
                S: BundleStaging,
            {
                let ($($name,)*) = self;
                $(staging.stage(&$name)?;)*
                staging.validate_final_state()?;
                $(staging.commit($name)?;)*
                Ok(())
            }
        }
    };
}

impl Bundle for () {
    fn insert_into(self, _world: &mut World, _entity: EntityId) -> SceneResult<()> {
        Ok(())
    }

    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging,
    {
        staging.validate_final_state()
    }
}

tuple_bundle!(A);
tuple_bundle!(A, B);
tuple_bundle!(A, B, C);
tuple_bundle!(A, B, C, D);
tuple_bundle!(A, B, C, D, E);
tuple_bundle!(A, B, C, D, E, F);
tuple_bundle!(A, B, C, D, E, F, G);
tuple_bundle!(A, B, C, D, E, F, G, H);
