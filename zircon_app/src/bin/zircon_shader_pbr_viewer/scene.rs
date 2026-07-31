use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;

use zircon_runtime::asset::importer::EnvironmentIblSourceStagingStatus;
use zircon_runtime::asset::pipeline::manager::{
    project_asset_manager_handle, AssetManager, ProjectAssetManagerAccess,
};
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    EnvironmentExtract, PreviewEnvironmentExtract, RenderOverlayExtract, RenderSceneSnapshot,
    RenderViewportSurfaceDescriptor, SceneViewportExtractRequest, ViewportRenderSettings,
};
use zircon_runtime::core::math::{UVec2, Vec4};
use zircon_runtime::core::runtime::modules::{TasksModule, TASKS_MODULE_NAME};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::graphics::{
    SceneRenderer, SceneRendererStartupOptions, SceneViewportSurface, ViewportFrame,
};

use crate::camera::{camera_render_descriptor, OrbitCamera};
use crate::hdri::{decode_viewer_hdri, source_cubemap_environment, SourceCubemapEnvironmentLoad};
use crate::project_assets::{viewer_project_assets_are_ready, write_viewer_project_assets};

const VIEWER_PROJECT_CACHE_VERSION: u32 = 2;

pub(crate) struct PbrMirrorScene {
    _project_root: CachedProjectRoot,
    world: Option<zircon_runtime::scene::world::World>,
    // The native surface must drop before its renderer/device owner.
    viewport_surface: Option<SceneViewportSurface>,
    renderer: Option<SceneRenderer>,
    environment: EnvironmentExtract,
    preview: PreviewEnvironmentExtract,
    ibl_load_report: PbrMirrorSceneIblLoadReport,
    frame_timing_report_requested: bool,
    last_frame_timing: PbrMirrorSceneFrameTimingReport,
    // Keep the cache root alive until runtime teardown releases its file watchers.
    _asset_runtime: Option<CoreRuntime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PbrMirrorSceneIblLoadReport {
    staging_status: EnvironmentIblSourceStagingStatus,
    staging_elapsed: std::time::Duration,
    total_elapsed: std::time::Duration,
}

impl PbrMirrorSceneIblLoadReport {
    pub(crate) const fn new(
        staging_status: EnvironmentIblSourceStagingStatus,
        staging_elapsed: std::time::Duration,
        total_elapsed: std::time::Duration,
    ) -> Self {
        Self {
            staging_status,
            staging_elapsed,
            total_elapsed,
        }
    }

    pub(crate) const fn staging_status(self) -> EnvironmentIblSourceStagingStatus {
        self.staging_status
    }

    pub(crate) const fn staging_elapsed(self) -> std::time::Duration {
        self.staging_elapsed
    }

