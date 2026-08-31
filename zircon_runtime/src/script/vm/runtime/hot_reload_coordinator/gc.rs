use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::Ordering;

use super::super::super::backend::VmError;
use super::super::super::gc_bridge::{VmGcBudget, VmGcSlotStepReport, VmGcStepReport};
use super::super::super::plugin::VmPluginGarbageCollectionMode;
use super::super::vm_plugin_slot_state::VmPluginSlotState;
use super::{gc_deadline::GcFrameDeadline, HotReloadCoordinator, PluginSlot};

impl HotReloadCoordinator {
    /// Steps eligible cooperative collectors in stable slot order until the frame budget is spent.
    pub fn gc_step(&self, budget: VmGcBudget) -> Result<VmGcStepReport, VmError> {
        let _gc_step_guard = self.lock_gc_step_guard();
        let deadline = GcFrameDeadline::start(budget.max_micros_per_frame);
        let frame_index = self
            .next_gc_frame
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |frame| {
                frame.checked_add(1)
            })
            .map_err(|_| VmError::Operation("vm GC frame index exhausted".to_string()))?;
        let due_slots = self.lock_gc_schedule().take_due(frame_index);
        let report_capacity = {
            let mut pending = self.lock_pending_gc_slots();
            pending.extend(due_slots);
            pending.len()
        };

        let mut slot_reports = Vec::with_capacity(report_capacity);
        loop {
            if deadline.remaining_micros() == 0 {
                break;
            }
            let Some(slot) = self.lock_pending_gc_slots().pop_front() else {
                break;
            };
            let mut instance = {
                let mut slots = self.lock_slots();
                let Some(slot_entry) = slots.get_mut(&slot) else {
                    continue;
                };
                if !gc_policy_is_cooperative_active(slot_entry) {
                    continue;
                }
                slot_entry.instance.take().ok_or_else(|| {
                    VmError::Operation(format!(
                        "vm plugin slot {} cannot step GC while active instance is unavailable",
                        slot.get()
                    ))
                })?
            };
            let remaining_micros = deadline.remaining_micros();
            if remaining_micros == 0 {
                self.restore_slot_instance(slot, instance, VmPluginSlotState::Active);
                self.requeue_gc_slot_front(slot);
                break;
            }
            let step_budget = VmGcBudget {
                max_micros_per_frame: remaining_micros,
            };
            let step_timer = deadline.begin_step();
            let outcome = catch_unwind(AssertUnwindSafe(|| instance.gc_step(step_budget)));
            let host_elapsed_micros = step_timer.elapsed_micros();
            self.restore_slot_instance(slot, instance, VmPluginSlotState::Active);
            let outcome = match outcome {
                Ok(Ok(outcome)) => outcome,
                Ok(Err(error)) => {
                    self.requeue_gc_slot_front(slot);
                    return Err(error);
                }
                Err(payload) => {
                    self.requeue_gc_slot_front(slot);
                    resume_unwind(payload);
                }
            };
            slot_reports.push(VmGcSlotStepReport {
                slot,
                budget_micros: remaining_micros,
                host_elapsed_micros,
                outcome,
            });
            if deadline.remaining_micros() == 0 {
                break;
            }
        }

        Ok(VmGcStepReport::from_slots(
            frame_index,
            budget,
            deadline.elapsed_micros(),
            slot_reports,
        ))
    }

    fn requeue_gc_slot_front(&self, slot: super::super::super::handles::PluginSlotId) {
        let mut pending = self.lock_pending_gc_slots();
        pending.push_front(slot);
    }

    pub(super) fn refresh_gc_schedule(&self, slot: super::super::super::handles::PluginSlotId) {
        let interval_frames = {
            let slots = self.lock_slots();
            slots.get(&slot).and_then(gc_schedule_interval)
        };
        let next_frame = self.next_gc_frame.load(Ordering::SeqCst);
        self.lock_gc_schedule()
            .replace(slot, interval_frames, next_frame);
    }
}

fn gc_schedule_interval(slot: &PluginSlot) -> Option<u64> {
    if !gc_policy_is_cooperative_active(slot) {
        return None;
    }
    match slot
        .package
        .manifest
        .management
        .garbage_collection
        .interval_frames
    {
        None => Some(1),
        Some(0) => None,
        Some(interval) => Some(interval),
    }
}

fn gc_policy_is_cooperative_active(slot: &PluginSlot) -> bool {
    slot.state == VmPluginSlotState::Active
        && matches!(
            slot.package.manifest.management.garbage_collection.mode,
            VmPluginGarbageCollectionMode::Cooperative
        )
}
