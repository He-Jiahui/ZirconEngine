use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchEffect, UiDispatchRejectedEffect, UiInputDispatchResult},
    event_ui::UiNodeId,
    surface::{UiFocusState, UiNavigationState},
    tree::UiTree,
};

use crate::ui::{
    surface::{
        UiSurface, UiSurfaceComponentStateStore, UiSurfaceInputState, UiSurfaceInvalidationState,
    },
    v2::UiV2RuntimeStyleIndex,
};

pub(super) struct UiInputTransaction {
    atomic: bool,
    effect_count: usize,
    base_generation: u64,
    snapshot: UiInputMutationSnapshot,
}

impl UiInputTransaction {
    pub(super) fn prepare(surface: &UiSurface, effects: &[UiDispatchEffect]) -> Self {
        let write_set = UiInputWriteSet::for_effects(effects);
        let atomic = effects.len() > 1 || effects.iter().any(effect_is_composite);
        Self {
            atomic,
            effect_count: effects.len(),
            base_generation: surface.invalidation.generations().generation,
            snapshot: UiInputMutationSnapshot::capture(surface, atomic.then_some(write_set)),
        }
    }

    pub(super) const fn is_atomic(&self) -> bool {
        self.atomic
    }

    pub(super) fn abort(
        self,
        surface: &mut UiSurface,
        result: &mut UiInputDispatchResult,
        failed_effect_index: usize,
        failed_reason: String,
    ) {
        self.snapshot.restore(surface);
        let rejected_effects = result
            .reply
            .effects
            .iter()
            .cloned()
            .enumerate()
            .map(|(effect_index, effect)| UiDispatchRejectedEffect {
                effect_index,
                effect,
                reason: if effect_index == failed_effect_index {
                    failed_reason.clone()
                } else {
                    format!(
                        "input transaction aborted because effect {failed_effect_index} was rejected: {failed_reason}"
                    )
                },
            })
            .collect();
        result.applied_effects.clear();
        result.rejected_effects = rejected_effects;
        result.host_requests.clear();
        result.component_events.clear();
        result.diagnostics.route_target = result.reply.handler;
        result.diagnostics.notes.push(format!(
            "input_transaction=aborted base_generation={} failed_effect={failed_effect_index}",
            self.base_generation
        ));
    }

    pub(super) fn commit(self, result: &mut UiInputDispatchResult) {
        if self.atomic {
            result.diagnostics.notes.push(format!(
                "input_transaction=committed base_generation={} effects={}",
                self.base_generation, self.effect_count
            ));
        }
    }
}

#[derive(Clone, Copy, Default)]
struct UiInputWriteSet {
    tree: bool,
    focus: bool,
    input: bool,
    component_states: bool,
    navigation: bool,
}

impl UiInputWriteSet {
    fn for_effects(effects: &[UiDispatchEffect]) -> Self {
        let mut write_set = Self::default();
        for effect in effects {
            write_set.include(effect);
        }
        write_set
    }

    fn include(&mut self, effect: &UiDispatchEffect) {
        match effect {
            UiDispatchEffect::SetFocus { .. } | UiDispatchEffect::ClearFocus { .. } => {
                self.tree = true;
                self.focus = true;
                self.input = true;
                self.component_states = true;
                self.navigation = true;
            }
            UiDispatchEffect::CapturePointer { .. }
            | UiDispatchEffect::ReleasePointerCapture { .. } => {
                self.focus = true;
                self.input = true;
            }
            UiDispatchEffect::LockPointer { .. }
            | UiDispatchEffect::UnlockPointer { .. }
            | UiDispatchEffect::UseHighPrecisionPointer { .. }
            | UiDispatchEffect::Tooltip { .. }
            | UiDispatchEffect::RequestInputMethod { .. } => {
                self.input = true;
            }
            UiDispatchEffect::DragDrop { .. } => {
                self.tree = true;
                self.focus = true;
                self.input = true;
                self.component_states = true;
            }
            UiDispatchEffect::RequestNavigation { .. } => {
                self.tree = true;
                self.focus = true;
                self.input = true;
                self.component_states = true;
                self.navigation = true;
            }
            UiDispatchEffect::Popup { .. } | UiDispatchEffect::DismissTransientUi { .. } => {
                self.tree = true;
                self.focus = true;
                self.input = true;
                self.component_states = true;
                self.navigation = true;
            }
            UiDispatchEffect::DirtyRedraw { .. } => {
                self.tree = true;
            }
            UiDispatchEffect::RequestClipboard { .. }
            | UiDispatchEffect::RequestLinkActivation { .. }
            | UiDispatchEffect::EmitComponentEvent { .. } => {}
        }
    }
}

fn effect_is_composite(effect: &UiDispatchEffect) -> bool {
    matches!(
        effect,
        UiDispatchEffect::DragDrop { .. }
            | UiDispatchEffect::Popup { .. }
            | UiDispatchEffect::DismissTransientUi { .. }
    )
}

#[derive(Default)]
struct UiInputMutationSnapshot {
    tree: Option<UiTreeMutationSnapshot>,
    focus: Option<UiFocusState>,
    input: Option<UiSurfaceInputState>,
    component_states: Option<UiSurfaceComponentStateStore>,
    navigation: Option<UiNavigationState>,
}

impl UiInputMutationSnapshot {
    fn capture(surface: &UiSurface, write_set: Option<UiInputWriteSet>) -> Self {
        let Some(write_set) = write_set else {
            return Self::default();
        };
        Self {
            tree: write_set
                .tree
                .then(|| UiTreeMutationSnapshot::capture(surface)),
            focus: write_set.focus.then(|| surface.focus.clone()),
            input: write_set.input.then(|| surface.input.clone()),
            component_states: write_set
                .component_states
                .then(|| surface.component_states.clone()),
            navigation: write_set.navigation.then(|| surface.navigation.clone()),
        }
    }

    fn restore(self, surface: &mut UiSurface) {
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