    pub(crate) const fn total_elapsed(self) -> std::time::Duration {
        self.total_elapsed
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PbrMirrorSceneFrameTimingReport {
    render_extract: std::time::Duration,
    renderer_frame_call: std::time::Duration,
    readback_and_completion: std::time::Duration,
}

impl PbrMirrorSceneFrameTimingReport {
    const fn new(
        render_extract: std::time::Duration,
        renderer_frame_call: std::time::Duration,
        readback_and_completion: std::time::Duration,
    ) -> Self {
        Self {
            render_extract,
            renderer_frame_call,
            readback_and_completion,
        }
    }

    pub(crate) const fn render_extract(self) -> std::time::Duration {
        self.render_extract
    }

    /// CPU wall-clock observed inside the renderer frame call.
    /// The direct surface path additionally includes surface acquisition, blit, and present.
    /// This is not a GPU execution-duration measurement.
    pub(crate) const fn renderer_frame_call(self) -> std::time::Duration {
        self.renderer_frame_call
    }

    pub(crate) const fn readback_and_completion(self) -> std::time::Duration {
        self.readback_and_completion
    }
}

impl PbrMirrorScene {
    // The HDRI loader resolves an omitted face size after decoding the source dimensions.
    pub(crate) fn new(
        hdri_path: &Path,
        face_size: Option<u32>,
        pmrem_face_size: Option<u32>,
        ibl_cache_dir: Option<&Path>,
    ) -> Result<Self, Box<dyn Error>> {
        let scene_load_started = Instant::now();
        let decoded_hdri = decode_viewer_hdri(hdri_path, face_size, pmrem_face_size)?;
        let hdri_decode_elapsed = scene_load_started.elapsed();
        let project_assets_started = Instant::now();
        let project_root = CachedProjectRoot::new();
        let paths = ProjectPaths::from_root(project_root.as_path())?;
        let scene_uri = AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml")?;
        let manifest = ProjectManifest::new("ShaderPbrMirrorViewer", scene_uri.clone(), 1);
        paths.ensure_layout(&manifest.asset_roots)?;
        manifest.save(paths.manifest_path())?;
        let asset_root = manifest.primary_asset_root_path(&paths)?;
        let project_assets_reused = viewer_project_assets_are_ready(&asset_root);
        if !project_assets_reused {
            write_viewer_project_assets(&asset_root)?;
        }
        let project_assets_elapsed = project_assets_started.elapsed();

        let runtime_bootstrap_started = Instant::now();
        let asset_runtime = CoreRuntime::new();
        asset_runtime.register_module(zircon_runtime::foundation::module_descriptor())?;
        asset_runtime.register_module(TasksModule.descriptor())?;
        asset_runtime.register_module(zircon_runtime::asset::module_descriptor())?;
        asset_runtime.activate_module(zircon_runtime::foundation::FOUNDATION_MODULE_NAME)?;
        asset_runtime.activate_module(TASKS_MODULE_NAME)?;
        asset_runtime.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)?;
        let core = asset_runtime.handle();
        let asset_access =
            ProjectAssetManagerAccess::new(core.clone(), project_asset_manager_handle(&core)?);
        let runtime_bootstrap_elapsed = runtime_bootstrap_started.elapsed();

        let project_open_started = Instant::now();
        let asset_manager = asset_access.resolve()?;
        asset_manager.open_project(project_root.as_path().to_string_lossy().as_ref())?;
        let project = asset_manager.current_project_manager().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "viewer asset manager did not retain its opened project",
            )
        })?;
        let project_open_elapsed = project_open_started.elapsed();

        let world_load_started = Instant::now();
        let world = zircon_runtime::scene::world::World::load_scene_from_uri(&project, &scene_uri)?;
        let world_load_elapsed = world_load_started.elapsed();

        let renderer_init_started = Instant::now();
        let (renderer, renderer_startup_report) =
            SceneRenderer::new_with_startup_options_and_report(
                asset_access,
                SceneRendererStartupOptions::environment_only_pbr_preview(),
            )?;
        let renderer_init_elapsed = renderer_init_started.elapsed();

        let ibl_restore_started = Instant::now();
        let ibl_cache_root = resolved_ibl_cache_root(ibl_cache_dir);
        let SourceCubemapEnvironmentLoad {
            environment,
            staging_status,
            staging_elapsed,
            total_elapsed,
        } = source_cubemap_environment(
            decoded_hdri,
            &ibl_cache_root,
            asset_runtime.task_pools().compute(),
        )?;
        let ibl_restore_elapsed = ibl_restore_started.elapsed();
        let ibl_load_report =
            PbrMirrorSceneIblLoadReport::new(staging_status, staging_elapsed, total_elapsed);
        let environment = EnvironmentExtract::source_cubemap(environment);
        let preview = PreviewEnvironmentExtract::from_environment(&environment, true, Vec4::ZERO);
        let core_startup = renderer_startup_report.core_startup();
        let environment_only_pbr_base_prewarm =
            renderer_startup_report.environment_only_pbr_base_prewarm();
        let scene = Self {
            _project_root: project_root,
            world: Some(world),
            viewport_surface: None,
            renderer: Some(renderer),
            environment,
            preview,
            ibl_load_report,
            frame_timing_report_requested: false,
            last_frame_timing: PbrMirrorSceneFrameTimingReport::default(),
            _asset_runtime: Some(asset_runtime),
        };
        println!(
            "PBR viewer startup timing: hdr_decode={hdri_decode_elapsed:.2?}, project_assets={project_assets_elapsed:.2?} ({project_assets_cache}), runtime_bootstrap={runtime_bootstrap_elapsed:.2?}, project_open={project_open_elapsed:.2?}, world_load={world_load_elapsed:.2?}, renderer_init={renderer_init_elapsed:.2?} (backend={:.2?}, core={:.2?} [setup={:.2?}, mesh_environment={:.2?}, shadows={:.2?}, deferred={:.2?} [lighting_pipelines={:.2?} [lighting_source_assembly={:.2?}, pipeline_foundation={:.2?}, standard_pso={:.2?}], fallback_resources={:.2?}], scene_effects={:.2?} [particles={:.2?}, sprites={:.2?}, hzb={:.2?}, post_process={:.2?}], overlay_ui={:.2?}], streamer={:.2?}, base_prewarm={environment_only_pbr_base_prewarm:?}), ibl_restore={ibl_restore_elapsed:.2?}, total={:.2?}",
            renderer_startup_report.backend_initialization(),
            renderer_startup_report.core_initialization(),
            core_startup.setup(),
            core_startup.mesh_and_environment(),
            core_startup.shadows(),
            core_startup.deferred(),
            core_startup.deferred_lighting_pipelines(),
            core_startup.deferred_lighting_shader_source_assembly(),
            core_startup.deferred_lighting_pipeline_foundation(),
            core_startup.deferred_lighting_standard_pipeline(),
            core_startup.deferred_fallback_resources(),
            core_startup.scene_effects(),
            core_startup.scene_effects_particles(),
            core_startup.scene_effects_sprites(),
            core_startup.scene_effects_hzb(),
            core_startup.scene_effects_post_process(),
            core_startup.overlay_and_ui(),
            renderer_startup_report.resource_streamer_initialization(),
            scene_load_started.elapsed(),
            project_assets_cache = if project_assets_reused {
                "reused"
            } else {
                "written"
            },
        );

