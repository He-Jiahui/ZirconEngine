use super::*;

#[test]
fn fixed_component_registrations_exist_in_empty_world() {
    let world = World::empty();
    let expected = [
        "zircon_runtime::scene::components::ActiveSelf",
        "zircon_runtime::scene::components::AmbientLight",
        "zircon_runtime::scene::components::LocalTransform",
        "zircon_runtime::scene::components::Name",
        "zircon_runtime::scene::components::RectLight",
        "zircon_runtime::scene::components::RenderLayerMask",
        "zircon_runtime::scene::components::RigidBodyComponent",
    ];

    for type_path in expected {
        let registration = world
            .reflect_schema(type_path)
            .expect("fixed component schema should be registered");
        assert!(registration.is_component);
        assert!(
            world
                .type_registry()
                .runtime_registration(type_path)
                .expect("fixed runtime registration should exist")
                .component
                .is_some()
        );
    }
}
