use crate::scene::World;

pub(in crate::scene) fn register_builtin_reflection(world: &mut World) {
    let registry = world.type_registry_mut_for_reflection();
    registry.clear();
    super::fixed::register_fixed_components(registry)
        .expect("builtin fixed reflection registrations must be unique and valid");
}
