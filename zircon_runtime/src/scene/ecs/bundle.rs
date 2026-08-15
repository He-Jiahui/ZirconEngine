use crate::scene::SceneResult;

use super::Component;

pub trait Bundle: 'static + Send + Sync {
    fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
    where
        S: BundleStaging;
}

/// Receives one fully preflighted bundle without exposing an intermediate
/// archetype signature to lifecycle observers.
///
/// Staging takes ownership of each value. This binds storage/schema
/// preflight to the exact value that the transaction will later publish.
pub trait BundleStaging {
    fn stage<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component;

    fn validate_final_state(&self) -> SceneResult<()>;
}

macro_rules! tuple_bundle {
    ($($name:ident),*) => {
        impl<$($name),*> Bundle for ($($name,)*)
        where
            $($name: Component,)*
        {
            #[allow(non_snake_case)]
            fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>
            where
                S: BundleStaging,
            {
                let ($($name,)*) = self;
                $(staging.stage($name)?;)*
                staging.validate_final_state()
            }
        }
    };
}

impl Bundle for () {
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
