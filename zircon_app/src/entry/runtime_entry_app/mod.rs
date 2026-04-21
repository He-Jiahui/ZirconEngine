mod application_handler;
mod camera_controller;
mod construct;

use std::sync::Arc;

use winit::window::Window;
use zircon_runtime::core::framework::input::InputManager;
use zircon_runtime::core::math::Vec2;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::scene::LevelSystem;

use self::camera_controller::RuntimeCameraController;
use crate::runtime_presenter::{RenderFrameworkRuntimeBridge, SoftbufferRuntimePresenter};

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<dyn Window>>,
    presenter: Option<SoftbufferRuntimePresenter>,
    render_bridge: RenderFrameworkRuntimeBridge,
    level: LevelSystem,
    selected_node: Option<u64>,
    camera_controller: RuntimeCameraController,
    cursor: Vec2,
    input_manager: Arc<dyn InputManager>,
    _core: CoreHandle,
}
