#[test]
fn script_subsystem_is_physically_absorbed_into_runtime_crate() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script_entry = runtime_root.join("src/script.rs");
    let script_mod = runtime_root.join("src/script/mod.rs");
    let workspace_manifest = runtime_root.join("../Cargo.toml");
    let legacy_manifest = runtime_root.join("../zircon_script/Cargo.toml");

    let script_entry_source = std::fs::read_to_string(&script_entry).unwrap_or_default();
    let workspace_manifest_source =
        std::fs::read_to_string(&workspace_manifest).unwrap_or_default();

    assert!(
        script_mod.exists(),
        "expected zircon_runtime/src/script/mod.rs to own the absorbed script subsystem"
    );
    assert!(
        !script_entry_source.contains("pub use zircon_script::*"),
        "zircon_runtime/src/script.rs should stop re-exporting zircon_script after absorption"
    );
    assert!(
        !workspace_manifest_source.contains("\"zircon_script\""),
        "workspace Cargo.toml should stop listing zircon_script after absorption"
    );
    assert!(
        !legacy_manifest.exists(),
        "standalone zircon_script crate should be deleted after absorption"
    );
}
