use super::sources::runtime_surface_present_source;

#[test]
fn runtime_sources_route_preview_through_dynamic_api_without_app_wgpu_dependency() {
    let lib_source = include_str!("../../../lib.rs");
    let production_lib_source = lib_source
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or(lib_source);
    let presenter_source = include_str!("../../../runtime_presenter.rs");
    let runtime_surface_present_source = runtime_surface_present_source();
    let manifest = include_str!("../../../../Cargo.toml");

    assert!(
        presenter_source.contains("softbuffer"),
        "runtime presenter should blit runtime-owned frame output through softbuffer"
    );
    assert!(
        runtime_surface_present_source.contains("capture_frame"),
        "runtime preview should request frames through the runtime dynamic API"
    );
    assert!(
        presenter_source.contains("RuntimeFrame"),
        "runtime presenter should consume runtime interface frames, not runtime implementation frames"
    );

    for forbidden in [
        "wgpu::",
        "RenderFrameExtract",
        "RenderFrameworkRuntimeBridge",
        "RuntimePreviewRenderer",
        "create_runtime_preview_renderer",
        "SharedTextureRenderService",
        "RenderService",
    ] {
        assert!(
            !production_lib_source.contains(forbidden),
            "runtime entry source should not reference `{forbidden}` after dynamic runtime migration"
        );
        assert!(
            !presenter_source.contains(forbidden),
            "runtime presenter source should not reference `{forbidden}` after dynamic runtime migration"
        );
    }

    assert!(
        !manifest.contains("wgpu.workspace = true"),
        "zircon_app/Cargo.toml should not depend on wgpu directly"
    );
}
