#[test]
fn physics_domain_keeps_framework_contract_and_plugin_owns_runtime_behavior() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should have a workspace parent");
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let runtime_lib = std::fs::read_to_string(runtime_root.join("src/lib.rs")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let plugin_workspace_manifest =
        std::fs::read_to_string(repo_root.join("zircon_plugins/Cargo.toml")).unwrap_or_default();
    let physics_plugin_root = repo_root.join("zircon_plugins/physics");
    let physics_plugin_lib =
        std::fs::read_to_string(physics_plugin_root.join("runtime/src/lib.rs")).unwrap_or_default();
    let physics_plugin_module =
        std::fs::read_to_string(physics_plugin_root.join("runtime/src/module.rs"))
            .unwrap_or_default();
    let physics_plugin_manager =
        std::fs::read_to_string(physics_plugin_root.join("runtime/src/manager.rs"))
            .unwrap_or_default();
    let physics_plugin_hook =
        std::fs::read_to_string(physics_plugin_root.join("runtime/src/scene_hook.rs"))
            .unwrap_or_default();
    let framework_physics_manager =
        std::fs::read_to_string(runtime_root.join("src/core/framework/physics/manager.rs"))
            .unwrap_or_default();

    assert!(
        !runtime_root.join("src/physics").exists(),
        "zircon_runtime should not keep concrete physics runtime files after plugin cutover"
    );
    assert!(
        !runtime_lib.contains("pub mod physics"),
        "zircon_runtime crate root should not export a concrete physics module"
    );
    assert!(
        runtime_root
            .join("src/core/framework/physics/scene_step_result.rs")
            .exists(),
        "runtime framework should keep neutral physics scene-step DTOs"
    );
    assert!(
        framework_physics_manager.contains("fn tick_scene_world"),
        "runtime framework should expose the neutral PhysicsManager scene tick contract"
    );
    assert!(
        physics_plugin_module.contains("PhysicsDriver")
            && physics_plugin_module.contains("PhysicsManagerHandle")
            && physics_plugin_module.contains("module_descriptor")
            && physics_plugin_manager.contains("impl PhysicsManager for DefaultPhysicsManager")
            && physics_plugin_manager.contains("tick_scene_world")
            && physics_plugin_hook.contains("PhysicsSceneRuntimeHook")
            && physics_plugin_lib.contains("register_scene_hook(scene_hook_registration())"),
        "physics plugin should own module wiring, manager behavior, and scene hook registration"
    );
    assert!(
        !runtime_manifest.contains("zircon_physics"),
        "zircon_runtime/Cargo.toml should not depend on a legacy zircon_physics crate"
    );
    assert!(
        !workspace_manifest.contains("\"zircon_physics\""),
        "workspace Cargo.toml should not list a legacy zircon_physics crate"
    );
    assert!(
        !runtime_manifest.contains("zircon_plugin_physics_runtime"),
        "zircon_runtime must not depend on the physics plugin crate"
    );
    assert!(
        !runtime_root.join("../zircon_physics/Cargo.toml").exists(),
        "legacy zircon_physics crate should be removed instead of kept as a compatibility shell"
    );
    assert!(
        physics_plugin_root.join("plugin.toml").exists(),
        "physics should own a zircon_plugins/physics/plugin.toml package manifest"
    );
    assert!(
        physics_plugin_root.join("runtime/Cargo.toml").exists(),
        "physics should own a zircon_plugins/physics/runtime crate"
    );
    assert!(
        physics_plugin_root.join("editor/Cargo.toml").exists(),
        "physics should own a zircon_plugins/physics/editor crate"
    );
    assert!(
        plugin_workspace_manifest.contains("\"physics/runtime\"")
            && plugin_workspace_manifest.contains("\"physics/editor\""),
        "zircon_plugins/Cargo.toml should list physics runtime/editor members"
    );
}