        Ok(scene)
    }

    pub(crate) fn render(
        &mut self,
        camera: &OrbitCamera,
        viewport_size: UVec2,
    ) -> Result<ViewportFrame, Box<dyn Error>> {
        let capture_frame_timing = self.frame_timing_report_requested;
        let render_extract_started = capture_frame_timing.then(Instant::now);
        let snapshot = self.render_snapshot(camera, viewport_size)?;
        let render_extract =
            render_extract_started.map_or(std::time::Duration::ZERO, |started| started.elapsed());

        let renderer = self
            .renderer
            .as_mut()
            .ok_or("PBR mirror scene renderer has already shut down")?;
        let frame = renderer.render(snapshot, viewport_size)?;
        if capture_frame_timing {
            let renderer_timing = renderer.last_frame_timing_report();
            self.frame_timing_report_requested = false;
            self.last_frame_timing = PbrMirrorSceneFrameTimingReport::new(
                render_extract,
                renderer_timing.render_submission(),
                renderer_timing.readback_and_completion(),
            );
        }
        Ok(frame)
    }

    pub(crate) fn attach_viewport_surface(
        &mut self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<(), Box<dyn Error>> {
        let renderer = self
            .renderer
            .as_ref()
            .ok_or("PBR mirror scene renderer has already shut down")?;
        self.viewport_surface = Some(renderer.create_viewport_surface(descriptor)?);
        Ok(())
    }

    pub(crate) fn render_to_viewport_surface(
        &mut self,
        camera: &OrbitCamera,
        viewport_size: UVec2,
    ) -> Result<(), Box<dyn Error>> {
        let capture_frame_timing = self.frame_timing_report_requested;
        let render_extract_started = capture_frame_timing.then(Instant::now);
        let snapshot = self.render_snapshot(camera, viewport_size)?;
        let render_extract =
            render_extract_started.map_or(std::time::Duration::ZERO, |started| started.elapsed());
        let viewport_surface = self
            .viewport_surface
            .as_mut()
            .ok_or("PBR mirror scene does not have a native viewport surface")?;
        let renderer = self
            .renderer
            .as_mut()
            .ok_or("PBR mirror scene renderer has already shut down")?;
        renderer.render_to_viewport_surface(snapshot, viewport_size, viewport_surface)?;
        if capture_frame_timing {
            let renderer_timing = renderer.last_frame_timing_report();
            self.frame_timing_report_requested = false;
            self.last_frame_timing = PbrMirrorSceneFrameTimingReport::new(
                render_extract,
                renderer_timing.render_submission(),
                renderer_timing.readback_and_completion(),
            );
        }
        Ok(())
    }

    pub(crate) fn detach_viewport_surface(&mut self) {
        self.viewport_surface.take();
    }

    fn render_snapshot(
        &self,
        camera: &OrbitCamera,
        viewport_size: UVec2,
    ) -> Result<RenderSceneSnapshot, Box<dyn Error>> {
        let camera_descriptor = camera_render_descriptor(camera, viewport_size);
        let world = self
            .world
            .as_ref()
            .ok_or("PBR mirror scene has already shut down")?;
        let mut snapshot = world.build_viewport_render_packet(&SceneViewportExtractRequest {
            settings: ViewportRenderSettings::default(),
            active_camera_override: None,
            camera: Some(camera_descriptor),
            viewport_size: Some(viewport_size),
            virtual_geometry_debug: None,
        });
        snapshot.environment = self.environment.clone();
        snapshot.preview = self.preview.clone();
        snapshot.overlays = RenderOverlayExtract::default();
        Ok(snapshot)
    }

    pub(crate) fn renderer_backend_name(&self) -> &str {
        self.renderer
            .as_ref()
            .expect("PBR mirror scene renderer must exist while the viewer is active")
            .backend_name()
    }

    pub(crate) const fn ibl_load_report(&self) -> PbrMirrorSceneIblLoadReport {
        self.ibl_load_report
    }

    pub(crate) fn request_next_frame_timing_report(&mut self) {
        self.frame_timing_report_requested = true;
        self.renderer
            .as_mut()
            .expect("PBR mirror scene renderer must exist while the viewer is active")
            .request_next_frame_timing_report();
    }

    pub(crate) const fn last_frame_timing_report(&self) -> PbrMirrorSceneFrameTimingReport {
        self.last_frame_timing
    }

    pub(crate) fn start_graphics_debugger_capture(&self) {
        self.renderer
            .as_ref()
            .expect("PBR mirror scene renderer must exist while the viewer is active")
            .start_graphics_debugger_capture();
    }

    pub(crate) fn stop_graphics_debugger_capture(&self) -> Result<(), Box<dyn Error>> {
        Ok(self
            .renderer
            .as_ref()
            .expect("PBR mirror scene renderer must exist while the viewer is active")
            .stop_graphics_debugger_capture()?)
    }
}

