#[test]
fn graphics_runtime_surface_no_longer_depends_on_legacy_scene_crate() {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));

    assert!(
        !manifest.contains("zircon_scene"),
        "zircon_runtime graphics should stay fully absorbed and stop depending on the legacy zircon_scene crate"
    );
}

#[test]
fn graphics_root_no_longer_exports_legacy_preview_or_render_service_surface() {
    let graphics_mod_source =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/graphics/mod.rs"));

    for forbidden in [
        "pub use backend::RuntimePreviewRenderer;",
        "pub use service::{RenderService, SharedTextureRenderService};",
        "mod service;",
    ] {
        assert!(
            !graphics_mod_source.contains(forbidden),
            "zircon_runtime graphics root should stop exporting legacy surface `{forbidden}`"
        );
    }
}

#[test]
fn graphics_root_no_longer_exports_runtime_ui_host_surface() {
    let graphics_mod_source =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/graphics/mod.rs"));
    let runtime_mod_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/graphics/runtime/mod.rs"
    ));

    for forbidden in [
        "RuntimeUiFixture",
        "RuntimeUiManager",
        "RuntimeUiManagerError",
        "mod ui;",
        "pub use ui::{RuntimeUiFixture, RuntimeUiManager, RuntimeUiManagerError};",
    ] {
        assert!(
            !graphics_mod_source.contains(forbidden) && !runtime_mod_source.contains(forbidden),
            "zircon_runtime graphics should stop owning runtime UI host surface `{forbidden}`"
        );
    }
}
