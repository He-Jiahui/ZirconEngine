use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ui::retained_host::primitives::{CloseRequestResponse, PhysicalSize, PlatformError};
use winit::event_loop::EventLoop;
use zircon_runtime_interface::ui::dispatch::{
    UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiWindowId,
};

use super::globals::{HostContractGlobal, HostContractState};

mod diagnostics;
mod event_loop;
mod handle;
mod presentation;
mod redraw;
mod template_hover;
#[cfg(test)]
mod test_support;
mod text_input;

pub(crate) use handle::{HostWindowHandle, HostWindowSnapshot};

// Keep the first editor frame on the same design canvas used by the workbench
// style studies while the retained shell is componentized.
const DEFAULT_HOST_WINDOW_WIDTH: u32 = 1672;
const DEFAULT_HOST_WINDOW_HEIGHT: u32 = 941;
const NATIVE_HOST_WINDOW_ID: &str = "editor.main";

#[derive(Clone)]
pub(crate) struct UiHostWindow {
    state: Rc<RefCell<HostContractState>>,
}

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
        let app = event_loop::UiHostWindowEventLoop::new(self.clone_strong());
        event_loop.run_app(app).map_err(platform_error)
    }

    pub(crate) fn window(&self) -> HostWindowHandle {
        HostWindowHandle {
            state: self.state.clone(),
        }
    }

    fn close_requested_response(&self) -> CloseRequestResponse {
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

    #[cfg(test)]
    pub(crate) fn exit_requested_for_test(&self) -> bool {
        self.state.borrow().exit_requested
    }
}

fn native_input_metadata(sequence: u64) -> UiInputEventMetadata {
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default();
    let mut metadata = UiInputEventMetadata::new(
        UiInputTimestamp::from_micros(micros),
        UiInputSequence::new(sequence),
    );
    metadata.window_id = Some(UiWindowId::new(NATIVE_HOST_WINDOW_ID));
    metadata
}

fn platform_error(error: impl std::fmt::Display) -> PlatformError {
    PlatformError::Other(error.to_string())
}

#[cfg(test)]
mod tests;
