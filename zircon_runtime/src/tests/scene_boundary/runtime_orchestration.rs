#[test]
fn scene_runtime_orchestration_and_level_system_are_absorbed_into_runtime_scene_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_mod = runtime_root.join("src/scene/mod.rs");
    let level_system = runtime_root.join("src/scene/level_system.rs");
    let module_dir = runtime_root.join("src/scene/module/mod.rs");
    let legacy_scene_lib = runtime_root.join("../zircon_scene/src/lib.rs");

    let scene_mod_source = std::fs::read_to_string(&scene_mod).unwrap_or_default();

    assert!(
        level_system.exists(),
        "runtime scene should own LevelSystem"
    );
    assert!(
        module_dir.exists(),
        "runtime scene should own folder-backed module orchestration"
    );
    assert!(scene_mod_source.contains("LevelSystem"));
    assert!(scene_mod_source.contains("DefaultLevelManager"));
    assert!(scene_mod_source.contains("WorldDriver"));
    assert!(
        !scene_mod_source.contains("pub use zircon_scene::*"),
        "runtime scene root should stop wildcard-re-exporting zircon_scene"
    );
    assert!(
        !legacy_scene_lib.exists(),
        "standalone zircon_scene crate should be removed after merging into zircon_runtime::scene"
    );
}
