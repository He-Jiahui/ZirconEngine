use crate::scene::{TypeRegistry, World};

pub(crate) fn builtin_type_registry() -> TypeRegistry {
    let mut registry = TypeRegistry::default();
    super::builtin_reflection::register(&mut registry)
        .expect("builtin derived reflection registrations must be unique and valid");
    registry
}

pub(in crate::scene) fn register_builtin_reflection(world: &mut World) {
    *world.type_registry_mut_for_reflection() = builtin_type_registry();
}
