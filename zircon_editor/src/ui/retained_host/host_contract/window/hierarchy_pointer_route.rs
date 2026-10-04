use std::cell::RefCell;
use std::rc::{Rc, Weak};

use super::super::globals::HostContractState;
use super::UiHostWindow;
use crate::ui::retained_host::host_contract::native_pointer::{
    route_pointer_move_to_pane, PanePointerTarget,
};

/// A callback may be stored in the host state, so it must retain only a weak source handle.
pub(crate) struct HierarchyPointerSource {
    state: Weak<RefCell<HostContractState>>,
}

impl UiHostWindow {
    pub(crate) fn hierarchy_pointer_source(&self) -> HierarchyPointerSource {
        HierarchyPointerSource {
            state: Rc::downgrade(&self.state),
        }
    }
}

impl HierarchyPointerSource {
    pub(crate) fn native_floating_window_id(&self) -> Option<String> {
        let state = self.state.upgrade()?;
        let state = state.borrow();
        let shell = &state.host_presentation.host_shell;
        if !shell.native_floating_window_mode || shell.native_floating_window_id.trim().is_empty() {
            return None;
        }
        Some(shell.native_floating_window_id.to_string())
    }

    /// Checks the same native pane route used for a move, without changing hover state.
    pub(crate) fn has_hierarchy_pointer_route(&self, x: f32, y: f32) -> bool {
        let Some(state) = self.state.upgrade() else {
            return false;
        };
        let generation = state.borrow().presentation_generation();
        route_pointer_move_to_pane(
            generation.structure(),
            generation.pane_interaction_state(),
            x,
            y,
        )
        .is_some_and(|route| matches!(route.target, PanePointerTarget::Hierarchy))
    }
}

#[cfg(test)]
#[path = "tests/hierarchy_pointer_route.rs"]
mod tests;
