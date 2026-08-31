use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

use super::super::presenter::{HostChromePresenter, HostPresenterBackend};
use super::super::redraw::HostRedrawRequest;
use super::UiHostWindow;
use crate::ui::retained_host::ui_perf::UiPerfScenario;
#[cfg(feature = "profiling")]
use input_outcome::UiInputOutcomeTracker;
#[cfg(feature = "profiling")]
use profile_capture::UiProfileWarmupState;

mod events;
mod input;
mod input_outcome;
mod lifecycle;
mod platform_input;
mod pointer_move_mailbox;
mod profile_capture;
mod redraw;

use pointer_move_mailbox::UiIdlePointerMoveMailbox;

const SURFACE_PRESENT_RETRY_BASE_DELAY: Duration = Duration::from_millis(8);
const SURFACE_PRESENT_RETRY_MAX_DELAY: Duration = Duration::from_millis(250);

pub(in crate::ui::retained_host::host_contract) struct UiHostWindowEventLoop {
    host: UiHostWindow,
    window: Option<Arc<dyn Window>>,
    presenter: Option<Box<dyn HostChromePresenter>>,
    presenter_backend: Option<HostPresenterBackend>,
    shared_gpu_presenter_active: bool,
    last_pointer_position: Option<(f32, f32)>,
    pending_idle_pointer_move: UiIdlePointerMoveMailbox,
    pressed_mouse_button_count: u8,
    pending_redraw: HostRedrawRequest,
    pending_surface_present_retry: HostRedrawRequest,
    pending_surface_present_retry_deadline: Option<Instant>,
    surface_present_retry_attempt: u8,
    pending_presenter_resize: Option<(u32, u32)>,
    runtime_presenter_upgrade_attempted: bool,
    runtime_presenter_upgrade_poll_deadline: Option<Instant>,
    ime_allowed: bool,
    current_modifiers: ModifiersState,
    next_input_sequence: u64,
    profile_artifact_capture_requested: bool,
    #[cfg(feature = "profiling")]
    input_outcomes: UiInputOutcomeTracker,
    #[cfg(feature = "profiling")]
    profile_warmup: UiProfileWarmupState,
}

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn new(host: UiHostWindow) -> Self {
        Self {
            host,
            window: None,
            presenter: None,
            presenter_backend: None,
            shared_gpu_presenter_active: false,
            last_pointer_position: None,
            pending_idle_pointer_move: UiIdlePointerMoveMailbox::default(),
            pressed_mouse_button_count: 0,
            pending_redraw: HostRedrawRequest::full_frame_for_scenario(
                UiPerfScenario::Startup,
                true,
            ),
            pending_surface_present_retry: HostRedrawRequest::None,
            pending_surface_present_retry_deadline: None,
            surface_present_retry_attempt: 0,
            pending_presenter_resize: None,
            runtime_presenter_upgrade_attempted: false,
            runtime_presenter_upgrade_poll_deadline: None,
            ime_allowed: false,
            current_modifiers: ModifiersState::empty(),
            next_input_sequence: 1,
            profile_artifact_capture_requested: false,
            #[cfg(feature = "profiling")]
            input_outcomes: UiInputOutcomeTracker::default(),
            #[cfg(feature = "profiling")]
            profile_warmup: UiProfileWarmupState::from_env(),
        }
    }
}

impl ApplicationHandler for UiHostWindowEventLoop {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.can_create_surfaces_impl(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.window_event_impl(event_loop, event);
    }

    fn proxy_wake_up(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.flush_pending_idle_pointer_move();
        if self.host.take_background_event_wake() {
            self.host.request_maintenance_frame_update();
            self.drain_external_redraw_request();
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.about_to_wait_impl(event_loop);
    }
}
