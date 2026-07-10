use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size};
use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::ViewportFrame;

use crate::args::ViewerConfig;
use crate::background_load::{BackgroundTask, BackgroundTaskPoll};
use crate::camera::OrbitCamera;
use crate::presenter::{window_size, SoftbufferViewportPresenter};
use crate::scene::PbrMirrorScene;

const DEFAULT_WINDOW_WIDTH: u32 = 1280;
const DEFAULT_WINDOW_HEIGHT: u32 = 960;
const MIN_WINDOW_WIDTH: f64 = 480.0;
const MIN_WINDOW_HEIGHT: f64 = 360.0;

pub(crate) struct PbrMirrorViewerApp {
    hdri_path: PathBuf,
    face_size: u32,
    renderdoc_capture_once: bool,
    exit_after_capture: bool,
    renderdoc_capture_finished: bool,
    scene: Option<PbrMirrorScene>,
    scene_loader: Option<BackgroundTask<PbrMirrorScene>>,
    scene_load_started_at: Option<Instant>,
    load_error: Option<String>,
    event_loop_proxy: EventLoopProxy,
    camera: OrbitCamera,
    window: Option<Arc<dyn Window>>,
    presenter: Option<SoftbufferViewportPresenter>,
    size: UVec2,
    left_dragging: bool,
    last_pointer_position: Option<PhysicalPosition<f64>>,
    redraw_requested: bool,
}

impl PbrMirrorViewerApp {
    pub(crate) fn new(config: ViewerConfig, event_loop_proxy: EventLoopProxy) -> Self {
        Self {
            hdri_path: config.hdri_path,
            face_size: config.face_size,
            renderdoc_capture_once: config.renderdoc_capture_once,
            exit_after_capture: config.exit_after_capture,
            renderdoc_capture_finished: false,
            scene: None,
            scene_loader: None,
            scene_load_started_at: None,
            load_error: None,
            event_loop_proxy,
            camera: OrbitCamera::default(),
            window: None,
            presenter: None,
            size: UVec2::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            left_dragging: false,
            last_pointer_position: None,
            redraw_requested: true,
        }
    }

    fn ensure_window(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("Zircon PBR HDRI Mirror Viewer - loading HDRI/PMREM")
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
        self.presenter = match SoftbufferViewportPresenter::new(window.clone()) {
            Ok(presenter) => Some(presenter),
            Err(error) => {
                eprintln!("failed to create viewer presenter: {error}");
                event_loop.exit();
                return;
            }
        };
        self.window = Some(window.clone());
        self.present_startup_frame(event_loop);
        self.start_scene_load(event_loop);
        self.request_redraw();
        window.request_redraw();
    }

    fn start_scene_load(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.scene.is_some() || self.scene_loader.is_some() || self.load_error.is_some() {
            return;
        }

        let hdri_path = self.hdri_path.clone();
        let face_size = self.face_size;
        let event_loop_proxy = self.event_loop_proxy.clone();
        match BackgroundTask::spawn(
            "zircon-pbr-scene-loader",
            move || PbrMirrorScene::new(&hdri_path, face_size).map_err(|error| error.to_string()),
            move || event_loop_proxy.wake_up(),
        ) {
            Ok(loader) => {
                self.scene_load_started_at = Some(Instant::now());
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

        match result {
            Ok(scene) => {
                self.scene = Some(scene);
                let elapsed = self
                    .scene_load_started_at
                    .take()
                    .map(|started| started.elapsed());
                if let Some(window) = self.window.as_ref() {
                    window.set_title("Zircon PBR HDRI Mirror Viewer - Ready");
                }
                if let Some(elapsed) = elapsed {
                    println!("HDRI/PMREM scene ready after {:.2?}", elapsed);
                }
                self.request_redraw();
            }
            Err(message) => self.handle_scene_load_failure(event_loop, message),
        }
    }

    fn handle_scene_load_failure(&mut self, event_loop: &dyn ActiveEventLoop, message: String) {
        eprintln!("failed to load PBR HDRI viewer scene: {message}");
        self.load_error = Some(message);
        if let Some(window) = self.window.as_ref() {
            window.set_title("Zircon PBR HDRI Mirror Viewer - load failed");
        }
        self.present_error_frame(event_loop);
    }

    fn request_redraw(&mut self) {
        self.redraw_requested = true;
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn resize(&mut self, event_loop: &dyn ActiveEventLoop, size: PhysicalSize<u32>) {
        self.size = UVec2::new(size.width.max(1), size.height.max(1));
        let Some(presenter) = self.presenter.as_mut() else {
            return;
        };
        if let Err(error) = presenter.resize(self.size) {
            eprintln!("failed to resize viewer presenter: {error}");
            event_loop.exit();
            return;
        }
        self.request_redraw();
    }

    fn render_and_present(&mut self, event_loop: &dyn ActiveEventLoop) {
        if !self.redraw_requested {
            return;
        }
        self.redraw_requested = false;

        let Some(scene) = self.scene.as_mut() else {
            if self.load_error.is_some() {
                self.present_error_frame(event_loop);
            } else {
                self.present_startup_frame(event_loop);
            }
            return;
        };

        let Some(presenter) = self.presenter.as_mut() else {
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
        match scene.render(&self.camera, self.size) {
            Ok(frame) => {
                if capture_this_frame {
                    if let Err(error) = scene.stop_graphics_debugger_capture() {
                        eprintln!("failed to stop graphics debugger capture: {error}");
                        event_loop.exit();
                        return;
                    }
                    self.renderdoc_capture_finished = true;
                    println!("graphics debugger capture completed");
                }
                if let Err(error) = presenter.present(&frame) {
                    eprintln!("failed to present viewer frame: {error}");
                    event_loop.exit();
                    return;
                }
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
                eprintln!("failed to render viewer frame: {error}");
                event_loop.exit();
            }
        }
    }

    fn present_startup_frame(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.present_status_frame(event_loop, startup_frame(self.size));
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
        self.request_redraw();
    }
}

fn startup_frame(size: UVec2) -> ViewportFrame {
    status_frame(size, [10, 15, 21], [35, 59, 80])
}

fn error_frame(size: UVec2) -> ViewportFrame {
    status_frame(size, [42, 12, 18], [94, 30, 38])
}

fn status_frame(size: UVec2, top: [u8; 3], bottom: [u8; 3]) -> ViewportFrame {
    let width = size.x.max(1);
    let height = size.y.max(1);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let t = y as f32 / height.saturating_sub(1).max(1) as f32;
        for x in 0..width {
            let shimmer = if ((x / 18) + (y / 18)) & 1 == 0 { 6 } else { 0 };
            rgba.push(lerp_u8(top[0], bottom[0], t).saturating_add(shimmer));
            rgba.push(lerp_u8(top[1], bottom[1], t).saturating_add(shimmer));
            rgba.push(lerp_u8(top[2], bottom[2], t).saturating_add(shimmer));
            rgba.push(255);
        }
    }
    ViewportFrame {
        width,
        height,
        rgba,
        generation: 0,
        capture_report: Default::default(),
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
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
        if self.redraw_requested {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }
}
