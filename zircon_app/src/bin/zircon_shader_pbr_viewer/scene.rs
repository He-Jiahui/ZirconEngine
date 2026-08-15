use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::Instant;

use zircon_runtime::asset::importer::{
    EnvironmentIblSourceStagingOutput, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming,
};
use zircon_runtime::asset::pipeline::manager::{
    project_asset_manager_handle, AssetManager, ProjectAssetManagerAccess,
};
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetUri, ProjectInfo};
use zircon_runtime::core::framework::render::{
    EnvironmentExtract, PreviewEnvironmentExtract, RenderGpuTimingStatus, RenderOverlayExtract,
    RenderSceneSnapshot, RenderViewportSurfaceDescriptor, SceneViewportExtractRequest,
    ShaderVariantMissReport, ViewportRenderSettings,
};
use zircon_runtime::core::math::{UVec2, Vec4};
use zircon_runtime::core::runtime::modules::{TasksModule, TASKS_MODULE_NAME};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::graphics::{
    SceneRenderer, SceneRendererGpuTimingReport, SceneRendererStartupOptions, SceneViewportSurface,
    ViewportFrame,
};
use zircon_runtime_interface::project::RelPath;

use crate::camera::{camera_render_descriptor, OrbitCamera};
use crate::hdri::{
    preflight_viewer_hdri, source_cubemap_environment, SourceCubemapEnvironmentLoad,
};
use crate::project_assets::{
    viewer_project_assets_are_ready, write_viewer_project_assets,
    ViewerProjectAssetGenerationReport, VIEWER_PROJECT_ASSET_ROOT,
};
use crate::work_paths::ViewerWorkPaths;

#[derive(Default)]
struct ViewerProjectRuntimeOpenReport {
    project_opens: u32,
    imported_assets: usize,
    ready_assets: usize,
}

impl ViewerProjectRuntimeOpenReport {
    fn record_open(&mut self, info: &ProjectInfo) {
        self.project_opens += 1;
        self.imported_assets = info.asset_count;
        self.ready_assets = info.ready_asset_count;
    }

    const fn project_opens(&self) -> u32 {
        self.project_opens
    }

    const fn imported_assets(&self) -> usize {
        self.imported_assets
    }

    const fn ready_assets(&self) -> usize {
        self.ready_assets
    }
}

pub(crate) struct PbrMirrorScene {
    _work_paths: ViewerWorkPaths,
    world: Option<zircon_runtime::scene::world::World>,
    // The native surface must drop before its renderer/device owner.
    viewport_surface: Option<SceneViewportSurface>,
    renderer: Option<SceneRenderer>,
    environment: EnvironmentExtract,
    preview: PreviewEnvironmentExtract,
    ibl_load_report: PbrMirrorSceneIblLoadReport,
    base_prewarm_report: PbrMirrorSceneBasePrewarmReport,
    startup_timing: PbrMirrorSceneStartupTiming,
    frame_timing_report_requested: bool,
    last_frame_timing: PbrMirrorSceneFrameTimingReport,
    // Keep the cache root alive until runtime teardown releases its file watchers.
    _asset_runtime: Option<CoreRuntime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PbrMirrorSceneIblLoadReport {
    staging_status: EnvironmentIblSourceStagingStatus,
    staging_elapsed: std::time::Duration,
    staging_timing: EnvironmentIblSourceStagingTiming,
    staging_output: EnvironmentIblSourceStagingOutput,
    total_elapsed: std::time::Duration,
    source_cubemap_face_size: u32,
    source_cubemap_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
}

impl PbrMirrorSceneIblLoadReport {
    pub(crate) const fn new(
        staging_status: EnvironmentIblSourceStagingStatus,
        staging_elapsed: std::time::Duration,
        staging_timing: EnvironmentIblSourceStagingTiming,
        staging_output: EnvironmentIblSourceStagingOutput,
        total_elapsed: std::time::Duration,
        source_cubemap_face_size: u32,
        source_cubemap_mip_count: u32,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
    ) -> Self {
        Self {
            staging_status,
            staging_elapsed,
            staging_timing,
            staging_output,
            total_elapsed,
            source_cubemap_face_size,
            source_cubemap_mip_count,
            pmrem_face_size,
            pmrem_mip_count,
        }
    }

    pub(crate) const fn staging_status(self) -> EnvironmentIblSourceStagingStatus {
        self.staging_status
    }

    pub(crate) const fn staging_elapsed(self) -> std::time::Duration {
        self.staging_elapsed
    }

