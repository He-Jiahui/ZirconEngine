#[test]
fn editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings() {
    let app_source = include_str!("../../../ui/slint_host/app.rs");
    let viewport_new_source = include_str!("../../../ui/slint_host/viewport/new.rs");
    let viewport_submit_source = include_str!("../../../ui/slint_host/viewport/submit_extract.rs");
    let viewport_poll_source = include_str!("../../../ui/slint_host/viewport/poll_image.rs");
    let manifest = include_str!("../../../../Cargo.toml");

    assert!(
        viewport_new_source.contains("resolve_render_framework"),
        "editor viewport controller should resolve RenderFramework from core"
    );
    assert!(
        viewport_submit_source.contains("submit_frame_extract"),
        "editor viewport controller should submit RenderFrameExtract through RenderFramework"
    );
    assert!(
        viewport_poll_source.contains("capture_frame"),
        "editor viewport controller should pull captured frames through RenderFramework"
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
            "editor app source should not reference `{forbidden}` after RenderFramework migration"
        );
        assert!(
            !viewport_new_source.contains(forbidden)
                && !viewport_submit_source.contains(forbidden)
                && !viewport_poll_source.contains(forbidden),
            "editor viewport sources should not reference `{forbidden}` after RenderFramework migration"
        );
    }

    assert!(
        !manifest.contains("wgpu.workspace = true"),
        "zircon_editor/Cargo.toml should not depend on wgpu directly"
    );
}

#[test]
fn editor_viewport_interaction_boundary_lives_in_editor_crate() {
    let lib_source = include_str!("../../../lib.rs");
    let scene_source = include_str!("../../../scene/mod.rs");
    let viewport_source = include_str!("../../../scene/viewport/mod.rs");
    let manifest = include_str!("../../../../Cargo.toml");

    assert!(
        scene_source.contains("pub mod viewport;"),
        "zircon_editor should expose the scene viewport module directly"
    );
    assert!(
        viewport_source.contains(
            "pub use interaction::{GizmoAxis, ViewportFeedback, ViewportInput, ViewportState};"
        ),
        "scene::viewport should own the editor viewport interaction types directly"
    );
    assert!(
        !lib_source.contains(
            "pub use scene::viewport::{GizmoAxis, ViewportFeedback, ViewportInput, ViewportState};"
        ),
        "zircon_editor should not keep a lib.rs compatibility re-export for viewport interaction types"
    );
    assert!(
        !manifest.contains("zircon_graphics = { path = \"../zircon_graphics\" }"),
        "zircon_editor/Cargo.toml should not depend on zircon_graphics after viewport interaction extraction"
    );
}

#[test]
fn editor_slint_host_prefers_winit_software_renderer_over_skia_wgpu_selection() {
    let app_source = include_str!("../../../ui/slint_host/app.rs");
    let manifest = include_str!("../../../../../Cargo.toml");

    assert!(
        manifest.contains("renderer-software"),
        "workspace slint dependency should keep the editor on the software renderer"
    );
    assert!(
        !manifest.contains("renderer-skia"),
        "workspace slint dependency should not require the skia renderer for editor host tests"
    );
    assert!(
        !manifest.contains("unstable-wgpu-27"),
        "workspace slint dependency should not require the unstable wgpu-27 path for the editor host"
    );

    assert!(
        app_source.contains(".backend_name(\"winit\".into())"),
        "editor host should explicitly select the winit backend"
    );
    assert!(
        app_source.contains(".renderer_name(\"software\".into())"),
        "editor host should explicitly select the software renderer"
    );
    assert!(
        !app_source.contains(".require_wgpu_27("),
        "editor host should not hard-require the unstable Slint wgpu-27 selector"
    );
}
