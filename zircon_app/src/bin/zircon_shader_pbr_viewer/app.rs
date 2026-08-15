use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size};
use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window, WindowAttributes};
use zircon_runtime::core::framework::render::{
    RenderViewportSurfaceDescriptor, IBL_BAKE_ALGORITHM_VERSION,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::{SceneRendererGpuTimingReport, ViewportFrame};
use zircon_runtime::rhi::RenderNativeSurfaceTarget;

use crate::args::ViewerConfig;
use crate::background_load::{BackgroundTask, BackgroundTaskPoll};
use crate::camera::OrbitCamera;
use crate::frame_io::{
    error_frame, startup_frame, write_ready_frame_evidence, ReadyFrameEvidenceMetadata,
};
use crate::gpu_timing_evidence::{
    format_gpu_timing_evidence, gpu_timing_report_parent, validate_gpu_timing_report_output,
    GpuTimingEvidenceRequest, GpuTimingEvidenceResolution,
};
use crate::presenter::{window_size, SoftbufferViewportPresenter};
use crate::renderdoc::RenderDocBridge;
use crate::scene::{PbrMirrorScene, PbrMirrorSceneIblLoadReport};

#[path = "base_pipeline_recheck.rs"]
mod base_pipeline_recheck;

use base_pipeline_recheck::{
    base_pipeline_recheck_deadline_with_cap, base_pipeline_recheck_is_due,
    one_shot_base_pipeline_wait_deadline, one_shot_base_pipeline_wait_is_expired,
    ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT,
};

const DEFAULT_WINDOW_WIDTH: u32 = 1280;
const DEFAULT_WINDOW_HEIGHT: u32 = 960;
const MIN_WINDOW_WIDTH: f64 = 480.0;
const MIN_WINDOW_HEIGHT: f64 = 360.0;
const LOAD_STATUS_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const PBR_VIEWER_RENDER_PROFILE: &str = "environment_only_pbr_preview";

pub(crate) struct PbrMirrorViewerApp {
    hdri_path: PathBuf,
    // Preserve automatic sizing until the background loader can inspect the HDR image.
    face_size: Option<u32>,
    pmrem_face_size: Option<u32>,
    work_dir: PathBuf,
    ibl_cache_dir: Option<PathBuf>,
    screenshot_path: Option<PathBuf>,
    screenshot_written: bool,
    gpu_timing_report_path: Option<PathBuf>,
    gpu_timing_request: Option<GpuTimingEvidenceRequest>,
    renderdoc_capture_once: bool,
    renderdoc_bridge: Option<RenderDocBridge>,
    exit_after_capture: bool,
    renderdoc_capture_finished: bool,
    scene: Option<PbrMirrorScene>,
    scene_loader: Option<BackgroundTask<PbrMirrorScene>>,
    scene_load_started_at: Option<Instant>,
    last_load_status_refresh_at: Option<Instant>,
    first_ready_scene_load_elapsed: Option<Duration>,
    first_ready_scene_load_started_at: Option<Instant>,
    first_ready_frame_started_at: Option<Instant>,
    load_error: Option<String>,
    event_loop_proxy: EventLoopProxy,
    camera: OrbitCamera,
    window: Option<Arc<dyn Window>>,
    presenter: Option<SoftbufferViewportPresenter>,
    direct_present_enabled: bool,
    size: UVec2,
    left_dragging: bool,
    last_pointer_position: Option<PhysicalPosition<f64>>,
    redraw_requested: bool,
    base_pipeline_recheck_at: Option<Instant>,
    base_pipeline_recheck_attempt: u32,
    one_shot_base_pipeline_wait_started_at: Option<Instant>,
    ready_title_dirty: bool,
}

impl PbrMirrorViewerApp {
    pub(crate) fn new(
        config: ViewerConfig,
        event_loop_proxy: EventLoopProxy,
        renderdoc_bridge: Option<RenderDocBridge>,
    ) -> Self {
        Self {
            hdri_path: config.hdri_path,
            face_size: config.face_size,
            pmrem_face_size: config.pmrem_face_size,
            work_dir: config.work_dir,
            ibl_cache_dir: config.ibl_cache_dir,
            screenshot_path: config.screenshot_path,
            screenshot_written: false,
            gpu_timing_report_path: config.gpu_timing_report_path,
            gpu_timing_request: None,
            renderdoc_capture_once: config.renderdoc_capture_once,
            renderdoc_bridge,
            exit_after_capture: config.exit_after_capture,
            renderdoc_capture_finished: false,
            scene: None,
            scene_loader: None,
            scene_load_started_at: None,
            last_load_status_refresh_at: None,
            first_ready_scene_load_elapsed: None,
            first_ready_scene_load_started_at: None,
            first_ready_frame_started_at: None,
            load_error: None,
            event_loop_proxy,
            camera: OrbitCamera::from_angles(
                config.initial_yaw_degrees,
                config.initial_pitch_degrees,
            ),
            window: None,
            presenter: None,
            direct_present_enabled: false,
            size: UVec2::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            left_dragging: false,
            last_pointer_position: None,
            redraw_requested: false,
            base_pipeline_recheck_at: None,
            base_pipeline_recheck_attempt: 0,
            one_shot_base_pipeline_wait_started_at: None,
            ready_title_dirty: false,
        }
    }

    fn ensure_window(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title(loading_window_title(Duration::ZERO))
            .with_surface_size(Size::Physical(PhysicalSize::new(
                DEFAULT_WINDOW_WIDTH,
                DEFAULT_WINDOW_HEIGHT,
            )))
            .with_min_surface_size(Size::Logical(LogicalSize::new(
                MIN_WINDOW_WIDTH,
                MIN_WINDOW_HEIGHT,
            )))
            .with_resizable(true);

        let window: Arc<dyn Window> = match event_loop.create_window(attributes) {
            Ok(window) => Arc::from(window),
            Err(error) => {
                eprintln!("failed to create viewer window: {error}");
                event_loop.exit();
                return;
            }
        };
        self.size = window_size(window.as_ref());
        self.window = Some(window);
        if let Err(error) = self.ensure_cpu_presenter() {
            eprintln!("failed to create viewer presenter: {error}");
            event_loop.exit();
            return;
        }
        self.present_startup_frame(event_loop);
        self.start_scene_load(event_loop);
    }

    fn ensure_cpu_presenter(&mut self) -> Result<(), String> {
        if self.presenter.is_some() {
            return Ok(());
        }

        let window = self
            .window
            .as_ref()
            .ok_or_else(|| "viewer window is not available".to_owned())?
            .clone();
        self.presenter =
            Some(SoftbufferViewportPresenter::new(window).map_err(|error| error.to_string())?);
        Ok(())
    }

    fn start_scene_load(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.scene.is_some() || self.scene_loader.is_some() || self.load_error.is_some() {
            return;
        }

        self.reset_base_pipeline_recheck();

        let hdri_path = self.hdri_path.clone();
        let face_size = self.face_size;
        let pmrem_face_size = self.pmrem_face_size;
        let work_dir = self.work_dir.clone();
        let ibl_cache_dir = self.ibl_cache_dir.clone();
        let gpu_timing_enabled = self.gpu_timing_report_path.is_some();
        let event_loop_proxy = self.event_loop_proxy.clone();
        let started_at = Instant::now();
        match BackgroundTask::spawn(
            "zircon-pbr-scene-loader",
            move || {
                PbrMirrorScene::new(
                    &hdri_path,
                    face_size,
                    pmrem_face_size,
                    &work_dir,
                    ibl_cache_dir.as_deref(),
                    gpu_timing_enabled,
                )
                .map_err(|error| error.to_string())
            },
            move || event_loop_proxy.wake_up(),
        ) {
            Ok(loader) => {
                self.scene_load_started_at = Some(started_at);
                // The initial loading title was set while creating the window. Wait a full
                // interval before touching the platform title again.
                self.last_load_status_refresh_at = Some(started_at);
                self.scene_loader = Some(loader);
            }
            Err(error) => {
                let message = error.to_string();
                self.handle_scene_load_failure(event_loop, message);
            }
        }
    }

    fn finish_scene_load(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(loader) = self.scene_loader.as_ref() else {
            return;
        };
        let result = match loader.try_take() {
            BackgroundTaskPoll::Pending => return,
            BackgroundTaskPoll::Completed(result) => result,
        };
        self.scene_loader = None;
        self.last_load_status_refresh_at = None;

        match result {
            Ok(scene) => {
                self.scene = Some(scene);
                if self.screenshot_path.is_none() {
                    match self.bind_scene_viewport_surface() {
                        Ok(()) => {
                            self.direct_present_enabled = true;
                            // The direct surface owns interactive presentation. Retain CPU
                            // staging only for screenshot export or an explicit fallback.
                            self.presenter = None;
                        }
                        Err(error) => {
                            eprintln!(
                                "native viewer surface unavailable; falling back to CPU presentation: {error}"
                            );
                        }
                    }
                }
                let scene_load_started_at = self.scene_load_started_at.take();
                self.first_ready_scene_load_elapsed =
                    scene_load_started_at.map(|started| started.elapsed());
                self.first_ready_scene_load_started_at = scene_load_started_at;
                self.ready_title_dirty = true;
                // A loader wake-up does not necessarily carry a redraw request. Force the first
                // ready frame here so the Ready title never leaves the startup checkerboard.
                self.first_ready_frame_started_at = Some(Instant::now());
                self.redraw_requested = true;
                self.render_and_present(event_loop);
            }
            Err(message) => self.handle_scene_load_failure(event_loop, message),
        }
    }

    fn refresh_scene_load_status(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(started_at) = self.scene_load_started_at else {
            return;
        };
        if self.scene_loader.is_none() {
            return;
        }

        let now = Instant::now();
        if load_status_refresh_is_due(self.last_load_status_refresh_at, now) {
            if let Some(window) = self.window.as_ref() {
                window.set_title(&loading_window_title(started_at.elapsed()));
            }
            self.last_load_status_refresh_at = Some(now);
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(load_status_refresh_deadline(
            self.last_load_status_refresh_at,
            now,
        )));
    }

    fn handle_scene_load_failure(&mut self, event_loop: &dyn ActiveEventLoop, message: String) {
        eprintln!("failed to load PBR HDRI viewer scene: {message}");
        self.load_error = Some(message);
        self.last_load_status_refresh_at = None;
        self.reset_base_pipeline_recheck();
        self.first_ready_scene_load_elapsed = None;
        self.first_ready_scene_load_started_at = None;
        self.first_ready_frame_started_at = None;
        if let Some(window) = self.window.as_ref() {
            window.set_title("Zircon PBR HDRI Mirror Viewer - load failed");
        }
        self.present_error_frame(event_loop);
        if one_shot_run_exits_after_load_failure(
            self.screenshot_path.is_some(),
            self.renderdoc_capture_once,
            self.exit_after_capture,
        ) {
            event_loop.exit();
        }
    }

    fn request_redraw(&mut self) {
        if request_redraw_transition(&mut self.redraw_requested) {
            let Some(window) = self.window.as_ref() else {
                return;
            };
            window.request_redraw();
        }
    }

    fn log_first_ready_frame_presented(&mut self) {
        let Some(first_ready_frame_started_at) = self.first_ready_frame_started_at.take() else {
            return;
        };
        let Some(scene_load_elapsed) = self.first_ready_scene_load_elapsed else {
            return;
        };
        let Some(scene_load_started_at) = self.first_ready_scene_load_started_at else {
            return;
        };
        let first_ready_frame_elapsed = first_ready_frame_started_at.elapsed();
        let viewer_ready_elapsed = scene_load_started_at.elapsed();
        println!(
            "PBR viewer readiness: scene_constructed={scene_load_elapsed:.2?}, first_frame_presented={first_ready_frame_elapsed:.2?}, time_to_ready={viewer_ready_elapsed:.2?}",
        );
    }

    fn reset_base_pipeline_recheck(&mut self) {
        self.base_pipeline_recheck_at = None;
        self.base_pipeline_recheck_attempt = 0;
        self.one_shot_base_pipeline_wait_started_at = None;
    }

    fn one_shot_base_pipeline_wait_has_expired(&mut self, now: Instant) -> bool {
        let started_at = *self
            .one_shot_base_pipeline_wait_started_at
            .get_or_insert(now);
        one_shot_base_pipeline_wait_is_expired(started_at, now)
    }

    fn one_shot_base_pipeline_wait_elapsed(&self, now: Instant) -> Duration {
        self.one_shot_base_pipeline_wait_started_at
            .map(|started_at| now.duration_since(started_at))
            .unwrap_or_default()
    }

    fn schedule_base_pipeline_recheck(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        deadline_cap: Option<Instant>,
    ) {
        let deadline = base_pipeline_recheck_deadline_with_cap(
            Instant::now(),
            self.base_pipeline_recheck_attempt,
            deadline_cap,
        );
        self.base_pipeline_recheck_attempt = self.base_pipeline_recheck_attempt.saturating_add(1);
        self.base_pipeline_recheck_at = Some(deadline);
        event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
    }

    fn request_base_pipeline_recheck_if_due(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(deadline) = self.base_pipeline_recheck_at else {
            return;
        };
        let now = Instant::now();
        if !base_pipeline_recheck_is_due(deadline, now) {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            return;
        }
        self.base_pipeline_recheck_at = None;
        self.request_redraw();
    }

    fn resize(&mut self, event_loop: &dyn ActiveEventLoop, size: PhysicalSize<u32>) {
        self.size = UVec2::new(size.width.max(1), size.height.max(1));
        if let Some(presenter) = self.presenter.as_mut() {
            if let Err(error) = presenter.resize(self.size) {
                eprintln!("failed to resize viewer presenter: {error}");
                event_loop.exit();
                return;
            }
        }
        self.request_redraw();
    }

    fn bind_scene_viewport_surface(&mut self) -> Result<(), String> {
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| "viewer window is not available".to_owned())?;
        let descriptor = viewport_surface_descriptor(window.as_ref(), self.size)?;
        self.scene
            .as_mut()
            .ok_or_else(|| "PBR scene is not available".to_owned())?
            .attach_viewport_surface(descriptor)
            .map_err(|error| error.to_string())
    }

    fn render_and_present(&mut self, event_loop: &dyn ActiveEventLoop) {
        if !self.redraw_requested {
            return;
        }
        self.redraw_requested = false;

        let screenshot_requested = self.screenshot_path.is_some() && !self.screenshot_written;
        let screenshot_camera = self.camera;
        let interactive_direct_present_enabled = self.direct_present_enabled;

        let capture_requested = self.renderdoc_capture_once && !self.renderdoc_capture_finished;
        let needs_environment_only_base_pipeline = screenshot_requested || capture_requested;
        let environment_only_base_pipeline_ready = {
            let Some(scene) = self.scene.as_mut() else {
                if self.load_error.is_some() {
                    self.present_error_frame(event_loop);
                } else {
                    self.present_startup_frame(event_loop);
                }
                return;
            };
            match scene.environment_only_base_pipeline_ready() {
                Ok(true) => true,
                Ok(false) => {
                    if let Err(error) = scene.retry_environment_only_base_pipeline_admission() {
                        eprintln!(
                            "environment-only PBR Base pipeline admission retry failed: {error}"
                        );
                        event_loop.exit();
                        return;
                    }
                    false
                }
                Err(error) => {
                    eprintln!("environment-only PBR Base pipeline startup failed: {error}");
                    event_loop.exit();
                    return;
                }
            }
        };
        let defer_one_shot_until_base_pipeline_ready =
            needs_environment_only_base_pipeline && !environment_only_base_pipeline_ready;
        let recheck_base_pipeline_after_present =
            !needs_environment_only_base_pipeline && !environment_only_base_pipeline_ready;
        let write_screenshot = screenshot_requested && environment_only_base_pipeline_ready;
        let one_shot_base_pipeline_wait_elapsed = write_screenshot
            .then(|| self.one_shot_base_pipeline_wait_elapsed(Instant::now()))
            .unwrap_or_default();
        let viewer_scene_load_elapsed = write_screenshot
            .then(|| {
                self.first_ready_scene_load_elapsed
                    .expect("Ready-frame evidence requires a completed viewer scene load")
            })
            .unwrap_or_default();
        let viewer_ready_started_at = write_screenshot.then(|| {
            self.first_ready_scene_load_started_at
                .expect("Ready-frame evidence requires the viewer load start time")
        });
        if defer_one_shot_until_base_pipeline_ready {
            if self.one_shot_base_pipeline_wait_has_expired(Instant::now()) {
                eprintln!(
                    "environment-only PBR Base pipeline startup timed out after {ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT:?}"
                );
                self.reset_base_pipeline_recheck();
                event_loop.exit();
                return;
            }
            let deadline = self
                .one_shot_base_pipeline_wait_started_at
                .map(one_shot_base_pipeline_wait_deadline)
                .expect("the one-shot timeout check must initialize its deadline");
            self.schedule_base_pipeline_recheck(event_loop, Some(deadline));
            return;
        }
        if environment_only_base_pipeline_ready {
            self.reset_base_pipeline_recheck();
        }
        let screenshot_input = write_screenshot.then(|| {
            (
                self.hdri_path.display().to_string(),
                self.face_size,
                self.pmrem_face_size,
            )
        });
        if !self.direct_present_enabled || screenshot_requested {
            if let Err(error) = self.ensure_cpu_presenter() {
                eprintln!("failed to create CPU presentation fallback: {error}");
                event_loop.exit();
                return;
            }
        }

        let scene = self
            .scene
            .as_mut()
            .expect("PBR scene must remain available after startup-state query");
        let capture_this_frame = capture_requested && environment_only_base_pipeline_ready;
        if capture_this_frame {
            println!(
                "starting graphics debugger capture on {}",
                scene.renderer_backend_name()
            );
            scene.start_graphics_debugger_capture();
        }
        if write_screenshot || capture_this_frame {
            scene.request_next_frame_timing_report();
        }
        if self.direct_present_enabled && !write_screenshot {
            let direct_present_started = capture_this_frame.then(Instant::now);
            match scene.render_to_viewport_surface(&self.camera, self.size) {
                Ok(()) => {
                    let gpu_timing_report = scene.take_completed_gpu_timing_report();
                    let gpu_timing_status = scene.last_gpu_timing_status();
                    if let Some(direct_present_started) = direct_present_started {
                        let scene_timing = scene.last_frame_timing_report();
                        println!(
                            "PBR viewer Direct-present timing: total={:.2?} [extract={:.2?}, render_and_present_call={:.2?}, readback_and_completion={:.2?}]",
                            direct_present_started.elapsed(),
                            scene_timing.render_extract(),
                            scene_timing.renderer_frame_call(),
                            scene_timing.readback_and_completion(),
                        );
                    }
                    if capture_this_frame {
                        if let Err(error) =
                            finish_graphics_debugger_capture(scene, self.renderdoc_bridge.as_ref())
                        {
                            eprintln!("failed to complete graphics debugger capture: {error}");
                            event_loop.exit();
                            return;
                        }
                        self.renderdoc_capture_finished = true;
                    }
                    self.flush_ready_window_title();
                    self.log_first_ready_frame_presented();
                    if recheck_base_pipeline_after_present {
                        self.schedule_base_pipeline_recheck(event_loop, None);
                    }
                    if let Err(error) =
                        self.resolve_gpu_timing_evidence(gpu_timing_report, gpu_timing_status)
                    {
                        eprintln!("failed to write PBR viewer GPU timing evidence: {error}");
                        event_loop.exit();
                        return;
                    }
                    if capture_this_frame
                        && self.exit_after_capture
                        && !self.gpu_timing_evidence_pending()
                    {
                        event_loop.exit();
                    } else if self.exit_after_screenshot() {
                        event_loop.exit();
                    }
                }
                Err(error) => {
                    if capture_this_frame {
                        if let Err(stop_error) = scene.stop_graphics_debugger_capture() {
                            eprintln!(
                                "failed to stop graphics debugger capture after render failure: {stop_error}"
                            );
                        }
                    }
                    eprintln!(
                        "native viewer surface failed; falling back to CPU presentation: {error}"
                    );
                    scene.detach_viewport_surface();
                    self.direct_present_enabled = false;
                    self.redraw_requested = true;
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            return;
        }
        let ready_frame_render_started = write_screenshot.then(Instant::now);
        match scene.render(&self.camera, self.size) {
            Ok(frame) => {
                let ready_frame_render_elapsed =
                    ready_frame_render_started.map(|started| started.elapsed());
                let scene_frame_timing = write_screenshot.then(|| scene.last_frame_timing_report());
                if capture_this_frame {
                    if let Err(error) =
                        finish_graphics_debugger_capture(scene, self.renderdoc_bridge.as_ref())
                    {
                        eprintln!("failed to complete graphics debugger capture: {error}");
                        event_loop.exit();
                        return;
                    }
                    self.renderdoc_capture_finished = true;
                }
                let screenshot_encode_started = write_screenshot.then(Instant::now);
                let screenshot_metadata = if write_screenshot {
                    let timing = scene_frame_timing
                        .expect("screenshot frames must retain their requested timing report");
                    let render_elapsed = ready_frame_render_elapsed
                        .expect("screenshot frames must retain their render interval");
                    let viewer_ready_elapsed = viewer_ready_started_at
                        .expect("screenshot frames must retain the viewer load start time")
                        .elapsed();
                    let ibl_report = scene.ibl_load_report();
                    let ibl_staging_timing = ibl_report.staging_timing();
                    let ibl_staging_output = ibl_report.staging_output();
                    let base_prewarm_report = scene.base_prewarm_report();
                    let shader_variant_miss_report = scene.shader_variant_miss_report();
                    let startup_timing = scene.startup_timing();
                    let (hdri_path, requested_source_face_size, requested_pmrem_face_size) =
                        screenshot_input
                            .expect("screenshot frames must retain their HDRI input identity");
                    Some(ReadyFrameEvidenceMetadata {
                        backend: scene.renderer_backend_name().to_owned(),
                        interactive_direct_present_enabled,
                        hdri_path,
                        requested_source_face_size,
                        requested_pmrem_face_size,
                        active_source_cubemap_face_size: ibl_report.source_cubemap_face_size(),
                        active_source_cubemap_mip_count: ibl_report.source_cubemap_mip_count(),
                        active_pmrem_face_size: ibl_report.pmrem_face_size(),
                        active_pmrem_mip_count: ibl_report.pmrem_mip_count(),
                        render_profile: PBR_VIEWER_RENDER_PROFILE.to_owned(),
                        environment_only_base_prewarm_pipeline_ready: base_prewarm_report
                            .pipeline_ready(),
                        environment_only_base_pipeline_ready_at_capture:
                            environment_only_base_pipeline_ready,
                        environment_only_base_prewarm_cache_hit: base_prewarm_report.cache_hit(),
                        environment_only_base_prewarm_shader_source_resolution: base_prewarm_report
                            .shader_source_resolution(),
                        environment_only_base_prewarm_pipeline_creation: base_prewarm_report
                            .pipeline_creation(),
                        environment_only_base_prewarm_elapsed: base_prewarm_report.elapsed(),
                        camera_yaw_degrees: screenshot_camera.yaw_degrees(),
                        camera_pitch_degrees: screenshot_camera.pitch_degrees(),
                        ibl_bake_algorithm_version: IBL_BAKE_ALGORITHM_VERSION,
                        ibl_staging_status: format!("{:?}", ibl_report.staging_status()),
                        ibl_staging_elapsed: ibl_report.staging_elapsed(),
                        ibl_staging_source_decode: ibl_staging_timing.source_decode(),
                        ibl_staging_cubemap_build: ibl_staging_timing.cubemap_build(),
                        ibl_staging_equirect_projection: ibl_staging_timing.equirect_projection(),
                        ibl_staging_source_mip_build: ibl_staging_timing.source_mip_build(),
                        ibl_staging_pmrem_build: ibl_staging_timing.pmrem_build(),
                        ibl_staging_sh9_build: ibl_staging_timing.sh9_build(),
                        ibl_staging_irradiance_cube_build: ibl_staging_timing
                            .irradiance_cube_build(),
                        ibl_staging_bundle_write: ibl_staging_timing.bundle_write(),
                        ibl_staging_source_zcube_bytes: ibl_staging_output.source_zcube_bytes(),
                        ibl_staging_asset_derived_bytes: ibl_staging_output.asset_derived_bytes(),
                        ibl_staging_parallel_executor_work_items: ibl_staging_output
                            .parallel_executor_work_items(),
                        ibl_staging_equirect_projection_parallel_work_items: ibl_staging_output
                            .equirect_projection_parallel_work_items(),
                        ibl_staging_source_mip_build_parallel_work_items: ibl_staging_output
                            .source_mip_build_parallel_work_items(),
                        ibl_staging_pmrem_build_parallel_work_items: ibl_staging_output
                            .pmrem_build_parallel_work_items(),
                        ibl_staging_irradiance_cube_build_parallel_work_items: ibl_staging_output
                            .irradiance_cube_build_parallel_work_items(),
                        ibl_staging_irradiance_cube_source_sample_visits: ibl_staging_output
                            .irradiance_cube_source_sample_visits(),
                        ibl_total_elapsed: ibl_report.total_elapsed(),
                        scene_startup_hdri_decode: startup_timing.hdri_decode(),
                        scene_startup_project_assets: startup_timing.project_assets(),
                        scene_startup_runtime_bootstrap: startup_timing.runtime_bootstrap(),
                        scene_startup_project_open: startup_timing.project_open(),
                        scene_startup_world_load: startup_timing.world_load(),
                        scene_startup_renderer_initialization: startup_timing
                            .renderer_initialization(),
                        scene_startup_renderer_backend_initialization: startup_timing
                            .renderer_backend_initialization(),
                        scene_startup_renderer_deferred_initialization: startup_timing
                            .renderer_deferred_initialization(),
                        scene_startup_renderer_deferred_standard_pipeline: startup_timing
                            .renderer_deferred_standard_pipeline(),
                        scene_startup_resource_streamer_initialization: startup_timing
                            .resource_streamer_initialization(),
                        scene_startup_ibl_restore: startup_timing.ibl_restore(),
                        scene_startup_total: startup_timing.total(),
                        one_shot_base_pipeline_wait_elapsed,
                        viewer_scene_load_elapsed,
                        viewer_ready_elapsed,
                        ready_frame_render_elapsed: render_elapsed,
                        ready_frame_render_extract: timing.render_extract(),
                        ready_frame_renderer_call: timing.renderer_frame_call(),
                        ready_frame_readback_and_completion: timing.readback_and_completion(),
                        shader_variant_miss_report,
                    })
                } else {
                    None
                };
                if write_screenshot {
                    if let Err(error) =
                        self.write_ready_frame_screenshot(&frame, screenshot_metadata.as_ref())
                    {
                        eprintln!("failed to write PBR viewer screenshot: {error}");
                        event_loop.exit();
                        return;
                    }
                    self.begin_gpu_timing_evidence(frame.generation);
                }
                let gpu_timing_report = scene.take_completed_gpu_timing_report();
                let gpu_timing_status = scene.last_gpu_timing_status();
                if let Err(error) =
                    self.resolve_gpu_timing_evidence(gpu_timing_report, gpu_timing_status)
                {
                    eprintln!("failed to write PBR viewer GPU timing evidence: {error}");
                    event_loop.exit();
                    return;
                }
                let screenshot_encode_elapsed =
                    screenshot_encode_started.map(|started| started.elapsed());
                let presenter = self
                    .presenter
                    .as_mut()
                    .expect("CPU presenter must exist for the CPU render path");
                let surface_present_started = write_screenshot.then(Instant::now);
                if let Err(error) = presenter.present(&frame) {
                    eprintln!("failed to present viewer frame: {error}");
                    event_loop.exit();
                    return;
                }
                if let (
                    Some(render_elapsed),
                    Some(scene_timing),
                    Some(encode_elapsed),
                    Some(present_started),
                ) = (
                    ready_frame_render_elapsed,
                    scene_frame_timing,
                    screenshot_encode_elapsed,
                    surface_present_started,
                ) {
                    println!(
                        "PBR viewer Ready-frame timing: render={render_elapsed:.2?} [extract={:.2?}, renderer_frame_call={:.2?}, readback_and_completion={:.2?}], screenshot_encode={encode_elapsed:.2?}, surface_present={:.2?}",
                        scene_timing.render_extract(),
                        scene_timing.renderer_frame_call(),
                        scene_timing.readback_and_completion(),
                        present_started.elapsed(),
                    );
                }
                self.flush_ready_window_title();
                self.log_first_ready_frame_presented();
                if recheck_base_pipeline_after_present {
                    self.schedule_base_pipeline_recheck(event_loop, None);
                }
                if capture_this_frame
                    && self.exit_after_capture
                    && !self.gpu_timing_evidence_pending()
                {
                    event_loop.exit();
                } else if self.exit_after_screenshot() {
                    event_loop.exit();
                }
            }
            Err(error) => {
                if capture_this_frame {
                    if let Err(stop_error) = scene.stop_graphics_debugger_capture() {
                        eprintln!(
                            "failed to stop graphics debugger capture after render failure: {stop_error}"
                        );
                    }
                }
                eprintln!("failed to render viewer frame: {error}");
                event_loop.exit();
            }
        }
    }

    fn present_startup_frame(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.present_status_frame(event_loop, startup_frame(self.size));
    }

    fn write_ready_frame_screenshot(
        &mut self,
        frame: &ViewportFrame,
        metadata: Option<&ReadyFrameEvidenceMetadata>,
    ) -> Result<(), String> {
        let Some(path) = self.screenshot_path.as_ref() else {
            return Ok(());
        };
        if self.screenshot_written {
            return Ok(());
        }
        let metadata = metadata.ok_or_else(|| {
            "Ready-frame screenshot requires an IBL and frame-timing provenance record".to_owned()
        })?;
        let metadata_path =
            write_ready_frame_evidence(path, frame.width, frame.height, &frame.rgba, metadata)?;
        self.screenshot_written = true;
        println!(
            "wrote PBR viewer Ready-frame screenshot and provenance: {} / {}",
            path.display(),
            metadata_path.display(),
        );
        Ok(())
    }

    fn begin_gpu_timing_evidence(&mut self, frame_generation: u64) {
        if self.gpu_timing_report_path.is_some() {
            self.gpu_timing_request = Some(GpuTimingEvidenceRequest::new(frame_generation));
        }
    }

    fn resolve_gpu_timing_evidence(
        &mut self,
        report: Option<SceneRendererGpuTimingReport>,
        status: zircon_runtime::core::framework::render::RenderGpuTimingStatus,
    ) -> Result<bool, String> {
        let Some(request) = self.gpu_timing_request.as_mut() else {
            return Ok(false);
        };
        let resolution = request.observe(report, status);
        if matches!(resolution, GpuTimingEvidenceResolution::Pending) {
            self.request_redraw();
            return Ok(false);
        }

        let path = self
            .gpu_timing_report_path
            .as_ref()
            .expect("a GPU timing request requires an output path");
        if let Some(parent) = gpu_timing_report_parent(path) {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let screenshot_path = self
            .screenshot_path
            .as_deref()
            .expect("a GPU timing request requires a screenshot path");
        validate_gpu_timing_report_output(screenshot_path, path)?;
        let timing_evidence = format_gpu_timing_evidence(screenshot_path, &resolution)?;
        fs::write(path, timing_evidence).map_err(|error| error.to_string())?;
        println!("wrote PBR viewer GPU timing evidence: {}", path.display());
        self.gpu_timing_request = None;
        Ok(true)
    }

    fn gpu_timing_evidence_pending(&self) -> bool {
        self.gpu_timing_request.is_some()
    }

    fn exit_after_screenshot(&self) -> bool {
        self.screenshot_path.is_some()
            && self.screenshot_written
            && !self.gpu_timing_evidence_pending()
    }

    fn present_error_frame(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.present_status_frame(event_loop, error_frame(self.size));
    }

    fn present_status_frame(&mut self, event_loop: &dyn ActiveEventLoop, frame: ViewportFrame) {
        let Some(presenter) = self.presenter.as_mut() else {
            return;
        };
        if let Err(error) = presenter.present(&frame) {
            eprintln!("failed to present viewer status frame: {error}");
            event_loop.exit();
        }
    }

    fn update_pointer_position(&mut self, position: PhysicalPosition<f64>) {
        if self.left_dragging {
            if let Some(previous) = self.last_pointer_position {
                let delta_x = position.x - previous.x;
                let delta_y = position.y - previous.y;
                self.camera.drag(delta_x as f32, delta_y as f32);
                self.mark_ready_window_title_dirty();
                self.request_redraw();
            }
        }
        self.last_pointer_position = Some(position);
    }

    fn handle_pointer_button(
        &mut self,
        state: ElementState,
        button: ButtonSource,
        position: PhysicalPosition<f64>,
    ) {
        if button.mouse_button() != Some(MouseButton::Left) {
            return;
        }
        self.left_dragging = state == ElementState::Pressed;
        self.last_pointer_position = Some(position);
    }

    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let wheel_y = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(position) => position.y as f32 / 96.0,
        };
        self.camera.zoom(wheel_y);
        self.mark_ready_window_title_dirty();
        self.request_redraw();
    }

    fn mark_ready_window_title_dirty(&mut self) {
        if self.scene.is_some() {
            self.ready_title_dirty = true;
        }
    }

    fn flush_ready_window_title(&mut self) {
        if !consume_ready_title_update(&mut self.ready_title_dirty) {
            return;
        }
        let (Some(window), Some(scene)) = (self.window.as_ref(), self.scene.as_ref()) else {
            return;
        };
        window.set_title(&ready_window_title(
            self.camera,
            Some(scene.ibl_load_report()),
        ));
    }
}

fn request_redraw_transition(redraw_requested: &mut bool) -> bool {
    if *redraw_requested {
        return false;
    }
    *redraw_requested = true;
    true
}

fn load_status_refresh_is_due(last_refresh_at: Option<Instant>, now: Instant) -> bool {
    last_refresh_at.is_none_or(|last_refresh_at| {
        now.saturating_duration_since(last_refresh_at) >= LOAD_STATUS_REFRESH_INTERVAL
    })
}

fn load_status_refresh_deadline(last_refresh_at: Option<Instant>, now: Instant) -> Instant {
    last_refresh_at
        .and_then(|last_refresh_at| last_refresh_at.checked_add(LOAD_STATUS_REFRESH_INTERVAL))
        .unwrap_or(now + LOAD_STATUS_REFRESH_INTERVAL)
}

fn consume_ready_title_update(ready_title_dirty: &mut bool) -> bool {
    if !*ready_title_dirty {
        return false;
    }
    *ready_title_dirty = false;
    true
}

fn finish_graphics_debugger_capture(
    scene: &PbrMirrorScene,
    bridge: Option<&RenderDocBridge>,
) -> Result<(), String> {
    scene
        .stop_graphics_debugger_capture()
        .map_err(|error| format!("stop graphics debugger capture: {error}"))?;
    if let Some(bridge) = bridge {
        let report = bridge.capture_report()?;
        let capture_path = report.capture_path_for_evidence()?;
        println!(
            "RenderDoc capture report: count={}, latest_path={}",
            report.capture_count(),
            capture_path.display(),
        );
        println!("graphics debugger capture completed");
    } else {
        println!("graphics debugger capture stopped without a direct RenderDoc evidence record");
    }
    Ok(())
}

fn viewport_surface_descriptor(
    window: &dyn Window,
    size: UVec2,
) -> Result<RenderViewportSurfaceDescriptor, String> {
    let raw_window_handle = window
        .window_handle()
        .map_err(|error| format!("read native window handle: {error}"))?
        .as_raw();
    let RawWindowHandle::Win32(window) = raw_window_handle else {
        return Err("native GPU presentation currently requires a Win32 window".to_owned());
    };
    let hwnd = u64::try_from(window.hwnd.get())
        .map_err(|_| "Win32 hwnd is outside the runtime surface ABI range".to_owned())?;
    let hinstance = window
        .hinstance
        .map(|hinstance| {
            u64::try_from(hinstance.get())
                .map_err(|_| "Win32 hinstance is outside the runtime surface ABI range".to_owned())
        })
        .transpose()?;
    Ok(RenderViewportSurfaceDescriptor::new(
        size,
        RenderNativeSurfaceTarget::Win32 { hwnd, hinstance },
    ))
}

fn one_shot_run_exits_after_load_failure(
    screenshot_requested: bool,
    renderdoc_capture_once: bool,
    exit_after_capture: bool,
) -> bool {
    screenshot_requested || (renderdoc_capture_once && exit_after_capture)
}

impl ApplicationHandler for PbrMirrorViewerApp {
    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.finish_scene_load(event_loop);
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.ensure_window(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.ensure_window(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::SurfaceResized(size) => self.resize(event_loop, size),
            WindowEvent::PointerMoved { position, .. } => self.update_pointer_position(position),
            WindowEvent::PointerButton {
                state,
                button,
                position,
                ..
            } => self.handle_pointer_button(state, button, position),
            WindowEvent::MouseWheel { delta, .. } => self.handle_mouse_wheel(delta),
            WindowEvent::RedrawRequested => self.render_and_present(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.finish_scene_load(event_loop);
        self.refresh_scene_load_status(event_loop);
        self.request_base_pipeline_recheck_if_due(event_loop);
    }
}

fn loading_window_title(elapsed: Duration) -> String {
    format!(
        "Zircon PBR HDRI Mirror Viewer - preparing HDRI/PMREM - {}s - window responsive",
        elapsed.as_secs()
    )
}

fn ready_window_title(
    camera: OrbitCamera,
    ibl_load_report: Option<PbrMirrorSceneIblLoadReport>,
) -> String {
    let Some(report) = ibl_load_report else {
        return format!(
            "Zircon PBR HDRI Mirror Viewer - Loading - IBL unavailable - yaw {:.0} pitch {:.0}",
            camera.yaw_degrees(),
            camera.pitch_degrees()
        );
    };
    let ibl_status = format!(
        "IBL {:?} staging {}ms total {}ms",
        report.staging_status(),
        report.staging_elapsed().as_millis(),
        report.total_elapsed().as_millis(),
    );
    format!(
        "Zircon PBR HDRI Mirror Viewer - Ready - {ibl_status} - yaw {:.0} pitch {:.0}",
        camera.yaw_degrees(),
        camera.pitch_degrees()
    )
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
