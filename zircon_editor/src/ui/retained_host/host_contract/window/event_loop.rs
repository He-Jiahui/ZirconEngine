use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

use super::super::presenter::{HostChromePresenter, HostPresenterBackend};
use super::super::redraw::HostRedrawRequest;
use super::UiHostWindow;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

mod events;
mod input;
mod lifecycle;
mod platform_input;
mod redraw;

pub(in crate::ui::retained_host::host_contract) struct UiHostWindowEventLoop {
    host: UiHostWindow,
    window: Option<Arc<dyn Window>>,
    presenter: Option<Box<dyn HostChromePresenter>>,
    presenter_backend: Option<HostPresenterBackend>,
    last_pointer_position: Option<(f32, f32)>,
    pending_redraw: HostRedrawRequest,
    ime_allowed: bool,
    current_modifiers: ModifiersState,
    next_input_sequence: u64,
}

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn new(host: UiHostWindow) -> Self {
        Self {
            host,
            window: None,
            presenter: None,
            presenter_backend: None,
            last_pointer_position: None,
            pending_redraw: HostRedrawRequest::full_frame_for_scenario(
                UiPerfScenario::Startup,
                true,
            ),
            ime_allowed: false,
            current_modifiers: ModifiersState::empty(),
            next_input_sequence: 1,
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

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.about_to_wait_impl(event_loop);
    }
}
