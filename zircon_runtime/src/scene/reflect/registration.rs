use crate::scene::World;

pub(in crate::scene) fn register_builtin_reflection(world: &mut World) {
    let registry = world.type_registry_mut_for_reflection();
    registry.clear();
    super::builtin_reflection::register(registry)
        .expect("builtin derived reflection registrations must be unique and valid");
}
