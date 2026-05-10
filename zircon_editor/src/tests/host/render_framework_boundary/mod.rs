#[test]
fn editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings() {
    let app_source = include_str!("../../../ui/retained_host/app.rs");
    let viewport_new_source = include_str!("../../../ui/retained_host/viewport/new.rs");
    let viewport_submit_source =
        include_str!("../../../ui/retained_host/viewport/submit_extract.rs");
    let viewport_poll_source = include_str!("../../../ui/retained_host/viewport/poll_image.rs");
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
fn editor_retained_host_uses_winit_software_presenter_without_backend_selector() {
    let app_source = include_str!("../../../ui/retained_host/app.rs");
    let host_window_source = include_str!("../../../ui/retained_host/host_contract/window.rs");
    let manifest = include_str!("../../../../../Cargo.toml");
    let editor_manifest = include_str!("../../../../Cargo.toml");
    let former_owner = ["sli", "nt"].concat();

    assert!(
        editor_manifest.contains("winit.workspace = true")
            && editor_manifest.contains("softbuffer"),
        "editor host manifest should use winit plus the Rust-owned software presenter"
    );
    assert!(
        host_window_source.contains("SoftbufferHostPresenter"),
        "retained host window should present through the Rust-owned software presenter"
    );
    assert!(
        !manifest.contains(&former_owner) && !editor_manifest.contains(&former_owner),
        "workspace manifests should not keep the deleted generated UI dependency"
    );

    assert!(
        !app_source.contains(".backend_name(")
            && !app_source.contains(".renderer_name(")
            && !app_source.contains(".require_wgpu_27("),
        "retained editor host should not select generated UI backends or renderer flags"
    );
}
