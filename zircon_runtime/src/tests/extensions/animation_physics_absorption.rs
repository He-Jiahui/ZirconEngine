#[test]
fn physics_domain_keeps_runtime_contract_and_has_plugin_package() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should have a workspace parent");
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let plugin_workspace_manifest =
        std::fs::read_to_string(repo_root.join("zircon_plugins/Cargo.toml")).unwrap_or_default();
    let physics_plugin_root = repo_root.join("zircon_plugins/physics");
    let physics_root = runtime_root.join("src/physics");
    let physics_mod = std::fs::read_to_string(physics_root.join("mod.rs")).unwrap_or_default();
    let physics_module =
        std::fs::read_to_string(physics_root.join("module.rs")).unwrap_or_default();
    let physics_runtime =
        std::fs::read_to_string(physics_root.join("runtime/mod.rs")).unwrap_or_default();

    assert!(
        physics_root.join("mod.rs").exists(),
        "physics shared runtime contract should remain under zircon_runtime/src/physics"
    );
    assert!(
        !runtime_root.join("src/physics.rs").exists(),
        "zircon_runtime should keep physics folder-backed, not in a flat root file"
    );
    assert!(
        physics_mod.contains("pub mod runtime") && physics_mod.contains("mod module"),
        "physics root should stay structural and delegate to module/runtime children"
    );
    assert!(
        physics_module.contains("PhysicsDriver")
            && physics_module.contains("PhysicsManagerHandle")
            && physics_runtime.contains("impl PhysicsManager for DefaultPhysicsManager"),
        "physics runtime contract should keep driver wiring, manager handle wiring, and the framework manager implementation available to its plugin package"
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
fn animation_domain_keeps_runtime_contract_and_has_plugin_package() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should have a workspace parent");
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let plugin_workspace_manifest =
        std::fs::read_to_string(repo_root.join("zircon_plugins/Cargo.toml")).unwrap_or_default();
    let animation_plugin_root = repo_root.join("zircon_plugins/animation");
    let animation_root = runtime_root.join("src/animation");
    let animation_mod = std::fs::read_to_string(animation_root.join("mod.rs")).unwrap_or_default();
    let animation_module =
        std::fs::read_to_string(animation_root.join("module.rs")).unwrap_or_default();
    let animation_runtime =
        std::fs::read_to_string(animation_root.join("runtime/mod.rs")).unwrap_or_default();
    let animation_sequence =
        std::fs::read_to_string(animation_root.join("sequence/mod.rs")).unwrap_or_default();

    assert!(
        animation_root.join("mod.rs").exists(),
        "animation shared runtime contract should remain under zircon_runtime/src/animation"
    );
    assert!(
        !runtime_root.join("src/animation.rs").exists(),
        "zircon_runtime should keep animation folder-backed, not in a flat root file"
    );
    assert!(
        animation_mod.contains("pub mod runtime") && animation_mod.contains("pub mod sequence"),
        "animation root should stay structural and delegate to runtime/sequence children"
    );
    assert!(
        animation_module.contains("AnimationDriver")
            && animation_module.contains("AnimationManagerHandle")
            && animation_runtime.contains("impl AnimationManager for DefaultAnimationManager")
            && animation_sequence.contains("apply_sequence_to_world"),
        "animation runtime contract should keep driver wiring, manager implementation, and sequence application available to its plugin package"
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
