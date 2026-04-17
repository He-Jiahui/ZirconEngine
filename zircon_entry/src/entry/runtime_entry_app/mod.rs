mod application_handler;
mod camera_controller;
mod construct;

use std::sync::Arc;

use winit::window::Window;
use zircon_core::CoreHandle;
use zircon_manager::InputManager;
use zircon_math::Vec2;
use zircon_scene::LevelSystem;

use self::camera_controller::RuntimeCameraController;
use crate::runtime_presenter::{RenderServerRuntimeBridge, SoftbufferRuntimePresenter};

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<dyn Window>>,
    presenter: Option<SoftbufferRuntimePresenter>,
    render_bridge: RenderServerRuntimeBridge,
    level: LevelSystem,
    camera_controller: RuntimeCameraController,
    cursor: Vec2,
    input_manager: Arc<dyn InputManager>,
    _core: CoreHandle,
}
