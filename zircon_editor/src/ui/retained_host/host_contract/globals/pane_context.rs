mod callbacks;
mod setters;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::state::{HostContractGlobal, HostContractState};

pub(crate) struct PaneSurfaceHostContext<'a> {
    state: Rc<RefCell<HostContractState>>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> HostContractGlobal for PaneSurfaceHostContext<'a> {
    fn from_state(state: Rc<RefCell<HostContractState>>) -> Self {
        Self {
            state,
            _lifetime: PhantomData,
        }
    }
}
