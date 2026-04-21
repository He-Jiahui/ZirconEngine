#[test]
fn scene_module_registration_is_absorbed_into_runtime_scene_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_entry = runtime_root.join("src/scene.rs");
    let scene_mod = runtime_root.join("src/scene/mod.rs");
    let legacy_scene_lib = runtime_root.join("../zircon_scene/src/lib.rs");

    let scene_entry_source = std::fs::read_to_string(&scene_entry).unwrap_or_default();
    let scene_mod_source = std::fs::read_to_string(&scene_mod).unwrap_or_default();

    assert!(
        scene_mod.exists(),
        "expected zircon_runtime/src/scene/mod.rs to own the absorbed scene module registration surface"
    );
    assert!(
        scene_mod_source.contains("SceneModule"),
        "zircon_runtime::scene should define SceneModule after scene module absorption"
    );
    assert!(
        !scene_entry_source.contains("pub use zircon_scene::*"),
        "zircon_runtime/src/scene.rs should stop re-exporting the entire zircon_scene crate after absorption"
    );
    assert!(
        !legacy_scene_lib.exists(),
        "standalone zircon_scene crate should be removed after merging into zircon_runtime::scene"
    );
}
