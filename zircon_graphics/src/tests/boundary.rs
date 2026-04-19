#[test]
fn graphics_production_surface_uses_scene_crate_only_in_dev_dependencies() {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|section| section.split("\n[").next())
        .expect("Cargo.toml should contain a [dependencies] section");
    let dev_dependencies = manifest
        .split("[dev-dependencies]")
        .nth(1)
        .and_then(|section| section.split("\n[").next())
        .expect("Cargo.toml should contain a [dev-dependencies] section");

    assert!(
        !dependencies.contains("zircon_scene"),
        "zircon_graphics production dependencies should no longer pull zircon_scene directly"
    );
    assert!(
        dev_dependencies.contains("zircon_scene = { path = \"../zircon_scene\" }"),
        "zircon_scene should remain available only for graphics tests that still construct runtime-world fixtures"
    );
}

#[test]
fn graphics_root_no_longer_exports_legacy_preview_or_render_service_surface() {
    let lib_source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));

    for forbidden in [
        "pub use backend::RuntimePreviewRenderer;",
        "pub use service::{RenderService, SharedTextureRenderService};",
        "mod service;",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_graphics root should stop exporting legacy surface `{forbidden}`"
        );
    }
}

#[test]
fn graphics_root_no_longer_exports_runtime_ui_host_surface() {
    let lib_source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
    let runtime_mod_source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime/mod.rs"));

    for forbidden in [
        "RuntimeUiFixture",
        "RuntimeUiManager",
        "RuntimeUiManagerError",
        "mod ui;",
        "pub use ui::{RuntimeUiFixture, RuntimeUiManager, RuntimeUiManagerError};",
    ] {
        assert!(
            !lib_source.contains(forbidden) && !runtime_mod_source.contains(forbidden),
            "zircon_graphics should stop owning runtime UI host surface `{forbidden}`"
        );
    }
}
