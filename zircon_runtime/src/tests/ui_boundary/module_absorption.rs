#[test]
fn ui_module_registration_is_absorbed_into_runtime_ui_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_entry = runtime_root.join("src/ui.rs");
    let ui_mod = runtime_root.join("src/ui/mod.rs");
    let ui_module = runtime_root.join("src/ui/module.rs");
    let legacy_ui_lib = runtime_root.join("../zircon_ui/src/lib.rs");

    let ui_entry_source = std::fs::read_to_string(&ui_entry).unwrap_or_default();
    let ui_mod_source = std::fs::read_to_string(&ui_mod).unwrap_or_default();
    let ui_module_source = std::fs::read_to_string(&ui_module).unwrap_or_default();

    assert!(
        ui_mod.exists(),
        "expected zircon_runtime/src/ui/mod.rs to own the absorbed UI module registration surface"
    );
    assert!(
        ui_mod_source.contains("UiModule"),
        "zircon_runtime::ui should define UiModule after UI module absorption"
    );
    assert!(
        !ui_entry_source.contains("pub use zircon_ui::*"),
        "zircon_runtime/src/ui.rs should stop re-exporting the entire zircon_ui crate after absorption"
    );
    assert!(
        !ui_mod_source.contains("pub use zircon_ui::*"),
        "zircon_runtime/src/ui/mod.rs should stop wildcard-re-exporting zircon_ui"
    );
    assert!(
        !ui_module_source.contains("stub_module_descriptor"),
        "zircon_runtime::ui module descriptor should stop using stub_module_descriptor"
    );
    assert!(
        !legacy_ui_lib.exists(),
        "standalone zircon_ui crate should be removed after merging into zircon_runtime::ui"
    );
}
