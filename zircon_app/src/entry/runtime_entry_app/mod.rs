mod application_handler;
mod construct;
#[cfg(feature = "gamepad-gilrs")]
mod gamepad;
mod window_attributes;
mod window_surface;

use std::sync::Arc;

use winit::window::Window;
use zircon_runtime::core::framework::window::WindowDescriptor;
use zircon_runtime_interface::{ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1};

use super::runtime_library::RuntimeSession;
use crate::runtime_presenter::SoftbufferRuntimePresenter;

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<dyn Window>>,
    window_descriptor: WindowDescriptor,
    presenter: Option<SoftbufferRuntimePresenter>,
    surface_present_enabled: bool,
    surface_present_failed: bool,
    surface_present_attempted: bool,
    session: RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    viewport_size: ZrRuntimeViewportSizeV1,
    #[cfg(feature = "gamepad-gilrs")]
    gamepads: Option<gilrs::Gilrs>,
    #[cfg(feature = "gamepad-gilrs")]
    gamepad_connections_announced: bool,
}
