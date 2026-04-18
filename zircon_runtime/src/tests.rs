use crate::builtin_runtime_modules;

#[test]
fn builtin_runtime_modules_include_absorbed_high_level_subsystems_and_extensions() {
    let descriptors = builtin_runtime_modules()
        .into_iter()
        .map(|module| module.descriptor().name)
        .collect::<Vec<_>>();

    for expected in [
        crate::foundation::FOUNDATION_MODULE_NAME,
        crate::platform::PLATFORM_MODULE_NAME,
        crate::input::INPUT_MODULE_NAME,
        crate::asset::ASSET_MODULE_NAME,
        crate::graphics::GRAPHICS_MODULE_NAME,
        crate::scene::SCENE_MODULE_NAME,
        crate::script::SCRIPT_MODULE_NAME,
        crate::ui::UI_MODULE_NAME,
        zircon_physics::PHYSICS_MODULE_NAME,
        zircon_sound::SOUND_MODULE_NAME,
        zircon_texture::TEXTURE_MODULE_NAME,
        zircon_net::NET_MODULE_NAME,
        zircon_navigation::NAVIGATION_MODULE_NAME,
        zircon_particles::PARTICLES_MODULE_NAME,
        zircon_animation::ANIMATION_MODULE_NAME,
    ] {
        assert!(
            descriptors.iter().any(|name| name == expected),
            "missing runtime module {expected}"
        );
    }
}
