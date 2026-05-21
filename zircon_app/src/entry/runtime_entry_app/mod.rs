mod application_handler;
mod config;
mod construct;
mod converters;
mod device_events;
mod event_loop_policy;
mod file_drag_drop;
mod frame_loop;
#[cfg(feature = "gamepad-gilrs")]
mod gamepad;
mod host_requests;
mod ime_input;
mod keyboard_input;
mod pointer_input;
mod surface_present;
mod window_attributes;
mod window_creation;
mod window_events;
mod window_lifecycle;
mod window_surface;

use std::sync::Arc;

use winit::window::Window;
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowLifecyclePolicy};
use zircon_runtime::platform::EventLoopPolicy;
use zircon_runtime_interface::{ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1};

use super::runtime_library::RuntimeSession;
use crate::runtime_presenter::SoftbufferRuntimePresenter;

pub(in crate::entry) use config::RuntimeEntryAppConfig;

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<dyn Window>>,
    window_descriptor: WindowDescriptor,
    event_loop_policy: EventLoopPolicy,
    window_lifecycle_policy: WindowLifecyclePolicy,
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
