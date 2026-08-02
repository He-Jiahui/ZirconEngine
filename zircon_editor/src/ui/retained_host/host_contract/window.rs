use std::cell::RefCell;
use std::rc::Rc;

use super::globals::HostContractState;

mod capture;
mod constants;
mod diagnostics;
mod event_loop;
mod failure;
mod handle;
mod lifecycle;
mod metadata;
mod presentation;
mod redraw;
mod template_hover;
#[cfg(test)]
mod test_support;
mod text_input;

pub(crate) use handle::{HostWindowHandle, HostWindowSnapshot};

#[derive(Clone)]
pub(crate) struct UiHostWindow {
    state: Rc<RefCell<HostContractState>>,
    fatal_failure: Rc<RefCell<Option<failure::EditorHostWindowFailure>>>,
}

#[cfg(test)]
mod tests;
