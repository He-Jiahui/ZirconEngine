use crate::scene::{EntityId, World};

use super::Component;

pub trait Bundle: 'static + Send + Sync {
    fn insert_into(self, world: &mut World, entity: EntityId) -> Result<(), String>;
}

macro_rules! tuple_bundle {
    ($($name:ident),*) => {
        impl<$($name),*> Bundle for ($($name,)*)
        where
            $($name: Component,)*
        {
            #[allow(non_snake_case)]
            fn insert_into(self, world: &mut World, entity: EntityId) -> Result<(), String> {
                let ($($name,)*) = self;
                $(world.insert(entity, $name)?;)*
                Ok(())
            }
        }
    };
}

impl Bundle for () {
    fn insert_into(self, _world: &mut World, _entity: EntityId) -> Result<(), String> {
        Ok(())
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
