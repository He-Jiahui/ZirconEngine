#[test]
fn editor_viewport_sources_route_through_render_server_without_wgpu_preview_bindings() {
    let app_source = include_str!("../../host/slint_host/app.rs");
    let viewport_source = include_str!("../../host/slint_host/viewport.rs");
    let manifest = include_str!("../../../Cargo.toml");

    assert!(
        viewport_source.contains("resolve_render_server"),
        "editor viewport controller should resolve RenderServer from core"
    );
    assert!(
        viewport_source.contains("submit_frame_extract"),
        "editor viewport controller should submit RenderFrameExtract through RenderServer"
    );
    assert!(
        viewport_source.contains("capture_frame"),
        "editor viewport controller should pull captured frames through RenderServer"
    );

    for forbidden in [
        "wgpu::",
        "SharedTextureRenderService",
        "create_shared_texture_render_service",
        "ViewportTextureBridge",
        "RuntimePreviewRenderer",
        "create_runtime_preview_renderer",
    ] {
        assert!(
            !app_source.contains(forbidden),
            "editor app source should not reference `{forbidden}` after RenderServer migration"
        );
        assert!(
            !viewport_source.contains(forbidden),
            "editor viewport source should not reference `{forbidden}` after RenderServer migration"
        );
    }

    assert!(
        !manifest.contains("wgpu.workspace = true"),
        "zircon_editor/Cargo.toml should not depend on wgpu directly"
    );
}
