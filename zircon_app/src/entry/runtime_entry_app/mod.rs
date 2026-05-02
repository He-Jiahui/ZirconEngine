mod application_handler;
mod construct;

use std::sync::Arc;

use winit::window::Window;
use zircon_runtime_interface::{ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1};

use super::runtime_library::RuntimeSession;
use crate::runtime_presenter::SoftbufferRuntimePresenter;

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<dyn Window>>,
    presenter: Option<SoftbufferRuntimePresenter>,
    session: RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    viewport_size: ZrRuntimeViewportSizeV1,
}
