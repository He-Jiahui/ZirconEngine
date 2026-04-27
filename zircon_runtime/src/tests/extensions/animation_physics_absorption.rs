#[test]
fn physics_extension_is_physically_externalized_to_plugin_package() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should have a workspace parent");
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let physics_root = repo_root.join("zircon_plugins/physics/runtime/src");
    let physics_mod = std::fs::read_to_string(physics_root.join("module.rs")).unwrap_or_default();
    let physics_interface =
        std::fs::read_to_string(physics_root.join("physics_interface.rs")).unwrap_or_default();

    assert!(
        !runtime_root.join("src/physics").exists(),
        "physics should not live under zircon_runtime after plugin cutover"
    );
    assert!(
        physics_root.join("lib.rs").exists(),
        "physics plugin package should own runtime/src/lib.rs"
    );
    assert!(
        !runtime_root.join("src/physics.rs").exists(),
        "zircon_runtime should stop keeping physics in a flat root file"
    );
    assert!(
        physics_mod.contains("PhysicsDriver")
            && physics_mod.contains("PhysicsManager")
            && physics_interface.contains("PhysicsInterface"),
        "physics plugin should own the driver, manager, and runtime-facing interface"
    );
    assert!(
        physics_interface.contains("pub trait PhysicsInterface"),
        "physics_interface should define a plugin-owned PhysicsInterface trait"
    );
    assert!(
        !runtime_manifest.contains("zircon_physics"),
        "zircon_runtime/Cargo.toml should stop depending on zircon_physics once physics is externalized"
    );
    assert!(
        !workspace_manifest.contains("\"zircon_physics\""),
        "workspace Cargo.toml should stop listing zircon_physics after plugin cutover"
    );
    assert!(
        !runtime_root.join("../zircon_physics/Cargo.toml").exists(),
        "legacy zircon_physics crate should be removed instead of kept as a compatibility shell"
    );
}

#[test]
fn animation_extension_is_physically_externalized_to_plugin_package() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should have a workspace parent");
    let runtime_manifest =
        std::fs::read_to_string(runtime_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_manifest =
        std::fs::read_to_string(runtime_root.join("../Cargo.toml")).unwrap_or_default();
    let animation_root = repo_root.join("zircon_plugins/animation/runtime/src");
    let animation_mod =
        std::fs::read_to_string(animation_root.join("module.rs")).unwrap_or_default();
    let animation_interface =
        std::fs::read_to_string(animation_root.join("animation_interface.rs")).unwrap_or_default();

    assert!(
        !runtime_root.join("src/animation").exists(),
        "animation should not live under zircon_runtime after plugin cutover"
    );
    assert!(
        animation_root.join("lib.rs").exists(),
        "animation plugin package should own runtime/src/lib.rs"
    );
    assert!(
        !runtime_root.join("src/animation.rs").exists(),
        "zircon_runtime should stop keeping animation in a flat root file"
    );
    assert!(
        animation_mod.contains("AnimationDriver")
            && animation_mod.contains("AnimationManager")
            && animation_interface.contains("AnimationInterface"),
        "animation plugin should own the driver, manager, and runtime-facing interface"
    );
    assert!(
        animation_interface.contains("pub trait AnimationInterface"),
        "animation_interface should define a plugin-owned AnimationInterface trait"
    );
    assert!(
        !runtime_manifest.contains("zircon_animation"),
        "zircon_runtime/Cargo.toml should stop depending on zircon_animation once animation is externalized"
    );
    assert!(
        !workspace_manifest.contains("\"zircon_animation\""),
        "workspace Cargo.toml should stop listing zircon_animation after plugin cutover"
    );
    assert!(
        !runtime_root.join("../zircon_animation/Cargo.toml").exists(),
        "legacy zircon_animation crate should be removed instead of kept as a compatibility shell"
    );
}