    pub(crate) const fn staging_timing(self) -> EnvironmentIblSourceStagingTiming {
        self.staging_timing
    }

    pub(crate) const fn staging_output(self) -> EnvironmentIblSourceStagingOutput {
        self.staging_output
    }

    pub(crate) const fn total_elapsed(self) -> std::time::Duration {
        self.total_elapsed
    }

    pub(crate) const fn source_cubemap_face_size(self) -> u32 {
        self.source_cubemap_face_size
    }

    pub(crate) const fn source_cubemap_mip_count(self) -> u32 {
        self.source_cubemap_mip_count
    }

    pub(crate) const fn pmrem_face_size(self) -> u32 {
        self.pmrem_face_size
    }

    pub(crate) const fn pmrem_mip_count(self) -> u32 {
        self.pmrem_mip_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PbrMirrorSceneBasePrewarmReport {
    pipeline_ready: bool,
    cache_hit: bool,
    shader_source_resolution: std::time::Duration,
    pipeline_creation: std::time::Duration,
    elapsed: std::time::Duration,
}

impl PbrMirrorSceneBasePrewarmReport {
    pub(crate) const fn pipeline_ready(self) -> bool {
        self.pipeline_ready
    }

    pub(crate) const fn cache_hit(self) -> bool {
        self.cache_hit
    }

    pub(crate) const fn shader_source_resolution(self) -> std::time::Duration {
        self.shader_source_resolution
    }

    pub(crate) const fn pipeline_creation(self) -> std::time::Duration {
        self.pipeline_creation
    }

    pub(crate) const fn elapsed(self) -> std::time::Duration {
        self.elapsed
    }
}

/// Startup timings retained with the scene so captured images carry the same
/// attribution as the viewer's startup log.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PbrMirrorSceneStartupTiming {
    hdri_decode: std::time::Duration,
    project_assets: std::time::Duration,
    runtime_bootstrap: std::time::Duration,
    project_open: std::time::Duration,
    world_load: std::time::Duration,
    renderer_initialization: std::time::Duration,
    renderer_backend_initialization: std::time::Duration,
    renderer_deferred_initialization: std::time::Duration,
    renderer_deferred_standard_pipeline: std::time::Duration,
    resource_streamer_initialization: std::time::Duration,
    ibl_restore: std::time::Duration,
    total: std::time::Duration,
}

impl PbrMirrorSceneStartupTiming {
    pub(crate) const fn hdri_decode(self) -> std::time::Duration {
        self.hdri_decode
    }

    pub(crate) const fn project_assets(self) -> std::time::Duration {
        self.project_assets
    }

    pub(crate) const fn runtime_bootstrap(self) -> std::time::Duration {
        self.runtime_bootstrap
    }

    pub(crate) const fn project_open(self) -> std::time::Duration {
        self.project_open
    }

    pub(crate) const fn world_load(self) -> std::time::Duration {
        self.world_load
    }

    pub(crate) const fn renderer_initialization(self) -> std::time::Duration {
        self.renderer_initialization
    }

    pub(crate) const fn renderer_backend_initialization(self) -> std::time::Duration {
        self.renderer_backend_initialization
    }

    pub(crate) const fn renderer_deferred_initialization(self) -> std::time::Duration {
        self.renderer_deferred_initialization
    }

    pub(crate) const fn renderer_deferred_standard_pipeline(self) -> std::time::Duration {
        self.renderer_deferred_standard_pipeline
    }

    pub(crate) const fn resource_streamer_initialization(self) -> std::time::Duration {
        self.resource_streamer_initialization
    }

    pub(crate) const fn ibl_restore(self) -> std::time::Duration {
        self.ibl_restore
    }

