mod application_handler;
mod construct;
mod window_surface;

use std::sync::Arc;

use winit::window::Window;
use zircon_runtime_interface::{ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1};

use super::runtime_library::RuntimeSession;
use crate::runtime_presenter::SoftbufferRuntimePresenter;

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<dyn Window>>,
    presenter: Option<SoftbufferRuntimePresenter>,
    surface_present_enabled: bool,
    surface_present_failed: bool,
    surface_present_attempted: bool,
    session: RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    viewport_size: ZrRuntimeViewportSizeV1,
}
