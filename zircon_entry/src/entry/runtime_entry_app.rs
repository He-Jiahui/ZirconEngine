mod application_handler;
mod construct;

use std::sync::Arc;

use winit::window::Window;
use zircon_core::CoreHandle;
use zircon_graphics::ViewportController;
use zircon_manager::InputManager;
use zircon_math::Vec2;
use zircon_scene::LevelSystem;

use crate::runtime_presenter::{RenderServerRuntimeBridge, SoftbufferRuntimePresenter};

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<Window>>,
    presenter: Option<SoftbufferRuntimePresenter>,
    render_bridge: RenderServerRuntimeBridge,
    level: LevelSystem,
    viewport_controller: ViewportController,
    cursor: Vec2,
    input_manager: Arc<dyn InputManager>,
    _core: CoreHandle,
}
