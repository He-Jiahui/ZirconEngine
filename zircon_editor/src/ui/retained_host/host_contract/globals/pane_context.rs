mod callbacks;
mod setters;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::state::{HostContractGlobal, HostContractState};

#[derive(Clone, Copy)]
pub(crate) struct HostAssetSurfaceInteractionState {
    pub(crate) tree_hovered_index: i32,
    pub(crate) tree_scroll_px: f32,
    pub(crate) content_hovered_index: i32,
    pub(crate) content_scroll_px: f32,
    pub(crate) references_hovered_index: i32,
    pub(crate) references_scroll_px: f32,
    pub(crate) used_by_hovered_index: i32,
    pub(crate) used_by_scroll_px: f32,
}

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
