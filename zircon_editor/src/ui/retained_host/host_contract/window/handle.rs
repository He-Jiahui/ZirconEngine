use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::retained_host::primitives::{
    CloseRequestResponse, PhysicalPosition, PhysicalSize, PlatformError,
};

use super::super::globals::HostContractState;
use super::super::paint_frame::HostRgbaFrame;
use super::super::presenter::paint_host_presentation_snapshot;
use super::presentation::host_presentation_from_state;

#[derive(Clone)]
pub(crate) struct HostWindowHandle {
    pub(super) state: Rc<RefCell<HostContractState>>,
}

impl HostWindowHandle {
    pub(crate) fn set_position(&self, position: PhysicalPosition) {
        self.state.borrow_mut().window_position = position;
    }

    pub(crate) fn set_size(&self, size: PhysicalSize) {
        self.state.borrow_mut().window_size = size;
    }

    pub(crate) fn size(&self) -> PhysicalSize {
        self.state.borrow().window_size.clone()
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

pub(crate) struct HostWindowSnapshot {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

impl HostWindowSnapshot {
    fn from_rgba_frame(frame: HostRgbaFrame) -> Self {
        Self {
            width: frame.width(),
            height: frame.height(),
            bytes: frame.into_bytes(),
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }
}
