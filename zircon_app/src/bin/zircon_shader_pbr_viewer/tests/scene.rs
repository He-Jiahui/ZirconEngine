const SOURCE: &str = include_str!("../scene.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("scene source should retain a test-module boundary")
}

fn assert_source_order(anchors: &[&str]) {
    let source = production_source();
    let mut offset = 0;
    for anchor in anchors {
        let relative = source[offset..]
            .find(anchor)
            .unwrap_or_else(|| panic!("missing viewer architecture anchor: {anchor}"));
        offset += relative + anchor.len();
    }
}

#[test]
fn render_framework_drops_before_its_runtime_services() {
    let fields = production_source()
        .split_once("pub(crate) struct PbrMirrorScene {")
        .and_then(|(_, rest)| rest.split_once('}'))
        .map(|(fields, _)| fields)
        .expect("PbrMirrorScene fields should remain visible to the lifecycle guard");

    assert!(
        fields
            .find("render_framework:")
            .expect("render framework field")
            < fields.find("_asset_runtime:").expect("runtime owner field"),
        "Rust drops struct fields in declaration order, so the runtime owner must follow the render framework"
    );
}

#[test]
fn scene_teardown_releases_world_viewport_framework_and_runtime_in_order() {
    assert_source_order(&[
        "self.world.take();",
        "framework.unbind_viewport_surface(self.viewport)",
        "framework.destroy_viewport(self.viewport)",
        "self.render_framework.take();",
        "self._asset_runtime.take();",
    ]);
    assert!(
        !production_source().contains("impl Drop for ViewerWorkPaths"),
        "the reusable project cache must outlive a single viewer process"
    );
}

#[test]
fn native_viewport_surface_is_owned_by_the_render_framework() {
    let fields = production_source()
        .split_once("pub(crate) struct PbrMirrorScene {")
        .and_then(|(_, rest)| rest.split_once('}'))
        .map(|(fields, _)| fields)
        .expect("PbrMirrorScene fields should remain visible to the lifecycle guard");

    assert!(
        !fields.contains("viewport_surface:"),
        "the viewer must not own a backend viewport surface outside the render framework"
    );
    assert_source_order(&[
        "pub(crate) fn attach_viewport_surface(",
        "framework.bind_viewport_surface(self.viewport, descriptor)?",
        "pub(crate) fn render_to_viewport_surface(",
        "framework.present_frame_extract(self.viewport, extract)?",
        "pub(crate) fn detach_viewport_surface(",
        "framework.unbind_viewport_surface(self.viewport)",
    ]);
    assert!(!production_source().contains("SceneViewportSurface"));
}

#[test]
fn viewer_ibl_staging_uses_resolved_work_paths() {
    assert_source_order(&[
        "let work_paths = ViewerWorkPaths::new(work_dir, ibl_cache_dir);",
        "work_paths.ibl_cache_root(),",
    ]);
    assert!(
        !production_source().contains("paths.cache_root(),"),
        "viewer staging must not use the viewer project asset cache"
    );
}

#[test]
fn viewer_caches_static_preview_environment_outside_the_frame_loop() {
    let source = production_source();
    assert_source_order(&[
        "let environment = EnvironmentExtract::source_cubemap(environment);",
        "let preview = PreviewEnvironmentExtract::from_environment(&environment, true, Vec4::ZERO);",
        "preview,",
    ]);
    assert!(
        source.contains("snapshot.preview = self.preview.clone();"),
        "the frame loop must reuse the scene's static preview extract"
    );
    assert!(
        !source.contains("PreviewEnvironmentExtract::from_environment(&snapshot.environment"),
        "the frame loop must not rederive preview settings from immutable HDRI state"
    );
}

#[test]
fn viewer_exposes_nonblocking_base_pipeline_admission_retry() {
    assert_source_order(&[
        "pub(crate) fn required_material_base_pipeline_ready(",
        "pub(crate) fn retry_required_material_base_pipeline_admission(",
        ".retry_environment_only_pbr_base_pipeline_admission()?;",
    ]);
    let source = production_source();
    assert!(source.contains("framework.pbr_ior_forward_base_pipeline_ready()?"));
    assert!(
        source.contains("framework.queue_pbr_ior_forward_base_pipeline_admission()?"),
        "the IOR fixture must retry its exact generic Forward variant"
    );
}

