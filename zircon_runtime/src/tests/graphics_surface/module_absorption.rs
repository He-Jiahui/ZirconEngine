#[test]
fn graphics_module_host_is_absorbed_into_runtime_graphics_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let graphics_entry = runtime_root.join("src/graphics.rs");
    let graphics_mod = runtime_root.join("src/graphics/mod.rs");
    let graphics_entry_source = std::fs::read_to_string(&graphics_entry).unwrap_or_default();
    let graphics_mod_source = std::fs::read_to_string(&graphics_mod).unwrap_or_default();

    assert!(
        graphics_mod.exists(),
        "expected zircon_runtime/src/graphics/mod.rs to own the absorbed graphics module-host surface"
    );
    assert!(
        graphics_mod_source.contains("GraphicsModule"),
        "zircon_runtime::graphics should define GraphicsModule after host absorption"
    );
    assert!(
        !graphics_entry_source.contains("pub use zircon_graphics::*"),
        "zircon_runtime/src/graphics.rs should stop re-exporting the entire zircon_graphics crate"
    );
    for forbidden in [
        "create_render_service",
        "create_runtime_preview_renderer",
        "create_shared_texture_render_service",
        "WgpuDriver",
        "WgpuRenderingManager",
    ] {
        assert!(
            !graphics_mod_source.contains(forbidden),
            "zircon_runtime::graphics should stop publicly exporting `{forbidden}` after graphics boundary cleanup"
        );
    }
}
