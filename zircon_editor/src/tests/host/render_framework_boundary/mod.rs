#[test]
fn editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings() {
    let app_source = include_str!("../../../ui/retained_host/app.rs");
    let viewport_new_source = include_str!("../../../ui/retained_host/viewport/new.rs");
    let viewport_state_source =
        include_str!("../../../ui/retained_host/viewport/viewport_state.rs");
    let viewport_access_source =
        include_str!("../../../ui/retained_host/viewport/render_framework_access.rs");
    let runtime_lease_source = include_str!("../../../ui/retained_host/app/runtime_lease.rs");
    let viewport_resolve_job_source =
        include_str!("../../../ui/retained_host/viewport/render_framework_resolve_job.rs");
    let viewport_submit_source =
        include_str!("../../../ui/retained_host/viewport/submit_extract.rs");
    let viewport_lifecycle_source =
        include_str!("../../../ui/retained_host/viewport/viewport_lifecycle.rs");
    let viewport_poll_source =
        include_str!("../../../ui/retained_host/viewport/poll_captured_frame.rs");
    let viewport_redraw_source =
        include_str!("../../../ui/retained_host/app/viewport_image_redraw.rs");
    let host_viewport_image_source =
        include_str!("../../../ui/retained_host/host_contract/data/viewport_image.rs");
    let host_viewport_image_production_source = host_viewport_image_source
        .split("\n#[cfg(test)]")
        .next()
        .expect("viewport image source should have a production section");
    let startup_assembly_source = include_str!(
        "../../../ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs"
    );
    let manifest = include_str!("../../../../Cargo.toml");

    assert!(
        viewport_new_source.contains("ViewportState::lazy(render_framework_access)")
            && viewport_state_source
                .contains("RenderFrameworkResolveJob::new(render_framework_access)")
            && viewport_state_source
                .contains("JobTicket<ManagerServiceHandle<dyn RenderFramework>>")
            && viewport_state_source.contains("JobCategory::Misc")
            && viewport_access_source.contains("struct ViewportRenderFrameworkAccess")
            && viewport_access_source.contains("CoreWeak")
            && viewport_access_source.contains("render_framework_handle(&core)")
            && viewport_access_source.contains("resolve_manager_service(&core, handle)")
            && runtime_lease_source.contains("ViewportRenderFrameworkAccess::new(&self.core)")
            && viewport_resolve_job_source.contains("context.check_cancelled()?")
            && !viewport_state_source.contains("CoreHandle")
            && !viewport_resolve_job_source.contains("CoreHandle")
            && startup_assembly_source.contains("viewport.bind_jobs(editor_jobs.clone())"),
        "editor viewport controller should lazily resolve RenderFramework through a typed access without retaining raw CoreHandle"
    );
    let thread_builder = ["std::thread", "::Builder"].concat();
    let join_handle = ["Join", "Handle"].concat();
    assert!(
        !viewport_state_source.contains(&thread_builder)
            && !viewport_state_source.contains(&join_handle),
        "viewport lazy resolve must be owned by EditorJobSystem typed tickets"
    );
    assert!(
        viewport_submit_source.contains("submit_frame_extract"),
        "editor viewport controller should submit RenderFrameExtract through RenderFramework"
    );
    assert!(
        viewport_submit_source.contains("let _operation = self.lock_viewport_lifecycle();")
            && viewport_submit_source.contains("let Some((viewport, render_framework))")
            && viewport_submit_source.contains("render_framework.submit_frame_extract_with_ui")
            && viewport_submit_source
                .contains("render_framework.query_visible_spatial_snapshot(viewport)")
            && viewport_submit_source.contains("active.handle == viewport"),
        "editor viewport controller should retain the viewport operation while submitting outside its state mutex"
    );
    assert!(
        !viewport_lifecycle_source.contains("let _operation")
            && viewport_lifecycle_source.contains("render_framework.destroy_viewport")
            && viewport_lifecycle_source.contains("render_framework.create_viewport")
            && viewport_lifecycle_source.contains("render_framework.set_quality_profile"),
        "viewport recreation must be called under the controller operation gate"
    );
    assert!(
        viewport_poll_source.contains("poll_captured_frame_if_newer")
            && viewport_poll_source.contains("shared.latest_generation")
            && !viewport_state_source.contains("latest_image")
            && !viewport_poll_source.contains("SharedPixelBuffer")
            && !viewport_poll_source.contains("Image::"),
        "editor viewport fallback should transfer the captured RGBA owner after generation validation"
    );
    assert!(
        viewport_redraw_source.contains("poll_captured_frame()")
            && viewport_redraw_source.contains("set_viewport_capture(viewport, frame)")
            && host_viewport_image_production_source.contains("rgba: frame.rgba")
            && host_viewport_image_production_source
                .contains("viewport_image_resource_key(viewport, generation)")
            && !host_viewport_image_production_source.contains("DefaultHasher")
            && !host_viewport_image_production_source.contains("to_rgba8")
            && !host_viewport_image_production_source.contains("to_vec"),
        "async viewport fallback must move the captured RGBA Vec into host presentation data without a content hash"
    );
    assert!(
        viewport_poll_source.contains("let poll_request = {")
            && viewport_poll_source.contains("render_framework.poll_captured_frame_if_newer"),
        "editor viewport controller should release its state mutex before polling the render framework"
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
                && !viewport_lifecycle_source.contains(forbidden)
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
fn retained_host_composition_owns_runtime_lifetime_outside_viewport_state() {
    let app_source = include_str!("../../../ui/retained_host/app.rs");
    let constructors_source =
        include_str!("../../../ui/retained_host/app/host_lifecycle/startup/constructors.rs");
    let startup_source =
        include_str!("../../../ui/retained_host/app/host_lifecycle/startup/with_viewport.rs");
    let construction_input_source = include_str!(
        "../../../ui/retained_host/app/host_lifecycle/startup/state/construction/input.rs"
    );
    let assembly_source = include_str!(
        "../../../ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs"
    );
    let viewport_state_source =
        include_str!("../../../ui/retained_host/viewport/viewport_state.rs");
    let viewport_job_source =
        include_str!("../../../ui/retained_host/viewport/render_framework_resolve_job.rs");

    assert!(
        app_source.contains("runtime_lease: RetainedHostRuntimeLease")
            && constructors_source.contains("RetainedHostRuntimeLease::new(core)")
            && startup_source.contains("runtime_lease.bootstrap_core()")
            && construction_input_source.contains("runtime_lease:")
            && construction_input_source.contains("RetainedHostRuntimeLease")
            && assembly_source.contains("runtime_lease,")
            && !viewport_state_source.contains("CoreHandle")
            && !viewport_job_source.contains("CoreHandle"),
        "retained-host composition must retain the runtime while viewport state and jobs use typed weak access"
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
    let host_window_event_loop_source =
        include_str!("../../../ui/retained_host/host_contract/window/event_loop.rs");
    let host_window_lifecycle_presenter_source = include_str!(
        "../../../ui/retained_host/host_contract/window/event_loop/lifecycle/presenter.rs"
    );
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
        include_str!("../../../ui/retained_host/host_contract/chrome_command_stream/mod.rs");
    let manifest = include_str!("../../../../../Cargo.toml");
    let editor_manifest = include_str!("../../../../Cargo.toml");
    let former_owner = ["sli", "nt"].concat();

    assert!(
        editor_manifest.contains("winit.workspace = true")
            && editor_manifest.contains("softbuffer"),
        "editor host manifest should use winit plus the Rust-owned software presenter"
    );
    assert!(
        host_window_event_loop_source.contains("Box<dyn HostChromePresenter>")
            && host_window_lifecycle_presenter_source.contains("create_host_chrome_presenter"),
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
            && host_window_lifecycle_presenter_source
                .contains("HostPresenterBackend::default_native()")
            && host_window_lifecycle_presenter_source.contains("HostPresenterBackend::fallback()"),
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
        host_window_event_loop_source,
        host_window_lifecycle_presenter_source,
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
