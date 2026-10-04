use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, PointerSource, WindowEvent};
use zircon_runtime_interface::ui::dispatch::{UiInputEventMetadata, UiInputSequence};

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::globals::UiHostContext;

struct PendingIdlePointerMove {
    device_id: Option<DeviceId>,
    metadata: UiInputEventMetadata,
    fallback_position: PhysicalPosition<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiCoalescedInputRange {
    pub(super) first_sequence: UiInputSequence,
    pub(super) last_sequence: UiInputSequence,
    pub(super) count: u64,
}

impl UiCoalescedInputRange {
    fn single(sequence: UiInputSequence) -> Self {
        Self {
            first_sequence: sequence,
            last_sequence: sequence,
            count: 1,
        }
    }

    fn include(&mut self, sequence: UiInputSequence) {
        debug_assert_eq!(
            sequence.0,
            self.last_sequence.0.saturating_add(1),
            "one idle-hover mailbox epoch must contain consecutive input sequences"
        );
        self.last_sequence = sequence;
        self.count = self.count.saturating_add(1);
    }
}

struct PendingIdlePointerMoveBatch {
    pending: PendingIdlePointerMove,
    received_count: u64,
    coalesced: Option<UiCoalescedInputRange>,
}

pub(super) struct UiIdlePointerMoveMailbox {
    pending_idle_pointer_move: Option<PendingIdlePointerMove>,
    received_count: u64,
    coalesced: Option<UiCoalescedInputRange>,
}

impl Default for UiIdlePointerMoveMailbox {
    fn default() -> Self {
        Self {
            pending_idle_pointer_move: None,
            received_count: 0,
            coalesced: None,
        }
    }
}

impl UiIdlePointerMoveMailbox {
    fn replace(&mut self, next: PendingIdlePointerMove) {
        self.received_count = self.received_count.saturating_add(1);
        let Some(replaced) = self.pending_idle_pointer_move.replace(next) else {
            return;
        };
        match self.coalesced.as_mut() {
            Some(range) => range.include(replaced.metadata.sequence),
            None => {
                self.coalesced = Some(UiCoalescedInputRange::single(replaced.metadata.sequence));
            }
        }
    }

    fn take(&mut self) -> Option<PendingIdlePointerMoveBatch> {
        let pending = self.pending_idle_pointer_move.take()?;
        let batch = PendingIdlePointerMoveBatch {
            pending,
            received_count: self.received_count,
            coalesced: self.coalesced.take(),
        };
        self.received_count = 0;
        Some(batch)
    }

    fn pending_device_id(&self) -> Option<Option<DeviceId>> {
        self.pending_idle_pointer_move
            .as_ref()
            .map(|pending| pending.device_id)
    }
}

impl UiHostWindowEventLoop {
    pub(super) fn try_defer_idle_pointer_move(&mut self, event: &WindowEvent) -> bool {
        let WindowEvent::PointerMoved {
            device_id,
            position,
            primary: true,
            source: PointerSource::Mouse,
        } = event
        else {
            return false;
        };
        if self.pressed_mouse_button_count != 0
            || self
                .host
                .global::<UiHostContext>()
                .pointer_move_requires_immediate_dispatch()
        {
            return false;
        }
        if self.pending_idle_pointer_move.pending_device_id().is_some()
            && self.pending_idle_pointer_move.pending_device_id() != Some(*device_id)
        {
            self.flush_pending_idle_pointer_move();
        }

        let next = PendingIdlePointerMove {
            device_id: *device_id,
            metadata: self.reserve_input_metadata(),
            fallback_position: *position,
        };
        self.pending_idle_pointer_move.replace(next);
        true
    }

    pub(super) fn flush_pending_idle_pointer_move(&mut self) {
        let Some(batch) = self.pending_idle_pointer_move.take() else {
            return;
        };
        self.record_idle_pointer_move_batch(batch.received_count, batch.coalesced);
        let platform_event = self.translate_reserved_pointer_move_event(
            batch.pending.metadata,
            batch.pending.device_id,
            batch.pending.fallback_position,
        );
        self.handle_pointer_moved(platform_event, batch.pending.fallback_position);
    }
}

#[cfg(test)]
#[path = "tests/pointer_move_mailbox.rs"]
mod tests;