#[test]
fn dielectric_fixture_uses_an_isolated_project_and_only_its_generic_forward_pso() {
    let source = production_source();

    assert_source_order(&[
        "material_fixture.project_root_component()",
        "work_paths.project_root().join(component)",
        "write_viewer_project_assets_for_fixture(&asset_root, material_fixture)?",
        "startup_options.without_environment_only_pbr_base_prewarm()",
        "WgpuRenderFramework::new_with_startup_options_and_report(",
        "render_framework.queue_pbr_ior_forward_base_pipeline_admission()?",
    ]);
    assert!(
        source.contains("if material_fixture == ViewerMaterialFixture::MetalMirror")
            && source.contains("base_prewarm_report.is_none()"),
        "only the mirror baseline may require the specialized prewarm report"
    );
}

#[test]
fn viewer_exposes_first_frame_extract_renderer_call_and_readback_timing() {
    let source = production_source();
    assert_source_order(&[
        "let capture_frame_timing = self.frame_timing_report_requested;",
        "let render_extract_started = capture_frame_timing.then(Instant::now);",
        "let render_extract =\n            render_extract_started",
        "framework.submit_frame_extract(self.viewport, extract)?;",
        ".capture_frame(self.viewport)?",
        "if capture_frame_timing {",
        "self.frame_timing_report_requested = false;",
        "self.last_frame_timing = PbrMirrorSceneFrameTimingReport::new(",
    ]);
    assert!(
        source.contains("pub(crate) const fn last_frame_timing_report(&self)"),
        "the app host must be able to read the decomposed timing after a completed frame"
    );
    assert!(
        source.contains("pub(crate) fn request_next_frame_timing_report(&mut self)"),
        "non-measurement viewer frames must not read timing clocks"
    );
}

#[test]
fn viewer_frame_timing_declares_the_direct_surface_cpu_boundary() {
    let source = production_source();

    assert!(source.contains("CPU wall-clock observed inside the renderer frame call"));
    assert!(source.contains(
        "direct surface path additionally includes surface acquisition, blit, and present"
    ));
    assert!(source.contains("This is not a GPU execution-duration measurement."));
}

#[test]
fn viewer_reserves_cpu_frame_rendering_for_image_consumers() {
    let source = production_source();
    assert_source_order(&[
        "pub(crate) fn render(",
        ") -> Result<ViewportFrame, Box<dyn Error>> {",
        "framework.submit_frame_extract(self.viewport, extract)?;",
        ".capture_frame(self.viewport)?",
    ]);
    assert_eq!(
        source.matches(".capture_frame(self.viewport)?").count(),
        1,
        "the viewer must not issue and discard CPU ViewportFrame readbacks for warmup"
    );
    assert!(!source.contains("SceneRenderer::"));
    assert!(
        !source.contains("warm_up_first_frame"),
        "pipeline warmup requires the Render17-owned no-readback API"
    );
}

#[test]
fn ibl_load_report_retains_the_active_cubemap_and_pmrem_layout() {
    let report = super::PbrMirrorSceneIblLoadReport::new(
        zircon_runtime::asset::importer::EnvironmentIblSourceStagingStatus::Reused,
        std::time::Duration::from_millis(3),
        zircon_runtime::asset::importer::EnvironmentIblSourceStagingTiming::default(),
        zircon_runtime::asset::importer::EnvironmentIblSourceStagingOutput::default(),
        std::time::Duration::from_millis(7),
        512,
        10,
        256,
        9,
    );

    assert_eq!(report.source_cubemap_face_size(), 512);
    assert_eq!(report.source_cubemap_mip_count(), 10);
    assert_eq!(report.pmrem_face_size(), 256);
    assert_eq!(report.pmrem_mip_count(), 9);
    assert_eq!(report.staging_output().parallel_executor_work_items(), 0);
}

#[test]
fn frame_timing_report_preserves_extract_renderer_call_and_readback_boundaries() {
    let report = super::PbrMirrorSceneFrameTimingReport::new(
        std::time::Duration::from_millis(3),
        std::time::Duration::from_millis(5),
        std::time::Duration::from_millis(7),
    );

    assert_eq!(report.render_extract(), std::time::Duration::from_millis(3));
    assert_eq!(
        report.renderer_frame_call(),
        std::time::Duration::from_millis(5)
    );
    assert_eq!(
        report.readback_and_completion(),
        std::time::Duration::from_millis(7)
    );
}

#[test]
fn startup_timing_reports_deferred_shader_and_standard_pso_boundaries() {
    let source = production_source();

    for expected in [
        "lighting_source_assembly={:.2?}",
        "pipeline_foundation={:.2?}",
        "standard_pso={:.2?}",
        "base_prewarm={base_prewarm_report:?}",
        "core_startup.deferred_lighting_shader_source_assembly()",
        "core_startup.deferred_lighting_pipeline_foundation()",
        "core_startup.deferred_lighting_standard_pipeline()",
        ".environment_only_pbr_base_prewarm()",
        "cache_hit: report.cache_hit()",
    ] {
        assert!(
            source.contains(expected),
            "viewer startup timing must retain `{expected}`"
        );
    }
}

