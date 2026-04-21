#[test]
fn physics_extension_is_physically_absorbed_into_runtime() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let physics_mod =
        std::fs::read_to_string(runtime_root.join("src/physics/mod.rs")).unwrap_or_default();
    let physics_interface =
        std::fs::read_to_string(runtime_root.join("src/physics/physics_interface.rs"))
            .unwrap_or_default();

    assert!(
        runtime_root.join("src/physics/mod.rs").exists(),
        "physics should live as a first-class subsystem at zircon_runtime/src/physics/mod.rs"
    );
    assert!(
        runtime_root
            .join("src/physics/physics_interface.rs")
            .exists(),
        "physics absorption should add a runtime-owned physics_interface surface"
    );
    assert!(
        !runtime_root.join("src/physics.rs").exists(),
        "zircon_runtime should stop keeping physics absorption in a flat root file"
    );
    assert!(
        physics_mod.contains("PhysicsDriver")
            && physics_mod.contains("PhysicsManager")
            && physics_mod.contains("PhysicsInterface"),
        "runtime physics subtree should own the driver, manager, and runtime-facing interface"
    );
    assert!(
        physics_interface.contains("pub trait PhysicsInterface"),
        "physics_interface should define a runtime-owned PhysicsInterface trait"
    );
    assert!(
        !runtime_manifest.contains("zircon_physics"),
        "zircon_runtime/Cargo.toml should stop depending on zircon_physics once physics is absorbed"
    );
    assert!(
        !workspace_manifest.contains("\"zircon_physics\""),
        "workspace Cargo.toml should stop listing zircon_physics after runtime absorption"
    );
    assert!(
        !runtime_root.join("../zircon_physics/Cargo.toml").exists(),
        "legacy zircon_physics crate should be removed instead of kept as a compatibility shell"
    );
}

#[test]
fn animation_extension_is_physically_absorbed_into_runtime() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let animation_mod =
        std::fs::read_to_string(runtime_root.join("src/animation/mod.rs")).unwrap_or_default();
    let animation_interface =
        std::fs::read_to_string(runtime_root.join("src/animation/animation_interface.rs"))
            .unwrap_or_default();

    assert!(
        runtime_root.join("src/animation/mod.rs").exists(),
        "animation should live as a first-class subsystem at zircon_runtime/src/animation/mod.rs"
    );
    assert!(
        runtime_root
            .join("src/animation/animation_interface.rs")
            .exists(),
        "animation absorption should add a runtime-owned animation_interface surface"
    );
    assert!(
        !runtime_root.join("src/animation.rs").exists(),
        "zircon_runtime should stop keeping animation absorption in a flat root file"
    );
    assert!(
        animation_mod.contains("AnimationDriver")
            && animation_mod.contains("AnimationManager")
            && animation_mod.contains("AnimationInterface"),
        "runtime animation subtree should own the driver, manager, and runtime-facing interface"
    );
    assert!(
        animation_interface.contains("pub trait AnimationInterface"),
        "animation_interface should define a runtime-owned AnimationInterface trait"
    );
    assert!(
        !runtime_manifest.contains("zircon_animation"),
        "zircon_runtime/Cargo.toml should stop depending on zircon_animation once animation is absorbed"
    );
    assert!(
        !workspace_manifest.contains("\"zircon_animation\""),
        "workspace Cargo.toml should stop listing zircon_animation after runtime absorption"
    );
    assert!(
        !runtime_root.join("../zircon_animation/Cargo.toml").exists(),
        "legacy zircon_animation crate should be removed instead of kept as a compatibility shell"
    );
}
