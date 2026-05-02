use crate::{
    builtin_runtime_modules, runtime_modules_for_target, ProjectPluginManifest,
    ProjectPluginSelection, RuntimePluginId, RuntimeTargetMode,
};

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
        crate::physics::PHYSICS_MODULE_NAME,
        crate::animation::ANIMATION_MODULE_NAME,
        crate::graphics::GRAPHICS_MODULE_NAME,
        crate::script::SCRIPT_MODULE_NAME,
    ] {
        assert!(
            descriptors.iter().any(|name| name == expected),
            "missing runtime module {expected}"
        );
    }

    #[cfg(feature = "plugin-ui")]
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
        .position(|name| *name == crate::graphics::GRAPHICS_MODULE_NAME)
        .expect("graphics module should exist in runtime builtins");
    let scene_index = descriptors
        .iter()
        .position(|name| *name == crate::scene::SCENE_MODULE_NAME)
        .expect("scene module should exist in runtime builtins");
    let physics_index = descriptors
        .iter()
        .position(|name| *name == crate::physics::PHYSICS_MODULE_NAME)
        .expect("physics module should exist in runtime builtins");
    let animation_index = descriptors
        .iter()
        .position(|name| *name == crate::animation::ANIMATION_MODULE_NAME)
        .expect("animation module should exist in runtime builtins");

    assert!(
        scene_index < script_index,
        "scene should remain part of the core spine before script"
    );
    assert_eq!(
        graphics_index,
        animation_index + 1,
        "graphics base should remain in the minimal runtime core before script"
    );
    assert_eq!(physics_index, scene_index + 1);
    assert_eq!(animation_index, physics_index + 1);

    #[cfg(feature = "plugin-ui")]
    {
        let ui_index = descriptors
            .iter()
            .position(|name| *name == crate::ui::UI_MODULE_NAME)
            .expect("ui module should exist in runtime builtins");

        assert_eq!(
            ui_index,
            script_index + 1,
            "ui plugin should follow the target-client core spine"
        );
    }

    #[cfg(not(feature = "plugin-ui"))]
    assert_eq!(
        script_index,
        descriptors.len() - 1,
        "core-min runtime spine should stop at script when plugin-ui is disabled"
    );
}

#[test]
fn required_unavailable_runtime_plugin_is_reported_as_fatal_missing() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            true,
        )],
    };

    let report = runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, Some(&manifest));

    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::VirtualGeometry));
    assert!(report
        .required_missing_summary()
        .contains("VirtualGeometry"));
}

#[test]
fn optional_unavailable_runtime_plugin_stays_warning_only() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            false,
        )],
    };

    let report = runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, Some(&manifest));

    assert!(report.required_missing().is_empty());
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("zircon_plugins/virtual_geometry")));
}

#[test]
fn physics_animation_manifest_entries_resolve_to_builtin_runtime_domains() {
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Physics, true, true),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, true),
        ],
    };

    let report = runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, Some(&manifest));

    assert!(report.required_missing().is_empty());
    assert!(report.errors.is_empty());
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("zircon_runtime::physics")));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("zircon_runtime::animation")));
}
