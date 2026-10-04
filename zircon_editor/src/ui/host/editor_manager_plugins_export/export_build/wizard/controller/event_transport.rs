use crossbeam_channel::{Sender, TrySendError};
use zircon_runtime_interface::export::ExportStage;

use super::super::{ExportWizardJobEvent, ExportWizardJobEventKind};

pub(super) const EXPORT_WIZARD_EVENT_CHANNEL_CAPACITY: usize = 192;
const MAX_PENDING_OUTPUT_EVENTS: usize = 128;
const MAX_CONTROL_EVENTS: usize = 2 * ExportStage::ALL.len() + 3;
const _: () =
    assert!(EXPORT_WIZARD_EVENT_CHANNEL_CAPACITY >= MAX_PENDING_OUTPUT_EVENTS + MAX_CONTROL_EVENTS);

pub(super) fn send_job_event(
    events: &Sender<ExportWizardJobEvent>,
    mut event: ExportWizardJobEvent,
    coalesced_output_events: &mut u64,
) {
    event.coalesced_output_events = *coalesced_output_events;
    if event.kind == ExportWizardJobEventKind::StageOutput {
        if events.len() >= MAX_PENDING_OUTPUT_EVENTS {
            *coalesced_output_events = coalesced_output_events.saturating_add(1);
            return;
        }
        if let Err(TrySendError::Full(_)) = events.try_send(event) {
            *coalesced_output_events = coalesced_output_events.saturating_add(1);
        }
        return;
    }

    // One producer emits at most MAX_CONTROL_EVENTS for the unique planned stages.
    // Output admission reserves that entire budget, even without a UI consumer.
    if let Err(TrySendError::Full(_)) = events.try_send(event) {
        unreachable!("export wizard control event reserve exhausted");
    }
}

#[cfg(test)]
#[path = "event_transport/tests/astra_admission_tests.rs"]
mod astra_admission_tests;
