//! Application runners for the editor host and the runtime preview host.

use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};
use zircon_asset::AssetWorkerPool;
use zircon_editor::run_editor;
use zircon_graphics::{
    EditorOrRuntimeFrame, GraphicsError, RuntimePreviewRenderer, ViewportController, ViewportInput,
    ViewportState,
};
use zircon_math::{UVec2, Vec2};
use zircon_scene::{NodeKind, Scene};

pub fn run_editor_runner() -> iced::Result {
    run_editor()
}

pub fn run_runtime_preview_runner() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = RuntimePreviewApp::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct RuntimePreviewApp {
    window: Option<Arc<Window>>,
    renderer: Option<RuntimePreviewRenderer>,
    worker_pool: AssetWorkerPool,
    scene: Scene,
    viewport_controller: ViewportController,
    cursor: Vec2,
}

impl RuntimePreviewApp {
    fn new() -> Self {
        let mut scene = Scene::new();
        let cube = scene
            .nodes()
            .iter()
            .find(|node| matches!(&node.kind, NodeKind::Cube))
            .map(|node| node.id)
            .unwrap_or(scene.active_camera());
        scene.set_selected(Some(cube));
        let orbit_target = scene
            .find_node(cube)
            .map(|node| node.transform.translation)
            .unwrap_or_default();
        let mut viewport_controller =
            ViewportController::new(ViewportState::new(UVec2::new(1280, 720)));
        viewport_controller.set_orbit_target(orbit_target);

        Self {
            window: None,
            renderer: None,
            worker_pool: AssetWorkerPool::new(
                std::thread::available_parallelism().map_or(2, |n| n.get().max(2) - 1),
            )
            .expect("asset workers"),
            scene,
            viewport_controller,
            cursor: Vec2::ZERO,
        }
    }

    fn current_frame(&self) -> EditorOrRuntimeFrame {
        EditorOrRuntimeFrame {
            scene: self.scene.to_render_snapshot(),
            viewport: self.viewport_controller.viewport().clone(),
        }
    }
}

impl ApplicationHandler for RuntimePreviewApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Zircon Runtime Preview")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)),
                )
                .expect("preview window"),
        );
        let inner = window.inner_size();
        self.viewport_controller.handle_input(
            &mut self.scene,
            ViewportInput::Resized(UVec2::new(inner.width, inner.height)),
        );
        self.window = Some(window.clone());
        self.renderer = Some(
            RuntimePreviewRenderer::new(
                window,
                self.worker_pool.request_sender(),
                self.worker_pool.completion_receiver(),
            )
            .expect("preview renderer"),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(UVec2::new(size.width, size.height));
                }
                let _ = self.viewport_controller.handle_input(
                    &mut self.scene,
                    ViewportInput::Resized(UVec2::new(size.width, size.height)),
                );
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Vec2::new(position.x as f32, position.y as f32);
                let _ = self
                    .viewport_controller
                    .handle_input(&mut self.scene, ViewportInput::PointerMoved(self.cursor));
            }
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => {
                    let _ = self
                        .viewport_controller
                        .handle_input(&mut self.scene, ViewportInput::LeftPressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Left) => {
                    let _ = self
                        .viewport_controller
                        .handle_input(&mut self.scene, ViewportInput::LeftReleased);
                }
                (ElementState::Pressed, MouseButton::Right) => {
                    let _ = self
                        .viewport_controller
                        .handle_input(&mut self.scene, ViewportInput::RightPressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Right) => {
                    let _ = self
                        .viewport_controller
                        .handle_input(&mut self.scene, ViewportInput::RightReleased);
                }
                (ElementState::Pressed, MouseButton::Middle) => {
                    let _ = self
                        .viewport_controller
                        .handle_input(&mut self.scene, ViewportInput::MiddlePressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Middle) => {
                    let _ = self
                        .viewport_controller
                        .handle_input(&mut self.scene, ViewportInput::MiddleReleased);
                }
                _ => {}
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.1,
                };
                let _ = self
                    .viewport_controller
                    .handle_input(&mut self.scene, ViewportInput::Scrolled(amount));
            }
            WindowEvent::RedrawRequested => {
                let frame = self.current_frame();
                if let Some(renderer) = self.renderer.as_mut() {
                    if let Err(error) = renderer.render(&frame) {
                        match error {
                            GraphicsError::Surface(wgpu::SurfaceError::Lost)
                            | GraphicsError::Surface(wgpu::SurfaceError::Outdated) => {
                                let size = frame.viewport.size;
                                renderer.resize(size);
                            }
                            GraphicsError::Surface(wgpu::SurfaceError::OutOfMemory) => {
                                event_loop.exit();
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
