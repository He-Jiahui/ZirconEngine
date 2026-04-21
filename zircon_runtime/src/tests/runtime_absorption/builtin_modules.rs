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
        "PhysicsModule",
        "SoundModule",
        "TextureModule",
        "NetModule",
        "NavigationModule",
        "ParticlesModule",
        "AnimationModule",
    ] {
        assert!(
            descriptors.iter().any(|name| name == expected),
            "missing runtime module {expected}"
        );
    }
}

#[test]
fn builtin_runtime_modules_keep_graphics_in_runtime_owned_ordering() {
    let descriptors = builtin_runtime_modules()
        .into_iter()
        .map(|module| module.descriptor().name)
        .collect::<Vec<_>>();

    let asset_index = descriptors
        .iter()
        .position(|name| *name == crate::asset::ASSET_MODULE_NAME)
        .expect("asset module should exist in runtime builtins");
    let graphics_index = descriptors
        .iter()
        .position(|name| *name == crate::graphics::GRAPHICS_MODULE_NAME)
        .expect("graphics module should exist in runtime builtins");
    let scene_index = descriptors
        .iter()
        .position(|name| *name == crate::scene::SCENE_MODULE_NAME)
        .expect("scene module should exist in runtime builtins");

    assert_eq!(
        graphics_index,
        asset_index + 1,
        "graphics module ordering should stay runtime-owned immediately after asset"
    );
    assert_eq!(
        scene_index,
        graphics_index + 1,
        "scene module ordering should follow graphics in the runtime-owned builtin chain"
    );
}