    pub(crate) const fn total(self) -> std::time::Duration {
        self.total
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
        work_dir: &Path,
        ibl_cache_dir: Option<&Path>,
        gpu_timing_enabled: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let scene_load_started = Instant::now();
        let hdri_preflight_started = Instant::now();
        let hdri = preflight_viewer_hdri(hdri_path, face_size, pmrem_face_size)?;
        let hdri_preflight_elapsed = hdri_preflight_started.elapsed();
        let project_assets_started = Instant::now();
        let work_paths = ViewerWorkPaths::new(work_dir, ibl_cache_dir);
        fs::create_dir_all(work_paths.project_root())?;
        let paths = ProjectPaths::from_root(work_paths.project_root())?;
        let scene_uri = AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml")?;
        let mut manifest = ProjectManifest::new("ShaderPbrMirrorViewer", scene_uri.clone(), 1);
        manifest.asset_roots = vec![RelPath::parse(VIEWER_PROJECT_ASSET_ROOT)?];
        let manifest_needs_publish = !paths.manifest_path().is_file();
        if manifest_needs_publish {
            manifest.save(paths.manifest_path())?;
        }
        let asset_root = manifest.primary_asset_root_path(&paths)?;
        let project_assets_reused = viewer_project_assets_are_ready(&asset_root);
        let project_asset_generation = if project_assets_reused {
            ViewerProjectAssetGenerationReport::reused()
        } else {
            write_viewer_project_assets(&asset_root)?
        };
        paths.ensure_layout(&manifest.asset_roots)?;
        let project_manifest_writes = u32::from(manifest_needs_publish);
        let project_startup_filesystem_writes =
            project_asset_generation.filesystem_writes() + project_manifest_writes;
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
        let mut project_runtime_report = ViewerProjectRuntimeOpenReport::default();
        let project_info =
            asset_manager.open_project(work_paths.project_root().to_string_lossy().as_ref())?;
        project_runtime_report.record_open(&project_info);
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
        let startup_options = SceneRendererStartupOptions::environment_only_pbr_preview()
            .with_async_pipeline_compile();
        let startup_options = if gpu_timing_enabled {
            startup_options.with_gpu_timing()
        } else {
            startup_options
        };
        let (renderer, renderer_startup_report) =
            SceneRenderer::new_with_startup_options_and_report(asset_access, startup_options)?;
        let renderer_init_elapsed = renderer_init_started.elapsed();

        let ibl_restore_started = Instant::now();
        let SourceCubemapEnvironmentLoad {
            environment,
            staging_status,
            staging_elapsed,
            staging_timing,
            staging_output,
            total_elapsed,
            source_pixel_decode_elapsed,
            source_cubemap_face_size,
            source_cubemap_mip_count,
            pmrem_face_size,
            pmrem_mip_count,
        } = source_cubemap_environment(
            hdri,
            work_paths.ibl_cache_root(),
            asset_runtime.task_pools().compute(),
        )?;
        let ibl_restore_elapsed = ibl_restore_started
            .elapsed()
            .saturating_sub(source_pixel_decode_elapsed);
        let hdri_decode_elapsed =
            hdri_preflight_elapsed.saturating_add(source_pixel_decode_elapsed);
        let ibl_load_report = PbrMirrorSceneIblLoadReport::new(
            staging_status,
            staging_elapsed,
            staging_timing,
            staging_output,
            total_elapsed,
            source_cubemap_face_size,
            source_cubemap_mip_count,
            pmrem_face_size,
            pmrem_mip_count,
        );
        let environment = EnvironmentExtract::source_cubemap(environment);
        let preview = PreviewEnvironmentExtract::from_environment(&environment, true, Vec4::ZERO);
        let core_startup = renderer_startup_report.core_startup();
        let base_prewarm_report = renderer_startup_report
            .environment_only_pbr_base_prewarm()
            .map(|report| PbrMirrorSceneBasePrewarmReport {
                pipeline_ready: report.pipeline_ready(),
                cache_hit: report.cache_hit(),
                shader_source_resolution: report.shader_source_resolution(),
                pipeline_creation: report.pipeline_creation(),
                elapsed: report.elapsed(),
            })
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "environment-only PBR viewer did not retain its Base prewarm report",
                )
            })?;
        let startup_timing = PbrMirrorSceneStartupTiming {
            hdri_decode: hdri_decode_elapsed,
            project_assets: project_assets_elapsed,
            runtime_bootstrap: runtime_bootstrap_elapsed,
            project_open: project_open_elapsed,
            world_load: world_load_elapsed,
            renderer_initialization: renderer_init_elapsed,
            renderer_backend_initialization: renderer_startup_report.backend_initialization(),
            renderer_deferred_initialization: core_startup.deferred(),
            renderer_deferred_standard_pipeline: core_startup.deferred_lighting_standard_pipeline(),
            resource_streamer_initialization: renderer_startup_report
                .resource_streamer_initialization(),
            ibl_restore: ibl_restore_elapsed,
            total: scene_load_started.elapsed(),
        };
        let scene = Self {
            _work_paths: work_paths,
            world: Some(world),
            viewport_surface: None,
            renderer: Some(renderer),
            environment,
            preview,
            ibl_load_report,
            base_prewarm_report,
            startup_timing,
            frame_timing_report_requested: false,
            last_frame_timing: PbrMirrorSceneFrameTimingReport::default(),
            _asset_runtime: Some(asset_runtime),
        };
        println!(
            "PBR viewer startup timing: hdr_decode={hdri_decode_elapsed:.2?}, project_assets={project_assets_elapsed:.2?} ({project_assets_cache}; mesh_generation_samples={}; serialized_source_bytes={}; asset_filesystem_writes={}; project_manifest_writes={}; startup_filesystem_writes={}), runtime_bootstrap={runtime_bootstrap_elapsed:.2?}, project_open={project_open_elapsed:.2?} (project_open_count={}; imported_assets={}; ready_assets={}), world_load={world_load_elapsed:.2?}, renderer_init={renderer_init_elapsed:.2?} (backend={:.2?}, core={:.2?} [setup={:.2?}, mesh_environment={:.2?}, shadows={:.2?}, deferred={:.2?} [lighting_pipelines={:.2?} [lighting_source_assembly={:.2?}, pipeline_foundation={:.2?}, standard_pso={:.2?}], fallback_resources={:.2?}], scene_effects={:.2?} [particles={:.2?}, sprites={:.2?}, hzb={:.2?}, post_process={:.2?}], overlay_ui={:.2?}], streamer={:.2?}, base_prewarm={base_prewarm_report:?}), ibl_restore={ibl_restore_elapsed:.2?}, total={:.2?}",
            project_asset_generation.mesh_generation_samples(),
            project_asset_generation.serialized_source_bytes(),
            project_asset_generation.filesystem_writes(),
            project_manifest_writes,
            project_startup_filesystem_writes,
            project_runtime_report.project_opens(),
            project_runtime_report.imported_assets(),
            project_runtime_report.ready_assets(),
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
            startup_timing.total(),
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

    pub(crate) fn shader_variant_miss_report(&self) -> ShaderVariantMissReport {
        self.renderer
            .as_ref()
            .expect("PBR mirror scene renderer must exist while the viewer is active")
            .last_shader_variant_miss_report()
    }

    pub(crate) fn take_completed_gpu_timing_report(
        &mut self,
    ) -> Option<SceneRendererGpuTimingReport> {
        self.renderer
            .as_mut()
            .expect("PBR mirror scene renderer must exist while the viewer is active")
            .take_completed_gpu_timing_report()
    }

    pub(crate) fn last_gpu_timing_status(&self) -> RenderGpuTimingStatus {
        self.renderer
            .as_ref()
            .expect("PBR mirror scene renderer must exist while the viewer is active")
            .last_gpu_timing_status()
    }

    pub(crate) const fn ibl_load_report(&self) -> PbrMirrorSceneIblLoadReport {
        self.ibl_load_report
    }

    pub(crate) const fn base_prewarm_report(&self) -> PbrMirrorSceneBasePrewarmReport {
        self.base_prewarm_report
    }

    pub(crate) const fn startup_timing(&self) -> PbrMirrorSceneStartupTiming {
        self.startup_timing
    }

    /// Drains completed Base-PSO work without blocking and reports whether a
    /// one-shot screenshot or graphics capture can contain the PBR mesh.
    pub(crate) fn environment_only_base_pipeline_ready(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .renderer
            .as_mut()
            .ok_or("PBR mirror scene renderer has already shut down")?
            .environment_only_pbr_base_pipeline_ready()?)
    }

    /// Retries nonblocking Base-PSO admission after the bounded worker frees capacity.
    pub(crate) fn retry_environment_only_base_pipeline_admission(
        &mut self,
    ) -> Result<(), Box<dyn Error>> {
        self.renderer
            .as_mut()
            .ok_or("PBR mirror scene renderer has already shut down")?
            .retry_environment_only_pbr_base_pipeline_admission()?;
        Ok(())
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
            !production_source().contains("impl Drop for ViewerWorkPaths"),
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
            "pub(crate) fn environment_only_base_pipeline_ready(",
            "pub(crate) fn retry_environment_only_base_pipeline_admission(",
            ".retry_environment_only_pbr_base_pipeline_admission()?;",
        ]);
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
            "renderer_startup_report.environment_only_pbr_base_prewarm()",
            "base_prewarm_report.cache_hit()",
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
            "SceneRenderer::new_with_startup_options_and_report(asset_access, startup_options)?;",
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
            "let project_asset_generation = if project_assets_reused {",
            "ViewerProjectAssetGenerationReport::reused()",
            "write_viewer_project_assets(&asset_root)?",
            "asset_manager.open_project(work_paths.project_root().to_string_lossy().as_ref())",
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
}