impl Drop for PbrMirrorScene {
    fn drop(&mut self) {
        // Windows file watchers can retain the temporary asset root until their runtime owner drops.
        self.world.take();
        self.viewport_surface.take();
        self.renderer.take();
        self._asset_runtime.take();
    }
}

struct CachedProjectRoot {
    path: PathBuf,
}

impl CachedProjectRoot {
    fn new() -> Self {
        Self {
            path: std::env::temp_dir().join(format!(
                "zircon_shader_pbr_viewer_project_v{VIEWER_PROJECT_CACHE_VERSION}"
            )),
        }
    }

    fn as_path(&self) -> &Path {
        &self.path
    }
}

fn persistent_ibl_cache_root() -> PathBuf {
    std::env::temp_dir().join("zircon_shader_pbr_viewer_ibl_cache")
}

fn resolved_ibl_cache_root(override_root: Option<&Path>) -> PathBuf {
    override_root
        .map(Path::to_path_buf)
        .unwrap_or_else(persistent_ibl_cache_root)
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("scene.rs");

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
    fn scene_renderer_drops_before_its_runtime_services() {
        let fields = production_source()
            .split_once("pub(crate) struct PbrMirrorScene {")
            .and_then(|(_, rest)| rest.split_once('}'))
            .map(|(fields, _)| fields)
            .expect("PbrMirrorScene fields should remain visible to the lifecycle guard");

        assert!(
            fields.find("renderer:").expect("renderer field")
                < fields.find("_asset_runtime:").expect("runtime owner field"),
            "Rust drops struct fields in declaration order, so the runtime owner must follow the renderer"
        );
    }

    #[test]
    fn scene_teardown_releases_world_renderer_and_runtime_in_order() {
        assert_source_order(&[
            "self.world.take();",
            "self.viewport_surface.take();",
            "self.renderer.take();",
            "self._asset_runtime.take();",
        ]);
        assert!(
            !production_source().contains("impl Drop for CachedProjectRoot"),
            "the reusable project cache must outlive a single viewer process"
        );
    }

    #[test]
    fn native_viewport_surface_is_bound_after_background_loading_and_drops_before_renderer() {
        let fields = production_source()
            .split_once("pub(crate) struct PbrMirrorScene {")
            .and_then(|(_, rest)| rest.split_once('}'))
            .map(|(fields, _)| fields)
            .expect("PbrMirrorScene fields should remain visible to the lifecycle guard");

        assert!(
            fields
                .find("viewport_surface:")
                .expect("viewport surface field")
                < fields.find("renderer:").expect("renderer field"),
            "the native surface must drop before its renderer/device owner"
        );
        assert_source_order(&[
            "pub(crate) fn attach_viewport_surface(",
            "renderer.create_viewport_surface(descriptor)?",
            "pub(crate) fn render_to_viewport_surface(",
            "renderer.render_to_viewport_surface(snapshot, viewport_size, viewport_surface)?",
            "pub(crate) fn detach_viewport_surface(",
            "self.viewport_surface.take();",
        ]);
    }

    #[test]
    fn viewer_project_cache_root_is_stable_and_versioned() {
        let root = super::CachedProjectRoot::new();
        assert!(
            root.as_path()
                .ends_with("zircon_shader_pbr_viewer_project_v2"),
            "the viewer cache path must change when its generated asset schema changes"
        );
    }

    #[test]
    fn viewer_ibl_cache_is_not_nested_under_the_project_asset_cache() {
        let cache_root = super::persistent_ibl_cache_root();
        assert!(cache_root.ends_with("zircon_shader_pbr_viewer_ibl_cache"));
        assert_source_order(&[
            "let ibl_cache_root = resolved_ibl_cache_root(ibl_cache_dir);",
            "&ibl_cache_root,",
        ]);
        assert!(
            !production_source().contains("paths.cache_root(),"),
            "viewer staging must not use the viewer project asset cache"
        );
    }

    #[test]
    fn viewer_cache_override_supports_reproducible_cold_and_warm_cache_runs() {
        assert_eq!(
            super::resolved_ibl_cache_root(Some(std::path::Path::new("E:/ViewerIblCache"))),
            std::path::PathBuf::from("E:/ViewerIblCache")
        );
        assert_eq!(
            super::resolved_ibl_cache_root(None),
            super::persistent_ibl_cache_root()
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
    fn viewer_exposes_first_frame_extract_renderer_call_and_readback_timing() {
        let source = production_source();
        assert_source_order(&[
            "let capture_frame_timing = self.frame_timing_report_requested;",
            "let render_extract_started = capture_frame_timing.then(Instant::now);",
            "let render_extract =\n            render_extract_started",
            "let frame = renderer.render(snapshot, viewport_size)?;",
            "if capture_frame_timing {",
            "let renderer_timing = renderer.last_frame_timing_report();",
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
            "let frame = renderer.render(snapshot, viewport_size)?;",
        ]);
        assert_eq!(
            source.matches("renderer.render(").count(),
            1,
            "the viewer must not issue and discard CPU ViewportFrame readbacks for warmup"
        );
        assert!(
            !source.contains("warm_up_first_frame"),
            "pipeline warmup requires the Render17-owned no-readback API"
        );
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
            "base_prewarm={environment_only_pbr_base_prewarm:?}",
            "core_startup.deferred_lighting_shader_source_assembly()",
            "core_startup.deferred_lighting_pipeline_foundation()",
            "core_startup.deferred_lighting_standard_pipeline()",
            "renderer_startup_report.environment_only_pbr_base_prewarm()",
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
            "let decoded_hdri = decode_viewer_hdri(hdri_path, face_size, pmrem_face_size)?;",
            "let project_root = CachedProjectRoot::new();",
            "register_module(zircon_runtime::foundation::module_descriptor())",
            "register_module(TasksModule.descriptor())",
            "register_module(zircon_runtime::asset::module_descriptor())",
            "activate_module(zircon_runtime::foundation::FOUNDATION_MODULE_NAME)",
            "activate_module(TASKS_MODULE_NAME)",
            "activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)",
            "ProjectAssetManagerAccess::new",
            "asset_manager.open_project(project_root.as_path().to_string_lossy().as_ref())",
            "asset_manager.current_project_manager()",
            "SceneRenderer::new_with_startup_options_and_report(",
            "SceneRendererStartupOptions::environment_only_pbr_preview()",
            "asset_runtime.task_pools().compute()",
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
    fn viewer_reuses_completed_project_assets_before_opening_the_runtime_project() {
        assert_source_order(&[
            "let asset_root = manifest.primary_asset_root_path(&paths)?;",
            "let project_assets_reused = viewer_project_assets_are_ready(&asset_root);",
            "if !project_assets_reused {",
            "write_viewer_project_assets(&asset_root)?;",
            "asset_manager.open_project(project_root.as_path().to_string_lossy().as_ref())",
        ]);
    }
}
