#[test]
fn editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings() {
    let app_source = include_str!("../../../ui/retained_host/app.rs");
    let viewport_new_source = include_str!("../../../ui/retained_host/viewport/new.rs");
    let viewport_state_source =
        include_str!("../../../ui/retained_host/viewport/viewport_state.rs");
    let viewport_submit_source =
        include_str!("../../../ui/retained_host/viewport/submit_extract.rs");
    let viewport_poll_source = include_str!("../../../ui/retained_host/viewport/poll_image.rs");
    let manifest = include_str!("../../../../Cargo.toml");

    assert!(
        viewport_new_source.contains("ViewportState::lazy(core)")
            && viewport_state_source.contains("resolve_render_framework(&core)")
            && viewport_state_source.contains("async_resolve_render_framework"),
        "editor viewport controller should lazily resolve RenderFramework from core"
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
                && !viewport_state_source.contains(forbidden)
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
fn editor_retained_host_presenter_boundary_keeps_wgpu_inside_runtime_rhi() {
    let app_source = include_str!("../../../ui/retained_host/app.rs");
    let host_window_source = include_str!("../../../ui/retained_host/host_contract/window.rs");
    let presenter_root_source =
        include_str!("../../../ui/retained_host/host_contract/presenter/mod.rs");
    let presenter_trait_source =
        include_str!("../../../ui/retained_host/host_contract/presenter/host_chrome_presenter.rs");
    let presenter_backend_source =
        include_str!("../../../ui/retained_host/host_contract/presenter/backend.rs");
    let presenter_factory_source =
        include_str!("../../../ui/retained_host/host_contract/presenter/factory.rs");
    let presenter_softbuffer_source =
        include_str!("../../../ui/retained_host/host_contract/presenter/softbuffer.rs");
    let presenter_gpu_source =
        include_str!("../../../ui/retained_host/host_contract/presenter/gpu.rs");
    let presenter_command_source =
        include_str!("../../../ui/retained_host/host_contract/presenter/command_stream.rs");
    let manifest = include_str!("../../../../../Cargo.toml");
    let editor_manifest = include_str!("../../../../Cargo.toml");
    let former_owner = ["sli", "nt"].concat();

    assert!(
        editor_manifest.contains("winit.workspace = true")
            && editor_manifest.contains("softbuffer"),
        "editor host manifest should use winit plus the Rust-owned software presenter"
    );
    assert!(
        host_window_source.contains("HostChromePresenter")
            && host_window_source.contains("create_host_chrome_presenter"),
        "retained host window should depend on the presenter seam instead of a concrete backend"
    );
    assert!(
        presenter_root_source.contains("mod softbuffer;")
            && presenter_softbuffer_source.contains("SoftbufferHostPresenter"),
        "softbuffer should remain the Rust-owned fallback presenter behind the seam"
    );
    assert!(
        presenter_trait_source.contains("trait HostChromePresenter")
            && presenter_factory_source.contains("Box<dyn HostChromePresenter>"),
        "retained host presentation should route through the object-safe presenter boundary"
    );
    assert!(
        presenter_backend_source.contains("Gpu")
            && presenter_backend_source.contains("default_native()")
            && host_window_source.contains("HostPresenterBackend::default_native()")
            && host_window_source.contains("HostPresenterBackend::fallback()"),
        "native retained host windows should default to GPU and keep softbuffer as an explicit fallback"
    );
    assert!(
        presenter_gpu_source.contains("zircon_runtime::rhi")
            && presenter_factory_source.contains("create_default_ui_surface_presenter")
            && presenter_factory_source.contains("UiSurfaceDescriptor::from_winit_window")
            && presenter_command_source.contains("ChromeCommandStream"),
        "GPU presenter work should consume the runtime RHI and backend-neutral command stream"
    );
    assert!(
        !manifest.contains(&former_owner) && !editor_manifest.contains(&former_owner),
        "workspace manifests should not keep the deleted generated UI dependency"
    );

    for source in [
        app_source,
        host_window_source,
        presenter_root_source,
        presenter_trait_source,
        presenter_backend_source,
        presenter_factory_source,
        presenter_softbuffer_source,
        presenter_gpu_source,
        presenter_command_source,
    ] {
        assert!(
            !source.contains("wgpu::"),
            "retained editor host presenter sources should not reference raw wgpu APIs"
        );
        assert!(
            !source.contains("rhi_wgpu"),
            "retained editor host presenter sources should not name concrete runtime RHI backends"
        );
        assert!(
            !source.contains(".backend_name(")
                && !source.contains(".renderer_name(")
                && !source.contains(".require_wgpu_27("),
            "retained editor host should not select generated UI backends or renderer flags"
        );
    }
}
