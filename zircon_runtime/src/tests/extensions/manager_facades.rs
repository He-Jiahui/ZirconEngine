#[test]
fn externalized_runtime_plugins_keep_manager_handles_under_core_manager_facades() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should have a workspace parent");
    let plugin_root = repo_root.join("zircon_plugins");

    let physics_mod_source =
        std::fs::read_to_string(plugin_root.join("physics/runtime/src/module.rs"))
            .unwrap_or_default();
    let physics_service_source =
        std::fs::read_to_string(plugin_root.join("physics/runtime/src/service_types.rs"))
            .unwrap_or_default();
    let animation_mod_source =
        std::fs::read_to_string(plugin_root.join("animation/runtime/src/module.rs"))
            .unwrap_or_default();
    let animation_service_source =
        std::fs::read_to_string(plugin_root.join("animation/runtime/src/service_types.rs"))
            .unwrap_or_default();
    let net_mod_source =
        std::fs::read_to_string(plugin_root.join("net/runtime/src/module.rs")).unwrap_or_default();
    let net_service_source =
        std::fs::read_to_string(plugin_root.join("net/runtime/src/service_types.rs"))
            .unwrap_or_default();
    let sound_mod_source = std::fs::read_to_string(plugin_root.join("sound/runtime/src/module.rs"))
        .unwrap_or_default();
    let sound_service_source =
        std::fs::read_to_string(plugin_root.join("sound/runtime/src/service_types.rs"))
            .unwrap_or_default();
    let manager_mod_source =
        std::fs::read_to_string(runtime_root.join("src/core/manager/mod.rs")).unwrap_or_default();
    let manager_resolver_source =
        std::fs::read_to_string(runtime_root.join("src/core/manager/resolver.rs"))
            .unwrap_or_default();
    let manager_service_names_source =
        std::fs::read_to_string(runtime_root.join("src/core/manager/service_names.rs"))
            .unwrap_or_default();

    for required in [
        "DefaultPhysicsManager",
        "impl zircon_runtime::core::framework::physics::PhysicsManager for DefaultPhysicsManager",
        "PHYSICS_MANAGER_NAME",
    ] {
        assert!(
            physics_mod_source.contains(required) || physics_service_source.contains(required),
            "physics plugin should keep its framework-backed manager service wiring `{required}`"
        );
    }

    for required in [
        "DefaultAnimationManager",
        "impl zircon_runtime::core::framework::animation::AnimationManager for DefaultAnimationManager",
        "ANIMATION_MANAGER_NAME",
    ] {
        assert!(
            animation_mod_source.contains(required) || animation_service_source.contains(required),
            "animation plugin should keep its framework-backed manager service wiring `{required}`"
        );
    }

    for required in [
        "DefaultNetManager",
        "impl zircon_runtime::core::framework::net::NetManager for DefaultNetManager",
        "NET_MANAGER_NAME",
    ] {
        assert!(
            net_mod_source.contains(required) || net_service_source.contains(required),
            "net plugin should keep its framework-backed manager service wiring `{required}`"
        );
    }

    for required in [
        "DefaultSoundManager",
        "impl zircon_runtime::core::framework::sound::SoundManager for DefaultSoundManager",
        "SOUND_MANAGER_NAME",
    ] {
        assert!(
            sound_mod_source.contains(required) || sound_service_source.contains(required),
            "sound plugin should keep its framework-backed manager service wiring `{required}`"
        );
    }

    for required in [
        "PhysicsManagerHandle",
        "AnimationManagerHandle",
        "NetManagerHandle",
        "SoundManagerHandle",
        "resolve_physics_manager",
        "resolve_animation_manager",
        "resolve_net_manager",
        "resolve_sound_manager",
    ] {
        assert!(
            manager_mod_source.contains(required) || manager_resolver_source.contains(required),
            "core manager surface should own public plugin manager handle wiring `{required}`"
        );
    }

    for required in [
        "PHYSICS_MANAGER_NAME",
        "ANIMATION_MANAGER_NAME",
        "NET_MANAGER_NAME",
        "SOUND_MANAGER_NAME",
    ] {
        assert!(
            manager_mod_source.contains(required)
                || manager_resolver_source.contains(required)
                || manager_service_names_source.contains(required),
            "core manager surface should own public plugin manager service names `{required}`"
        );
    }
}
