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
    RenderNativeSurfaceTarget, RenderViewportSurfaceDescriptor,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::ViewportFrame;

use crate::args::ViewerConfig;
use crate::background_load::{BackgroundTask, BackgroundTaskPoll};
use crate::camera::OrbitCamera;
use crate::frame_io::{error_frame, startup_frame, write_ready_frame_png};
use crate::presenter::{window_size, SoftbufferViewportPresenter};
use crate::renderdoc::RenderDocBridge;
use crate::scene::{PbrMirrorScene, PbrMirrorSceneIblLoadReport};

const DEFAULT_WINDOW_WIDTH: u32 = 1280;
const DEFAULT_WINDOW_HEIGHT: u32 = 960;
const MIN_WINDOW_WIDTH: f64 = 480.0;
const MIN_WINDOW_HEIGHT: f64 = 360.0;
const LOAD_STATUS_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

pub(crate) struct PbrMirrorViewerApp {
    hdri_path: PathBuf,
    // Preserve automatic sizing until the background loader can inspect the HDR image.
    face_size: Option<u32>,
    pmrem_face_size: Option<u32>,
    ibl_cache_dir: Option<PathBuf>,
    screenshot_path: Option<PathBuf>,
    screenshot_written: bool,
    renderdoc_capture_once: bool,
    renderdoc_bridge: Option<RenderDocBridge>,
    exit_after_capture: bool,
    renderdoc_capture_finished: bool,
    scene: Option<PbrMirrorScene>,
    scene_loader: Option<BackgroundTask<PbrMirrorScene>>,
    scene_load_started_at: Option<Instant>,
    last_load_status_refresh_at: Option<Instant>,
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
            ibl_cache_dir: config.ibl_cache_dir,
            screenshot_path: config.screenshot_path,
            screenshot_written: false,
            renderdoc_capture_once: config.renderdoc_capture_once,
            renderdoc_bridge,
            exit_after_capture: config.exit_after_capture,
            renderdoc_capture_finished: false,
            scene: None,
            scene_loader: None,
            scene_load_started_at: None,
            last_load_status_refresh_at: None,
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

        let hdri_path = self.hdri_path.clone();
        let face_size = self.face_size;
        let pmrem_face_size = self.pmrem_face_size;
        let ibl_cache_dir = self.ibl_cache_dir.clone();
        let event_loop_proxy = self.event_loop_proxy.clone();
        match BackgroundTask::spawn(
            "zircon-pbr-scene-loader",
            move || {
                PbrMirrorScene::new(
                    &hdri_path,
                    face_size,
                    pmrem_face_size,
                    ibl_cache_dir.as_deref(),
                )
                .map_err(|error| error.to_string())
            },
            move || event_loop_proxy.wake_up(),
        ) {
            Ok(loader) => {
                let started_at = Instant::now();
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
                let elapsed = self
                    .scene_load_started_at
                    .take()
                    .map(|started| started.elapsed());
                self.ready_title_dirty = true;
                // A loader wake-up does not necessarily carry a redraw request. Force the first
                // ready frame here so the Ready title never leaves the startup checkerboard.
                let first_ready_frame_started = Instant::now();
                self.redraw_requested = true;
                self.render_and_present(event_loop);
                let first_ready_frame_elapsed = first_ready_frame_started.elapsed();
                if let Some(scene_load_elapsed) = elapsed {
                    println!(
                        "PBR viewer readiness: scene_constructed={scene_load_elapsed:.2?}, first_frame_presented={:.2?}, total={:.2?}",
                        first_ready_frame_elapsed,
                        scene_load_elapsed + first_ready_frame_elapsed,
                    );
                }
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

        let write_screenshot = self.screenshot_path.is_some() && !self.screenshot_written;
        if !self.direct_present_enabled || write_screenshot {
            if let Err(error) = self.ensure_cpu_presenter() {
                eprintln!("failed to create CPU presentation fallback: {error}");
                event_loop.exit();
                return;
            }
        }

        let Some(scene) = self.scene.as_mut() else {
            if self.load_error.is_some() {
                self.present_error_frame(event_loop);
            } else {
                self.present_startup_frame(event_loop);
            }
            return;
        };

        let capture_this_frame = self.renderdoc_capture_once && !self.renderdoc_capture_finished;
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
                        if let Err(error) = scene.stop_graphics_debugger_capture() {
                            eprintln!("failed to stop graphics debugger capture: {error}");
                            event_loop.exit();
                            return;
                        }
                        self.renderdoc_capture_finished = true;
                        println!("graphics debugger capture completed");
                        if let Some(bridge) = self.renderdoc_bridge.as_ref() {
                            match bridge.capture_report() {
                                Ok(report) => println!(
                                    "RenderDoc capture report: count={}, latest_path={}",
                                    report.capture_count(),
                                    report.latest_capture_path().map_or_else(
                                        || "<none>".to_owned(),
                                        |path| path.display().to_string(),
                                    ),
                                ),
                                Err(error) => {
                                    eprintln!("failed to query RenderDoc capture report: {error}");
                                }
                            }
                        }
                    }
                    self.flush_ready_window_title();
                    if capture_this_frame && self.exit_after_capture {
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
                    if let Err(error) = scene.stop_graphics_debugger_capture() {
                        eprintln!("failed to stop graphics debugger capture: {error}");
                        event_loop.exit();
                        return;
                    }
                    self.renderdoc_capture_finished = true;
                    println!("graphics debugger capture completed");
                    if let Some(bridge) = self.renderdoc_bridge.as_ref() {
                        match bridge.capture_report() {
                            Ok(report) => println!(
                                "RenderDoc capture report: count={}, latest_path={}",
                                report.capture_count(),
                                report.latest_capture_path().map_or_else(
                                    || "<none>".to_owned(),
                                    |path| path.display().to_string(),
                                ),
                            ),
                            Err(error) => {
                                eprintln!("failed to query RenderDoc capture report: {error}");
                            }
                        }
                    }
                }
                let screenshot_encode_started = write_screenshot.then(Instant::now);
                if let Err(error) = self.write_ready_frame_screenshot(&frame) {
                    eprintln!("failed to write PBR viewer screenshot: {error}");
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
                if capture_this_frame && self.exit_after_capture {
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

    fn write_ready_frame_screenshot(&mut self, frame: &ViewportFrame) -> Result<(), String> {
        let Some(path) = self.screenshot_path.as_ref() else {
            return Ok(());
        };
        if self.screenshot_written {
            return Ok(());
        }
        write_ready_frame_png(path, frame.width, frame.height, &frame.rgba)?;
        self.screenshot_written = true;
        println!(
            "wrote PBR viewer Ready-frame screenshot: {}",
            path.display()
        );
        Ok(())
    }

    fn exit_after_screenshot(&self) -> bool {
        self.screenshot_path.is_some() && self.screenshot_written
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
