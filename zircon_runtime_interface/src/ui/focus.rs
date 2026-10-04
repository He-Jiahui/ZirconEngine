use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::tree::UiTree;

/// Controls whether a focusable node can receive pointer and keyboard focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiFocusMode {
    None,
    Click,
    #[default]
    All,
}

impl UiFocusMode {
    pub const fn allows_pointer_focus(self) -> bool {
        !matches!(self, Self::None)
    }

    pub const fn allows_tab_focus(self) -> bool {
        matches!(self, Self::All)
    }
}

/// Identifies why a node became focused so visual policy can distinguish keyboard focus rings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiFocusCause {
    Pointer,
    #[default]
    Navigation,
    Programmatic,
    Restore,
}

impl UiFocusCause {
    pub const fn focus_visible(self) -> UiFocusVisible {
        match self {
            Self::Navigation => UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation),
            Self::Pointer => UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction),
            Self::Programmatic | Self::Restore => {
                UiFocusVisible::hidden(UiFocusVisibleReason::Programmatic)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusVisibleReason {
    #[default]
    Initial,
    KeyboardNavigation,
    PointerInteraction,
    Programmatic,
    DisabledOrHidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusVisible {
    pub visible: bool,
    pub reason: UiFocusVisibleReason,
}

impl UiFocusVisible {
    pub const fn visible(reason: UiFocusVisibleReason) -> Self {
        Self {
            visible: true,
            reason,
        }
    }

    pub const fn hidden(reason: UiFocusVisibleReason) -> Self {
        Self {
            visible: false,
            reason,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputFocus {
    pub focused: Option<UiNodeId>,
    pub previous: Option<UiNodeId>,
    pub pending_autofocus: Option<UiNodeId>,
    pub focus_visible: UiFocusVisible,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusChangeReason {
    #[default]
    Input,
    Navigation,
    Programmatic,
    Autofocus,
    Clear,
    Disabled,
    Hidden,
    Despawned,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusChangeEvent {
    pub previous: Option<UiNodeId>,
    pub current: Option<UiNodeId>,
    pub reason: UiFocusChangeReason,
    pub visible: UiFocusVisible,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusedInputKind {
    #[default]
    Keyboard,
    Text,
    Ime,
    Navigation,
    Pointer,
    AccessibilityAction,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusedInput {
    pub focused: UiNodeId,
    pub kind: UiFocusedInputKind,
    pub route: Vec<UiNodeId>,
    pub handled_by: Option<UiNodeId>,
    pub accepted: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusContract {
    pub focusable: bool,
    #[serde(default)]
    pub mode: UiFocusMode,
    pub autofocus: bool,
    pub restore_on_close: bool,
    pub focus_visible: Option<UiFocusVisible>,
}

impl UiFocusContract {
    pub const fn allows_pointer_focus(&self) -> bool {
        self.focusable && self.mode.allows_pointer_focus()
    }

    pub const fn allows_tab_focus(&self) -> bool {
        self.focusable && self.mode.allows_tab_focus()
    }
}

/// Returns the stable Tab traversal order for the reachable retained tree.
///
/// Authored tab indices sort before default pre-order candidates; equal indices retain pre-order.
pub fn focus_chain(tree: &UiTree) -> Vec<UiNodeId> {
    let mut default_candidates = Vec::new();
    let mut indexed_candidates = Vec::new();
    let mut visited = HashSet::with_capacity(tree.nodes.len());
    let mut pre_order = 0usize;

    for root in &tree.roots {
        collect_focus_candidates(
            tree,
            *root,
            true,
            &mut visited,
            &mut pre_order,
            &mut default_candidates,
            &mut indexed_candidates,
        );
    }

    finish_focus_chain(default_candidates, indexed_candidates)
}

struct UiFocusChainCandidate {
    node_id: UiNodeId,
    tab_index: crate::ui::navigation::UiTabIndex,
    pre_order: usize,
}

fn finish_focus_chain(
    mut default_candidates: Vec<UiNodeId>,
    mut indexed_candidates: Vec<UiFocusChainCandidate>,
) -> Vec<UiNodeId> {
    indexed_candidates.sort_by_key(|candidate| (candidate.tab_index.order, candidate.pre_order));
    if indexed_candidates.is_empty() {
        return default_candidates;
    }

    let mut chain = Vec::with_capacity(indexed_candidates.len() + default_candidates.len());
    chain.extend(
        indexed_candidates
            .into_iter()
            .map(|candidate| candidate.node_id),
    );
    chain.append(&mut default_candidates);
    chain
}

fn collect_focus_candidates(
    tree: &UiTree,
    node_id: UiNodeId,
    ancestors_render_visible: bool,
    visited: &mut HashSet<UiNodeId>,
    pre_order: &mut usize,
    default_candidates: &mut Vec<UiNodeId>,
    indexed_candidates: &mut Vec<UiFocusChainCandidate>,
) {
    if !visited.insert(node_id) {
        return;
    }
    let Some(node) = tree.node(node_id) else {
        return;
    };

    let render_visible = ancestors_render_visible && node.is_render_visible();
    let tab_index = node.navigation.tab_index;
    if render_visible
        && node.state_flags.enabled
        && node.focus.allows_tab_focus()
        && tab_index.map(|index| index.tabbable).unwrap_or(true)
    {
        if let Some(tab_index) = tab_index {
            indexed_candidates.push(UiFocusChainCandidate {
                node_id,
                tab_index,
                pre_order: *pre_order,
            });
        } else {
            default_candidates.push(node_id);
        }
    }
    *pre_order = pre_order.saturating_add(1);

    for child in &node.children {
        collect_focus_candidates(
            tree,
            *child,
            render_visible,
            visited,
            pre_order,
            default_candidates,
            indexed_candidates,
        );
    }
}

#[cfg(test)]
#[path = "tests/focus_performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "focus/tests/focus_tests.rs"]
mod focus_tests;
