#[test]
fn optional_extension_module_registration_is_absorbed_into_runtime_extensions_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let extensions_mod = runtime_root.join("src/extensions/mod.rs");
    let builtin_source =
        std::fs::read_to_string(runtime_root.join("src/builtin/mod.rs")).unwrap_or_default();

    assert!(
        extensions_mod.exists(),
        "expected zircon_runtime/src/extensions/mod.rs to own optional extension module registration"
    );

    for (legacy_lib, module_name) in [
        ("../zircon_physics/src/lib.rs", "PhysicsModule"),
        ("../zircon_animation/src/lib.rs", "AnimationModule"),
    ] {
        let legacy_source =
            std::fs::read_to_string(runtime_root.join(legacy_lib)).unwrap_or_default();

        assert!(
            !legacy_source.contains(&format!("pub struct {module_name}")),
            "legacy extension crate {legacy_lib} should stop owning {module_name}"
        );
        assert!(
            !legacy_source.contains("pub fn module_descriptor()"),
            "legacy extension crate {legacy_lib} should stop owning module_descriptor()"
        );
        assert!(
            !builtin_source.contains("Arc::new(zircon_"),
            "builtin runtime module list should stop constructing optional extension modules from legacy crate roots"
        );
    }
}
