mod snapshot;

use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::retained_host::primitives::{
    CloseRequestResponse, PhysicalPosition, PhysicalSize, PlatformError,
};

use super::super::globals::HostContractState;
use super::super::presenter::paint_host_presentation_snapshot;
use super::presentation::host_presentation_from_state;
pub(crate) use snapshot::HostWindowSnapshot;

#[derive(Clone)]
pub(crate) struct HostWindowHandle {
    pub(in crate::ui::retained_host::host_contract) state: Rc<RefCell<HostContractState>>,
}

impl HostWindowHandle {
    pub(crate) fn set_position(&self, position: PhysicalPosition) {
        let mut state = self.state.borrow_mut();
        if state.window_position != position {
            state.window_position = position;
        }
    }

    pub(crate) fn set_size(&self, size: PhysicalSize) {
        let mut state = self.state.borrow_mut();
        if state.window_size != size {
            state.window_size = size;
        }
    }

    pub(crate) fn position(&self) -> PhysicalPosition {
        self.state.borrow().window_position.clone()
    }

    pub(crate) fn size(&self) -> PhysicalSize {
        self.state.borrow().window_size.clone()
    }

    pub(crate) fn set_scale_factor(&self, scale_factor: f32) {
        self.state
            .borrow_mut()
            .set_window_scale_factor(scale_factor);
    }

    pub(crate) fn scale_factor(&self) -> f32 {
        self.state.borrow().window_scale_factor()
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.state.borrow().window_visible
    }

    pub(crate) fn set_maximized(&self, maximized: bool) {
        self.state.borrow_mut().window_maximized = maximized;
    }

    pub(crate) fn is_maximized(&self) -> bool {
        self.state.borrow().window_maximized
    }

    pub(crate) fn on_close_requested(&self, callback: impl Fn() -> CloseRequestResponse + 'static) {
        self.state.borrow_mut().close_requested = Some(Rc::new(callback));
    }

    pub(crate) fn take_snapshot(&self) -> Result<HostWindowSnapshot, PlatformError> {
        let state = self.state.borrow();
        let presentation = host_presentation_from_state(&state);
        let frame = paint_host_presentation_snapshot(
            state.window_size.width,
            state.window_size.height,
            &presentation,
        );
        Ok(HostWindowSnapshot::from_rgba_frame(frame))
    }
}