#[test]
fn animation_domain_keeps_framework_contract_and_plugin_owns_runtime_behavior() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should have a workspace parent");
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let runtime_lib = std::fs::read_to_string(runtime_root.join("src/lib.rs")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let plugin_workspace_manifest =
        std::fs::read_to_string(repo_root.join("zircon_plugins/Cargo.toml")).unwrap_or_default();
    let animation_plugin_root = repo_root.join("zircon_plugins/animation");
    let animation_plugin_lib =
        std::fs::read_to_string(animation_plugin_root.join("runtime/src/lib.rs"))
            .unwrap_or_default();
    let animation_plugin_module =
        std::fs::read_to_string(animation_plugin_root.join("runtime/src/module.rs"))
            .unwrap_or_default();
    let animation_plugin_manager =
        std::fs::read_to_string(animation_plugin_root.join("runtime/src/manager.rs"))
            .unwrap_or_default();
    let animation_plugin_sequence =
        std::fs::read_to_string(animation_plugin_root.join("runtime/src/sequence.rs"))
            .unwrap_or_default();
    let animation_plugin_hook =
        std::fs::read_to_string(animation_plugin_root.join("runtime/src/scene_hook.rs"))
            .unwrap_or_default();
    let framework_animation_manager =
        std::fs::read_to_string(runtime_root.join("src/core/framework/animation/manager.rs"))
            .unwrap_or_default();

    assert!(
        !runtime_root.join("src/animation").exists(),
        "zircon_runtime should not keep concrete animation runtime files after plugin cutover"
    );
    assert!(
        !runtime_lib.contains("pub mod animation"),
        "zircon_runtime crate root should not export a concrete animation module"
    );
    assert!(
        runtime_root
            .join("src/core/framework/animation/sequence_apply_report.rs")
            .exists(),
        "runtime framework should keep neutral animation sequence apply DTOs"
    );
    assert!(
        framework_animation_manager.contains("fn apply_sequence_to_world"),
        "runtime framework should expose the neutral AnimationManager sequence apply contract"
    );
    assert!(
        animation_plugin_module.contains("AnimationDriver")
            && animation_plugin_module.contains("AnimationManagerHandle")
            && animation_plugin_module.contains("module_descriptor")
            && animation_plugin_manager.contains("impl AnimationManager for DefaultAnimationManager")
            && animation_plugin_manager.contains("apply_sequence_to_world")
            && animation_plugin_sequence.contains("apply_sequence_to_world")
            && animation_plugin_hook.contains("AnimationSceneRuntimeHook")
            && animation_plugin_lib.contains("register_scene_hook(scene_hook_registration())"),
        "animation plugin should own module wiring, manager behavior, sequence application, and scene hook registration"
    );
    assert!(
        !runtime_manifest.contains("zircon_animation"),
        "zircon_runtime/Cargo.toml should not depend on a legacy zircon_animation crate"
    );
    assert!(
        !workspace_manifest.contains("\"zircon_animation\""),
        "workspace Cargo.toml should not list a legacy zircon_animation crate"
    );
    assert!(
        !runtime_manifest.contains("zircon_plugin_animation_runtime"),
        "zircon_runtime must not depend on the animation plugin crate"
    );
    assert!(
        !runtime_root.join("../zircon_animation/Cargo.toml").exists(),
        "legacy zircon_animation crate should be removed instead of kept as a compatibility shell"
    );
    assert!(
        animation_plugin_root.join("plugin.toml").exists(),
        "animation should own a zircon_plugins/animation/plugin.toml package manifest"
    );
    assert!(
        animation_plugin_root.join("runtime/Cargo.toml").exists(),
        "animation should own a zircon_plugins/animation/runtime crate"
    );
    assert!(
        animation_plugin_root.join("editor/Cargo.toml").exists(),
        "animation should own a zircon_plugins/animation/editor crate"
    );
    assert!(
        plugin_workspace_manifest.contains("\"animation/runtime\"")
            && plugin_workspace_manifest.contains("\"animation/editor\""),
        "zircon_plugins/Cargo.toml should list animation runtime/editor members"
    );
}
