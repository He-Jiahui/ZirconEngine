mod application_handler;
mod config;
mod construct;
mod converters;
mod device_events;
mod event_dispatch;
mod event_loop_policy;
mod failure;
mod file_drag_drop;
mod frame_capture;
mod frame_loop;
#[cfg(feature = "gamepad-gilrs")]
mod gamepad;
mod host_requests;
mod ime_input;
mod keyboard_input;
mod mvp_input_probe;
mod pointer_input;
mod runtime_product_diagnostics;
mod surface_present;
mod window_attributes;
mod window_creation;
mod window_events;
mod window_lifecycle;
mod window_surface;

use std::{path::PathBuf, sync::Arc};

use winit::dpi::PhysicalPosition;
use winit::window::Window;
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowLifecyclePolicy};
use zircon_runtime_interface::{ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1};

use super::runtime_library::RuntimeSession;
use crate::runtime_presenter::SoftbufferRuntimePresenter;
use event_loop_policy::RuntimeFrameCadence;

pub(in crate::entry) use config::RuntimeEntryAppConfig;
pub(in crate::entry) use failure::RuntimeEntryAppFailureState;

pub(super) struct RuntimeEntryApp {
    window: Option<Arc<dyn Window>>,
    window_descriptor: WindowDescriptor,
    frame_cadence: RuntimeFrameCadence,
    window_lifecycle_policy: WindowLifecyclePolicy,
    presenter: Option<SoftbufferRuntimePresenter>,
    surface_present_enabled: bool,
    surface_present_failed: bool,
    surface_present_attempted: bool,
    exit_after_first_presented_frame: bool,
    first_frame_capture_path: Option<PathBuf>,
    require_persisted_scene_diagnostics: bool,
    first_frame_capture_written: bool,
    first_frame_product_diagnostics_emitted: bool,
    mvp_input_probe_submitted: bool,
    // `run_app` consumes the handler, so this slot carries the first terminal
    // callback failure back to EntryRunner after the event loop ends.
    failure_state: RuntimeEntryAppFailureState,
    session: RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    viewport_size: ZrRuntimeViewportSizeV1,
    last_pointer_position: Option<PhysicalPosition<f64>>,
    #[cfg(feature = "gamepad-gilrs")]
    gamepads: Option<gilrs::Gilrs>,
    #[cfg(feature = "gamepad-gilrs")]
    gamepad_connections_announced: bool,
    #[cfg(feature = "gamepad-gilrs")]
    gamepad_rumble_effects: Option<gamepad::RunningRumbleEffects>,
}
