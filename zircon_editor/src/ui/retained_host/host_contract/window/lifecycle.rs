use std::cell::RefCell;
use std::rc::Rc;

use winit::event_loop::EventLoop;

use super::constants::{DEFAULT_HOST_WINDOW_HEIGHT, DEFAULT_HOST_WINDOW_WIDTH};
use super::handle::HostWindowHandle;
use super::metadata::platform_error;
use super::UiHostWindow;
use crate::ui::retained_host::host_contract::globals::{HostContractGlobal, HostContractState};
use crate::ui::retained_host::primitives::{CloseRequestResponse, PhysicalSize, PlatformError};

impl UiHostWindow {
    pub(crate) fn new() -> Result<Self, PlatformError> {
        Ok(Self {
            state: Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
                DEFAULT_HOST_WINDOW_WIDTH,
                DEFAULT_HOST_WINDOW_HEIGHT,
            )))),
        })
    }

    pub(crate) fn clone_strong(&self) -> Self {
        self.clone()
    }

    pub(crate) fn show(&self) -> Result<(), PlatformError> {
        self.state.borrow_mut().window_visible = true;
        Ok(())
    }

    pub(crate) fn hide(&self) -> Result<(), PlatformError> {
        self.state.borrow_mut().window_visible = false;
        Ok(())
    }

    pub(crate) fn run(&self) -> Result<(), PlatformError> {
        let event_loop = EventLoop::new().map_err(platform_error)?;
        let app = super::event_loop::UiHostWindowEventLoop::new(self.clone_strong());
        event_loop.run_app(app).map_err(platform_error)
    }

    pub(crate) fn window(&self) -> HostWindowHandle {
        HostWindowHandle {
            state: self.state.clone(),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn close_requested_response(
        &self,
    ) -> CloseRequestResponse {
        let callback = self.state.borrow().close_requested.clone();
        callback
            .as_ref()
            .map(|callback| callback())
            .unwrap_or(CloseRequestResponse::HideWindow)
    }

    pub(crate) fn global<T>(&self) -> T
    where
        T: HostContractGlobal,
    {
        T::from_state(self.state.clone())
    }

    pub(crate) fn request_exit(&self) {
        let mut state = self.state.borrow_mut();
        state.window_visible = false;
        state.exit_requested = true;
    }

    pub(crate) fn set_exit_after_first_presented_frame(&self, exit: bool) {
        self.state.borrow_mut().exit_after_first_presented_frame = exit;
    }

    pub(in crate::ui::retained_host::host_contract) fn exit_after_first_presented_frame(
        &self,
    ) -> bool {
        self.state.borrow().exit_after_first_presented_frame
    }

    #[cfg(test)]
    pub(crate) fn exit_requested_for_test(&self) -> bool {
        self.state.borrow().exit_requested
    }
}
