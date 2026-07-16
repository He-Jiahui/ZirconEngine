use crate::builtin::builtin_runtime_modules;

#[test]
fn builtin_runtime_modules_include_target_client_core_and_required_plugins() {
    let descriptors = builtin_runtime_modules()
        .into_iter()
        .map(|module| module.descriptor().name)
        .collect::<Vec<_>>();

    for expected in [
        crate::foundation::FOUNDATION_MODULE_NAME,
        crate::platform::PLATFORM_MODULE_NAME,
        crate::input::INPUT_MODULE_NAME,
        crate::asset::ASSET_MODULE_NAME,
        crate::scene::SCENE_MODULE_NAME,
        crate::core::framework::render::GRAPHICS_MODULE_NAME,
        crate::script::SCRIPT_MODULE_NAME,
    ] {
        assert!(
            descriptors.iter().any(|name| name == expected),
            "missing runtime module {expected}"
        );
    }

    #[cfg(feature = "ui")]
    assert!(
        descriptors
            .iter()
            .any(|name| name == crate::ui::UI_MODULE_NAME),
        "missing runtime module {}",
        crate::ui::UI_MODULE_NAME
    );
}

#[test]
fn builtin_runtime_modules_keep_client_plugins_after_core_spine() {
    let descriptors = builtin_runtime_modules()
        .into_iter()
        .map(|module| module.descriptor().name)
        .collect::<Vec<_>>();

    let script_index = descriptors
        .iter()
        .position(|name| *name == crate::script::SCRIPT_MODULE_NAME)
        .expect("script module should exist in runtime builtins");
    let graphics_index = descriptors
        .iter()
        .position(|name| *name == crate::core::framework::render::GRAPHICS_MODULE_NAME)
        .expect("graphics module should exist in runtime builtins");
    let scene_index = descriptors
        .iter()
        .position(|name| *name == crate::scene::SCENE_MODULE_NAME)
        .expect("scene module should exist in runtime builtins");

    assert!(
        scene_index < script_index,
        "scene should remain part of the core spine before script"
    );
    assert_eq!(
        graphics_index,
        scene_index + 1,
        "graphics base should remain in the minimal runtime core before script"
    );

    #[cfg(feature = "ui")]
    {
        let ui_index = descriptors
            .iter()
            .position(|name| *name == crate::ui::UI_MODULE_NAME)
            .expect("ui module should exist in runtime builtins");

        assert!(
            graphics_index < ui_index,
            "ui should initialize after the graphics module it depends on"
        );
        assert!(
            ui_index < script_index,
            "scene-level ui should initialize before the post-level script module"
        );
    }

    #[cfg(not(feature = "ui"))]
    assert_eq!(
        script_index,
        descriptors.len() - 1,
        "core-min runtime spine should stop at script when ui is disabled"
    );
}
