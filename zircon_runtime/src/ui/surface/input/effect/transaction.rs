use zircon_runtime_interface::ui::dispatch::{
    UiDispatchEffect, UiDispatchRejectedEffect, UiDragDropEffectKind, UiInputDiagnosticsMode,
    UiInputDispatchResult,
};

use crate::ui::surface::{UiSurface, UiSurfaceMutationDomains, UiSurfaceMutationSnapshot};

pub(super) struct UiInputTransaction {
    atomic: bool,
    effect_count: usize,
    base_generation: u64,
    snapshot: UiSurfaceMutationSnapshot,
}

impl UiInputTransaction {
    pub(super) fn prepare(surface: &UiSurface, effects: &[UiDispatchEffect]) -> Self {
        let write_set = UiInputWriteSet::for_effects(effects);
        let atomic = effects.len() > 1
            || effects
                .iter()
                .any(|effect| effect_requires_atomic_snapshot(surface, effect));
        Self {
            atomic,
            effect_count: effects.len(),
            base_generation: surface.invalidation.generations().generation,
            snapshot: UiSurfaceMutationSnapshot::capture(
                surface,
                atomic
                    .then(|| write_set.mutation_domains())
                    .unwrap_or_default(),
            ),
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
        diagnostics_mode: UiInputDiagnosticsMode,
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
        if diagnostics_mode.captures_full_trace() {
            result.diagnostics.notes.push(format!(
                "input_transaction=aborted base_generation={} failed_effect={failed_effect_index}",
                self.base_generation
            ));
        }
    }

    pub(super) fn commit(
        self,
        result: &mut UiInputDispatchResult,
        diagnostics_mode: UiInputDiagnosticsMode,
    ) {
        if self.atomic && diagnostics_mode.captures_full_trace() {
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
    const fn mutation_domains(self) -> UiSurfaceMutationDomains {
        UiSurfaceMutationDomains {
            tree: self.tree,
            focus: self.focus,
            input: self.input,
            component_states: self.component_states,
            navigation: self.navigation,
        }
    }

    fn for_effects(effects: &[UiDispatchEffect]) -> Self {
        let mut write_set = Self {
            // Every successful effect drains deferred focus/IME lifecycle state.
            input: !effects.is_empty(),
            ..Self::default()
        };
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

fn effect_requires_atomic_snapshot(surface: &UiSurface, effect: &UiDispatchEffect) -> bool {
    match effect {
        UiDispatchEffect::DragDrop {
            kind: UiDragDropEffectKind::Update,
            target,
            pointer_id,
            session_id,
            ..
        } => {
            // A steady-target update validates ownership before mutation and leaves component
            // flags unchanged, so no later fallible style invalidation needs rollback.
            let steady_target_update = surface.input.drag_drop.as_ref().is_some_and(|drag| {
                drag.target == *target
                    && drag.pointer_id == *pointer_id
                    && session_id.is_none_or(|session_id| session_id == drag.session_id)
            });
            !steady_target_update
        }
        UiDispatchEffect::DragDrop { .. }
        | UiDispatchEffect::Popup { .. }
        | UiDispatchEffect::DismissTransientUi { .. } => true,
        _ => false,
    }
}