#[test]
fn viewer_uses_real_runtime_module_and_asset_manager_lifecycle() {
    let source = production_source();
    assert!(
        !source.contains("fn viewer_uses_real_runtime_module_and_asset_manager_lifecycle"),
        "architecture guards must never search their own anchor strings"
    );
    assert_source_order(&[
        "let hdri = preflight_viewer_hdri(hdri_path, face_size, pmrem_face_size)?;",
        "let work_paths = ViewerWorkPaths::new(work_dir, ibl_cache_dir);",
        "register_module(zircon_runtime::foundation::module_descriptor())",
        "register_module(TasksModule.descriptor())",
        "register_module(zircon_runtime::asset::module_descriptor())",
        "activate_module(zircon_runtime::foundation::FOUNDATION_MODULE_NAME)",
        "activate_module(TASKS_MODULE_NAME)",
        "activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)",
        "ProjectAssetManagerAccess::new",
        "asset_manager.open_project(work_paths.project_root().to_string_lossy().as_ref())",
        "asset_manager.current_project_manager()",
        "let startup_options = SceneRendererStartupOptions::environment_only_pbr_preview()",
        ".with_async_pipeline_compile();",
        "let startup_options = if gpu_timing_enabled {",
        "startup_options.with_gpu_timing()",
        "WgpuRenderFramework::new_with_startup_options_and_report(",
        "asset_access,",
        "startup_options,",
    ]);
    assert!(
        !source.contains("ProjectManager::open("),
        "the viewer must reuse the asset manager's scanned project instead of reopening it"
    );
    assert!(
        !source.contains(concat!("ProjectAssetManager::", "default")),
        "viewer must not construct a direct default asset manager"
    );
    assert!(
        !source.contains(concat!("new_", "for_test")),
        "viewer must not use a test-only renderer path"
    );
}

#[test]
fn cancelled_scene_load_checks_each_bootstrap_boundary_before_publication() {
    let source = production_source();

    assert!(!source.contains("pub(crate) fn new("));
    assert_source_order(&[
        "pub(crate) fn new_with_cancellation(",
        "check_scene_load_cancellation(cancellation, \"before HDRI preflight\")?;",
        "let hdri = preflight_viewer_hdri(hdri_path, face_size, pmrem_face_size)?;",
        "check_scene_load_cancellation(cancellation, \"after HDRI preflight\")?;",
        "check_scene_load_cancellation(cancellation, \"after project assets\")?;",
        "check_scene_load_cancellation(cancellation, \"after runtime bootstrap\")?;",
        "check_scene_load_cancellation(cancellation, \"after project open\")?;",
        "check_scene_load_cancellation(cancellation, \"after world load\")?;",
        "check_scene_load_cancellation(cancellation, \"after renderer bootstrap\")?;",
        "check_scene_load_cancellation(cancellation, \"after IBL restore\")?;",
        "Ok(scene)",
    ]);
}

#[test]
fn viewer_reuses_completed_project_assets_before_opening_the_runtime_project() {
    assert_source_order(&[
        "let asset_root = manifest.primary_asset_root_path(&paths)?;",
        "viewer_project_assets_are_ready_for_fixture(&asset_root, material_fixture);",
        "let project_asset_generation = if project_assets_reused {",
        "ViewerProjectAssetGenerationReport::reused()",
        "write_viewer_project_assets_for_fixture(&asset_root, material_fixture)?",
        "asset_manager.open_project(project_root.to_string_lossy().as_ref())",
        "project_runtime_report.record_open(&project_info);",
    ]);
}

#[test]
fn viewer_startup_reports_generated_artifact_and_runtime_open_counts() {
    let source = production_source();
    for field in [
        "mesh_generation_samples={}",
        "serialized_source_bytes={}",
        "asset_filesystem_writes={}",
        "project_manifest_writes={}",
        "startup_filesystem_writes={}",
        "project_open_count={}",
        "imported_assets={}",
        "ready_assets={}",
    ] {
        assert!(
            source.contains(field),
            "viewer startup timing must retain `{field}`"
        );
    }
    assert_source_order(&[
        "let mut manifest = ProjectManifest::new(\"ShaderPbrMirrorViewer\", scene_uri.clone(), 1);",
        "manifest.asset_roots = vec![RelPath::parse(VIEWER_PROJECT_ASSET_ROOT)?];",
        "let manifest_needs_publish = !paths.manifest_path().is_file();",
        "if manifest_needs_publish {",
        "manifest.save(paths.manifest_path())?;",
    ]);
}
