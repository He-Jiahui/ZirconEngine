use crate::ui::retained_host::host_contract::HierarchyPointerSource;
use crate::ui::workbench::layout::MainPageId;
use zircon_runtime_interface::ui::dispatch::{UiPointerId, UiPointerInputEvent};
use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};

use super::super::RetainedEditorHost;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum HierarchyTerminalReason {
    Released,
    Superseded,
    ForeignWindow,
    OutsidePane,
    UnroutedRelease,
    PointerCancel,
    SecondaryPress,
    Escape,
    FocusLost,
    WindowClose,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HierarchyActivePointer {
    source_window: Option<MainPageId>,
    pointer_id: UiPointerId,
    button: UiPointerButton,
    generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedPointerInput {
    source_window: Option<MainPageId>,
    pointer_id: UiPointerId,
    kind: UiPointerEventKind,
    button: Option<UiPointerButton>,
    motion_eligible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct HierarchyTerminalReceipt {
    pub(in crate::ui::retained_host::app) generation: u64,
    pub(in crate::ui::retained_host::app) reason: HierarchyTerminalReason,
}

#[derive(Default)]
pub(in crate::ui::retained_host::app) struct HierarchyInputOwner {
    active: Option<HierarchyActivePointer>,
    observed_input: Option<ObservedPointerInput>,
    next_generation: u64,
    terminal_count: u64,
    last_terminal: Option<HierarchyTerminalReceipt>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReleaseOwnership {
    Owner,
    ForeignSamePointer,
    DifferentPointer,
    NoOwner,
}

impl HierarchyInputOwner {
    pub(super) fn needs_observation(&self, pointer: &UiPointerInputEvent) -> bool {
        self.active.is_some()
            || (pointer.event.kind == UiPointerEventKind::Down
                && pointer.event.button == Some(UiPointerButton::Primary))
    }

    pub(in crate::ui::retained_host::app) fn terminal_count(&self) -> u64 {
        self.terminal_count
    }

    pub(in crate::ui::retained_host::app) fn last_terminal(
        &self,
    ) -> Option<&HierarchyTerminalReceipt> {
        self.last_terminal.as_ref()
    }

    pub(super) fn arm(&mut self, source_window: Option<MainPageId>) -> bool {
        let pointer_id = self
            .observed_input
            .as_ref()
            .filter(|input| {
                input.source_window == source_window
                    && input.kind == UiPointerEventKind::Down
                    && input.button == Some(UiPointerButton::Primary)
            })
            .map(|input| input.pointer_id)
            .unwrap_or_default();
        let Some(generation) = self.next_generation.checked_add(1) else {
            return false;
        };
        self.next_generation = generation;
        self.active = Some(HierarchyActivePointer {
            source_window,
            pointer_id,
            button: UiPointerButton::Primary,
            generation,
        });
        true
    }

    pub(super) fn admits_primary_press(&self, source_window: &Option<MainPageId>) -> bool {
        let Some(active) = self.active.as_ref() else {
            return true;
        };
        let Some(observed) = self.observed_input.as_ref() else {
            return false;
        };
        observed.source_window == *source_window
            && observed.pointer_id == active.pointer_id
            && observed.kind == UiPointerEventKind::Down
            && observed.button == Some(active.button)
    }

    pub(super) fn release_ownership(&self, source_window: &Option<MainPageId>) -> ReleaseOwnership {
        let Some(active) = self.active.as_ref() else {
            return ReleaseOwnership::NoOwner;
        };
        let pointer_id = self
            .observed_input
            .as_ref()
            .map(|input| input.pointer_id)
            .unwrap_or_default();
        if pointer_id != active.pointer_id {
            return ReleaseOwnership::DifferentPointer;
        }
        if source_window != &active.source_window {
            return ReleaseOwnership::ForeignSamePointer;
        }
        ReleaseOwnership::Owner
    }

    pub(super) fn owns_window(&self, window: &Option<MainPageId>) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| &active.source_window == window)
    }

    pub(super) fn owns_primary_release(
        &self,
        window: &Option<MainPageId>,
        pointer_id: UiPointerId,
    ) -> bool {
        self.active.as_ref().is_some_and(|active| {
            &active.source_window == window
                && active.pointer_id == pointer_id
                && active.button == UiPointerButton::Primary
        })
    }

    pub(super) fn has_active(&self) -> bool {
        self.active.is_some()
    }

    pub(super) fn admits_move(&self, source_window: &Option<MainPageId>) -> bool {
        let Some(active) = self.active.as_ref() else {
            return true;
        };
        self.observed_input.as_ref().is_some_and(|observed| {
            observed.source_window == *source_window
                && observed.pointer_id == active.pointer_id
                && observed.motion_eligible
        })
    }

    pub(super) fn note_ineligible_move(
        &mut self,
        source_window: Option<MainPageId>,
        pointer_id: UiPointerId,
    ) {
        self.observed_input = Some(ObservedPointerInput {
            source_window,
            pointer_id,
            kind: UiPointerEventKind::Move,
            button: None,
            motion_eligible: false,
        });
    }

    pub(super) fn needs_route_check(
        &self,
        window: &Option<MainPageId>,
        pointer: &UiPointerInputEvent,
    ) -> bool {
        self.needs_route_check_event(
            window,
            pointer.metadata.pointer_id.unwrap_or_default(),
            pointer.event.kind,
            pointer.event.button,
        )
    }

    pub(super) fn needs_move_route_check(
        &self,
        window: &Option<MainPageId>,
        pointer_id: UiPointerId,
    ) -> bool {
        self.needs_route_check_event(window, pointer_id, UiPointerEventKind::Move, None)
    }

    fn needs_route_check_event(
        &self,
        window: &Option<MainPageId>,
        pointer_id: UiPointerId,
        kind: UiPointerEventKind,
        button: Option<UiPointerButton>,
    ) -> bool {
        self.active.as_ref().is_some_and(|active| {
            &active.source_window == window
                && active.pointer_id == pointer_id
                && matches!(kind, UiPointerEventKind::Move | UiPointerEventKind::Up)
                && (kind != UiPointerEventKind::Up || button == Some(active.button))
        })
    }

    pub(super) fn observe(
        &mut self,
        source_window: Option<MainPageId>,
        pointer: &UiPointerInputEvent,
        hierarchy_route: bool,
    ) -> Option<HierarchyTerminalReason> {
        self.observe_event(
            source_window,
            pointer.metadata.pointer_id.unwrap_or_default(),
            pointer.event.kind,
            pointer.event.button,
            hierarchy_route,
        )
    }

    pub(super) fn observe_move(
        &mut self,
        source_window: Option<MainPageId>,
        pointer_id: UiPointerId,
        hierarchy_route: bool,
    ) -> Option<HierarchyTerminalReason> {
        self.observe_event(
            source_window,
            pointer_id,
            UiPointerEventKind::Move,
            None,
            hierarchy_route,
        )
    }

    fn observe_event(
        &mut self,
        source_window: Option<MainPageId>,
        pointer_id: UiPointerId,
        kind: UiPointerEventKind,
        button: Option<UiPointerButton>,
        hierarchy_route: bool,
    ) -> Option<HierarchyTerminalReason> {
        self.observed_input = Some(ObservedPointerInput {
            source_window: source_window.clone(),
            pointer_id,
            kind,
            button,
            motion_eligible: true,
        });
        let active = self.active.as_ref()?;
        if active.pointer_id != pointer_id {
            return None;
        }
        if source_window != active.source_window {
            return Some(HierarchyTerminalReason::ForeignWindow);
        }
        match kind {
            UiPointerEventKind::Cancel => Some(HierarchyTerminalReason::PointerCancel),
            UiPointerEventKind::Down if button == Some(UiPointerButton::Secondary) => {
                Some(HierarchyTerminalReason::SecondaryPress)
            }
            UiPointerEventKind::Move if !hierarchy_route => {
                Some(HierarchyTerminalReason::OutsidePane)
            }
            UiPointerEventKind::Up if button == Some(active.button) && !hierarchy_route => {
                Some(HierarchyTerminalReason::OutsidePane)
            }
            _ => None,
        }
    }

    pub(super) fn finish(&mut self, reason: HierarchyTerminalReason) -> bool {
        let Some(active) = self.active.take() else {
            return false;
        };
        self.terminal_count = self.terminal_count.saturating_add(1);
        self.last_terminal = Some(HierarchyTerminalReceipt {
            generation: active.generation,
            reason,
        });
        true
    }
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn finish_unrouted_hierarchy_native_release(
        &mut self,
        source: &HierarchyPointerSource,
        pointer_id: UiPointerId,
    ) {
        if !self.hierarchy_input_owner.has_active() {
            return;
        }
        let source_window = source.native_floating_window_id().map(MainPageId::new);
        if self
            .hierarchy_input_owner
            .owns_primary_release(&source_window, pointer_id)
        {
            self.retire_hierarchy_drag_with_reason(HierarchyTerminalReason::UnroutedRelease);
        }
    }

    pub(in crate::ui::retained_host::app) fn observe_hierarchy_native_move(
        &mut self,
        source: &HierarchyPointerSource,
        pointer_id: UiPointerId,
        x: f32,
        y: f32,
        eligible: bool,
    ) {
        if !self.hierarchy_input_owner.has_active() {
            return;
        }
        let source_window = source.native_floating_window_id().map(MainPageId::new);
        if !eligible {
            self.hierarchy_input_owner
                .note_ineligible_move(source_window, pointer_id);
            return;
        }
        let hierarchy_route = !self
            .hierarchy_input_owner
            .needs_move_route_check(&source_window, pointer_id)
            || source.has_hierarchy_pointer_route(x, y);
        if let Some(reason) =
            self.hierarchy_input_owner
                .observe_move(source_window, pointer_id, hierarchy_route)
        {
            self.retire_hierarchy_drag_with_reason(reason);
        }
    }

    pub(in crate::ui::retained_host::app) fn observe_hierarchy_native_input(
        &mut self,
        source: &HierarchyPointerSource,
        pointer: &UiPointerInputEvent,
    ) {
        if !self.hierarchy_input_owner.needs_observation(pointer) {
            return;
        }
        let source_window = source.native_floating_window_id().map(MainPageId::new);
        let hierarchy_route = !self
            .hierarchy_input_owner
            .needs_route_check(&source_window, pointer)
            || source.has_hierarchy_pointer_route(pointer.event.point.x, pointer.event.point.y);
        if let Some(reason) =
            self.hierarchy_input_owner
                .observe(source_window, pointer, hierarchy_route)
        {
            self.retire_hierarchy_drag_with_reason(reason);
        }
    }

    pub(in crate::ui::retained_host::app) fn cancel_hierarchy_drag_for_window(
        &mut self,
        window: &Option<MainPageId>,
        reason: HierarchyTerminalReason,
    ) {
        if self.hierarchy_input_owner.owns_window(window) {
            self.retire_hierarchy_drag_with_reason(reason);
        }
    }
}
