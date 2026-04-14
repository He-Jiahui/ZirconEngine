//! Entry runners that bootstrap the core runtime and host editor/runtime shells.

use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};
use zircon_core::{CoreError, CoreHandle, CoreRuntime, ModuleDescriptor};
use zircon_editor::run_editor;
use zircon_graphics::{
    create_runtime_preview_renderer, EditorOrRuntimeFrame, GraphicsError, RuntimePreviewRenderer,
    ViewportController, ViewportInput, ViewportState,
};
use zircon_manager::{InputButton, InputEvent, InputManager, ManagerResolver};
use zircon_math::{UVec2, Vec2};
use zircon_scene::{create_default_level, LevelSystem, NodeKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryProfile {
    Editor,
    Runtime,
    Headless,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryConfig {
    pub profile: EntryProfile,
}

impl EntryConfig {
    pub const fn new(profile: EntryProfile) -> Self {
        Self { profile }
    }
}

#[derive(Clone, Debug)]
pub struct BuiltinEntryModuleSet {
    descriptors: Vec<ModuleDescriptor>,
}

impl BuiltinEntryModuleSet {
    pub fn for_profile(profile: EntryProfile) -> Self {
        let mut descriptors = vec![
            zircon_manager::module_descriptor(),
            zircon_platform::module_descriptor(),
            zircon_input::module_descriptor(),
            zircon_asset::module_descriptor(),
            zircon_graphics::module_descriptor(),
            zircon_scene::module_descriptor(),
            zircon_script::module_descriptor(),
            zircon_physics::module_descriptor(),
            zircon_sound::module_descriptor(),
            zircon_texture::module_descriptor(),
            zircon_ui::module_descriptor(),
            zircon_net::module_descriptor(),
            zircon_navigation::module_descriptor(),
            zircon_particles::module_descriptor(),
            zircon_animation::module_descriptor(),
        ];

        if matches!(profile, EntryProfile::Editor) {
            descriptors.push(zircon_editor::module_descriptor());
        }

        Self { descriptors }
    }

    pub fn descriptors(&self) -> &[ModuleDescriptor] {
        &self.descriptors
    }
}

#[derive(Debug, Default)]
pub struct EntryRunner;

impl EntryRunner {
    pub fn bootstrap(config: EntryConfig) -> Result<CoreHandle, CoreError> {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();
        let modules = BuiltinEntryModuleSet::for_profile(config.profile);

        for descriptor in modules.descriptors() {
            runtime.register_module(descriptor.clone())?;
        }
        for descriptor in modules.descriptors() {
            runtime.activate_module(&descriptor.name)?;
        }

        Ok(handle)
    }

    pub fn run_editor() -> Result<(), Box<dyn Error>> {
        let core = Self::bootstrap(EntryConfig::new(EntryProfile::Editor))?;
        run_editor(core)?;
        Ok(())
    }

    pub fn run_runtime() -> Result<(), Box<dyn Error>> {
        let core = Self::bootstrap(EntryConfig::new(EntryProfile::Runtime))?;
        let event_loop = EventLoop::new()?;
        let mut app = RuntimeEntryApp::new(core)?;
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    pub fn run_headless() -> Result<(), Box<dyn Error>> {
        let _ = Self::bootstrap(EntryConfig::new(EntryProfile::Headless))?;
        Ok(())
    }
}

struct RuntimeEntryApp {
    core: CoreHandle,
    window: Option<Arc<Window>>,
    renderer: Option<RuntimePreviewRenderer>,
    level: LevelSystem,
    viewport_controller: ViewportController,
    cursor: Vec2,
    input_manager: Arc<dyn InputManager>,
}

impl RuntimeEntryApp {
    fn new(core: CoreHandle) -> Result<Self, Box<dyn Error>> {
        let resolver = ManagerResolver::new(core.clone());
        let input_manager = resolver.input()?;
        let level = create_default_level(&core)?;
        let orbit_target = level.with_world_mut(|world| {
            let cube = world
                .nodes()
                .iter()
                .find(|node| matches!(&node.kind, NodeKind::Cube))
                .map(|node| node.id)
                .unwrap_or(world.active_camera());
            world.set_selected(Some(cube));
            world.find_node(cube)
                .map(|node| node.transform.translation)
                .unwrap_or_default()
        });

        let mut viewport_controller =
            ViewportController::new(ViewportState::new(UVec2::new(1280, 720)));
        viewport_controller.set_orbit_target(orbit_target);

        Ok(Self {
            core,
            window: None,
            renderer: None,
            level,
            viewport_controller,
            cursor: Vec2::ZERO,
            input_manager,
        })
    }

    fn current_frame(&self) -> EditorOrRuntimeFrame {
        EditorOrRuntimeFrame {
            scene: self.level.with_world(|world| world.to_render_snapshot()),
            viewport: self.viewport_controller.viewport().clone(),
        }
    }

    fn handle_viewport_input(&mut self, input: ViewportInput) {
        let _feedback = self
            .level
            .with_world_mut(|world| self.viewport_controller.handle_input(world, input));
        let orbit_target = self.level.with_world(|world| {
            world.selected_node()
                .and_then(|selected| world.find_node(selected))
                .map(|node| node.transform.translation)
        });
        if let Some(target) = orbit_target {
            self.viewport_controller.set_orbit_target(target);
        }
    }
}

impl ApplicationHandler for RuntimeEntryApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Zircon Runtime")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)),
                )
                .expect("runtime window"),
        );
        let inner = window.inner_size();
        self.handle_viewport_input(ViewportInput::Resized(UVec2::new(
            inner.width,
            inner.height,
        )));
        self.window = Some(window.clone());
        self.renderer = Some(
            create_runtime_preview_renderer(&self.core, window).expect("runtime preview renderer"),
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
                self.handle_viewport_input(ViewportInput::Resized(UVec2::new(
                    size.width,
                    size.height,
                )));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Vec2::new(position.x as f32, position.y as f32);
                self.input_manager.submit_event(InputEvent::CursorMoved {
                    x: self.cursor.x,
                    y: self.cursor.y,
                });
                self.handle_viewport_input(ViewportInput::PointerMoved(self.cursor));
            }
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
                    self.handle_viewport_input(ViewportInput::LeftPressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Left) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonReleased(InputButton::MouseLeft));
                    self.handle_viewport_input(ViewportInput::LeftReleased);
                }
                (ElementState::Pressed, MouseButton::Right) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonPressed(InputButton::MouseRight));
                    self.handle_viewport_input(ViewportInput::RightPressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Right) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonReleased(InputButton::MouseRight));
                    self.handle_viewport_input(ViewportInput::RightReleased);
                }
                (ElementState::Pressed, MouseButton::Middle) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonPressed(InputButton::MouseMiddle));
                    self.handle_viewport_input(ViewportInput::MiddlePressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Middle) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonReleased(InputButton::MouseMiddle));
                    self.handle_viewport_input(ViewportInput::MiddleReleased);
                }
                _ => {}
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.1,
                };
                self.input_manager
                    .submit_event(InputEvent::WheelScrolled { delta: amount });
                self.handle_viewport_input(ViewportInput::Scrolled(amount));
            }
            WindowEvent::RedrawRequested => {
                let frame = self.current_frame();
                if let Some(renderer) = self.renderer.as_mut() {
                    if let Err(error) = renderer.render(&frame) {
                        match error {
                            GraphicsError::Surface(wgpu::SurfaceError::Lost)
                            | GraphicsError::Surface(wgpu::SurfaceError::Outdated) => {
                                renderer.resize(frame.viewport.size);
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

#[cfg(test)]
mod tests {
    use zircon_editor::{EditorManager, EDITOR_MANAGER_NAME};
    use zircon_manager::{
        resolve_asset_manager, resolve_config_manager, resolve_event_manager, resolve_input_manager,
        resolve_rendering_manager, ManagerResolver,
    };
    use zircon_scene::create_default_level;

    use super::{BuiltinEntryModuleSet, EntryConfig, EntryProfile, EntryRunner};

    #[test]
    fn editor_bootstrap_registers_editor_and_primary_managers() {
        let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Editor)).unwrap();
        let asset_manager = resolve_asset_manager(&core).unwrap();
        let rendering_manager = resolve_rendering_manager(&core).unwrap();
        let input_manager = resolve_input_manager(&core).unwrap();
        let config_manager = resolve_config_manager(&core).unwrap();
        let event_manager = resolve_event_manager(&core).unwrap();
        let level = create_default_level(&core).unwrap();
        let _editor_manager = core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();

        assert!(asset_manager.pipeline_info().default_worker_count > 0);
        assert!(level.snapshot().nodes().len() >= 3);
        assert_eq!(rendering_manager.backend_info().backend_name, "wgpu");
        input_manager.submit_event(zircon_manager::InputEvent::ButtonPressed(
            zircon_manager::InputButton::MouseLeft,
        ));
        assert_eq!(
            input_manager.snapshot().pressed_buttons,
            vec![zircon_manager::InputButton::MouseLeft]
        );
        config_manager
            .set_value("editor.mode", serde_json::json!("docked"))
            .unwrap();
        assert_eq!(
            config_manager.get_value("editor.mode"),
            Some(serde_json::json!("docked"))
        );
        let receiver = event_manager.subscribe("editor.ready");
        event_manager.publish("editor.ready", serde_json::json!({ "booted": true }));
        assert_eq!(receiver.recv().unwrap().payload["booted"], true);
    }

    #[test]
    fn runtime_bootstrap_excludes_editor_module() {
        let modules = BuiltinEntryModuleSet::for_profile(EntryProfile::Runtime);
        assert!(modules
            .descriptors()
            .iter()
            .all(|descriptor| descriptor.name != zircon_editor::EDITOR_MODULE_NAME));

        let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Runtime)).unwrap();
        assert!(core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .is_err());
        assert!(ManagerResolver::new(core).rendering().is_ok());
    }
}
