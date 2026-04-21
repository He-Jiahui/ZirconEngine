#[test]
fn graphics_runtime_host_no_longer_owns_legacy_preview_or_render_service_wiring() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let host_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/runtime_builtin_graphics/host/mod.rs"),
    )
    .unwrap_or_default();
    let module_host_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/runtime_builtin_graphics/host/module_host/mod.rs"),
    )
    .unwrap_or_default();
    let create_mod_source = std::fs::read_to_string(
        runtime_root.join("src/graphics/runtime_builtin_graphics/host/module_host/create/mod.rs"),
    )
    .unwrap_or_default();
    let rendering_manager_mod_source =
        std::fs::read_to_string(runtime_root.join(
            "src/graphics/runtime_builtin_graphics/host/module_host/rendering_manager/mod.rs",
        ))
        .unwrap_or_default();

    for forbidden in [
        "create_render_service",
        "create_runtime_preview_renderer",
        "create_shared_texture_render_service",
        "WgpuDriver",
        "WgpuRenderingManager",
        "WGPU_DRIVER_NAME",
    ] {
        assert!(
            !host_mod_source.contains(forbidden),
            "runtime graphics host should stop publicly exporting `{forbidden}`"
        );
    }

    assert!(
        module_host_mod_source.contains("pub use module_registration::module_descriptor;"),
        "runtime graphics module host should keep module_descriptor on its own re-export line to avoid grouped-use name collisions"
    );
    assert!(
        !module_host_mod_source.contains("pub use module_registration::{"),
        "runtime graphics module host should not group module_descriptor with other module_registration re-exports"
    );

    for forbidden in [
        "create_render_service",
        "create_runtime_preview_renderer",
        "create_shared_texture_render_service",
    ] {
        assert!(
            !create_mod_source.contains(forbidden),
            "runtime graphics create surface should stop routing legacy helper `{forbidden}`"
        );
    }

    for forbidden in [
        "manager_create_runtime_preview_renderer",
        "manager_spawn_render_service",
        "manager_spawn_shared_texture_render_service",
    ] {
        assert!(
            !rendering_manager_mod_source.contains(forbidden),
            "runtime rendering manager should stop owning legacy helper module `{forbidden}`"
        );
    }
}
