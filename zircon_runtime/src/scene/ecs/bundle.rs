use crate::scene::SceneResult;

use super::Component;

/// 把一组组件交给 World 的插入事务；实现应先完成整组预检，再使任何结构变化对观察者可见。
/// 直接插入与延迟命令均通过 stage_into 进入同一 BundleStaging 契约。
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
