use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiFocusState, UiNavigationState},
    tree::UiTree,
};

use crate::ui::v2::UiV2RuntimeStyleIndex;

use super::{
    UiSurface, UiSurfaceClipboardTransferSnapshot, UiSurfaceComponentStateStore,
    UiSurfaceInputState, UiSurfaceInvalidationState,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiSurfaceMutationDomains {
    pub(crate) tree: bool,
    pub(crate) focus: bool,
    pub(crate) input: bool,
    pub(crate) component_states: bool,
    pub(crate) navigation: bool,
}

impl UiSurfaceMutationDomains {
    pub(crate) const fn binding_targets() -> Self {
        Self {
            tree: true,
            focus: true,
            input: true,
            component_states: true,
            navigation: true,
        }
    }
}

#[derive(Default)]
pub(crate) struct UiSurfaceMutationSnapshot {
    tree: Option<UiTreeMutationSnapshot>,
    focus: Option<UiFocusState>,
    input: Option<UiSurfaceInputState>,
    component_states: Option<UiSurfaceComponentStateStore>,
    navigation: Option<UiNavigationState>,
    clipboard_transfers: Option<UiSurfaceClipboardTransferSnapshot>,
}

impl UiSurfaceMutationSnapshot {
    pub(crate) fn capture(surface: &UiSurface, domains: UiSurfaceMutationDomains) -> Self {
        Self {
            tree: domains
                .tree
                .then(|| UiTreeMutationSnapshot::capture(surface)),
            focus: domains.focus.then(|| surface.focus.clone()),
            input: domains.input.then(|| surface.input.clone()),
            component_states: domains
                .component_states
                .then(|| surface.component_states.clone()),
            navigation: domains.navigation.then(|| surface.navigation.clone()),
            clipboard_transfers: (domains.tree || domains.focus || domains.input)
                .then(|| surface.clipboard_transfers.snapshot()),
        }
    }

    pub(crate) fn restore(self, surface: &mut UiSurface) {
        if let Some(tree) = self.tree {
            tree.restore(surface);
        }
        if let Some(focus) = self.focus {
            surface.focus = focus;
        }
        if let Some(input) = self.input {
            surface.input = input;
        }
        if let Some(component_states) = self.component_states {
            surface.component_states = component_states;
        }
        if let Some(navigation) = self.navigation {
            surface.navigation = navigation;
        }
        if let Some(clipboard_transfers) = self.clipboard_transfers {
            surface.clipboard_transfers.restore(clipboard_transfers);
        }
    }
}

struct UiTreeMutationSnapshot {
    tree: UiTree,
    runtime_style: UiV2RuntimeStyleIndex,
    invalidation: UiSurfaceInvalidationState,
    dirty_node_ids: BTreeSet<UiNodeId>,
}

impl UiTreeMutationSnapshot {
    fn capture(surface: &UiSurface) -> Self {
        Self {
            tree: surface.tree.clone(),
            runtime_style: surface.runtime_style.clone(),
            invalidation: surface.invalidation.clone(),
            dirty_node_ids: surface.dirty_node_ids.clone(),
        }
    }

    fn restore(self, surface: &mut UiSurface) {
        surface.tree = self.tree;
        surface.runtime_style = self.runtime_style;
        surface.invalidation = self.invalidation;
        surface.dirty_node_ids = self.dirty_node_ids;
    }
}
