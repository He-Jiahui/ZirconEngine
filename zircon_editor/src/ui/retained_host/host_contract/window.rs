use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::globals::HostContractState;
use super::presenter::RuntimeUiSurfacePresenterFactory;

mod capture;
mod constants;
mod diagnostics;
mod event_loop;
mod event_wake;
mod failure;
mod handle;
mod lifecycle;
mod metadata;
mod presentation;
mod redraw;
mod resize_reflow;
#[cfg(test)]
mod test_support;
mod text_input;

pub(crate) use handle::{HostWindowHandle, HostWindowSnapshot};

#[derive(Clone)]
pub(crate) struct UiHostWindow {
    state: Rc<RefCell<HostContractState>>,
    event_wake: event_wake::HostEventLoopWake,
    fatal_failure: Rc<RefCell<Option<failure::EditorHostWindowFailure>>>,
    runtime_presenter_factory: Rc<RefCell<Option<Arc<dyn RuntimeUiSurfacePresenterFactory>>>>,
    direct_viewport_products_active: Rc<Cell<bool>>,
}

#[cfg(test)]
mod tests;
