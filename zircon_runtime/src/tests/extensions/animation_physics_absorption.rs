#[test]
fn physics_domain_is_absorbed_into_runtime_package() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let physics_root = runtime_root.join("src/physics");
    let physics_mod = std::fs::read_to_string(physics_root.join("mod.rs")).unwrap_or_default();
    let physics_module =
        std::fs::read_to_string(physics_root.join("module.rs")).unwrap_or_default();
    let physics_runtime =
        std::fs::read_to_string(physics_root.join("runtime/mod.rs")).unwrap_or_default();

    assert!(
        physics_root.join("mod.rs").exists(),
        "physics should live under zircon_runtime/src/physics after runtime absorption"
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
        "runtime physics should own driver wiring, manager handle wiring, and the framework manager implementation"
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
}

#[test]
fn animation_domain_is_absorbed_into_runtime_package() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
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
        "animation should live under zircon_runtime/src/animation after runtime absorption"
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
        "runtime animation should own driver wiring, manager implementation, and sequence application"
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
}
