use std::collections::HashSet;
use std::mem::size_of;

use super::{FencePrerequisite, LaneState, WorkEntry};
use crate::core::runtime::tasks::bounded_keyed_io::{
    BoundedKeyedIoFailure, BoundedKeyedIoTerminal, GlobalAdmissionEpoch,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct FencePrerequisitePlan {
    latest_fence_ticket_id: Option<u64>,
    latest_fence_epoch: Option<GlobalAdmissionEpoch>,
    fence_epoch: GlobalAdmissionEpoch,
    prerequisite_count: usize,
}

impl FencePrerequisitePlan {
    pub(super) fn retained_bytes(self) -> Option<usize> {
        self.prerequisite_count
            .checked_mul(size_of::<FencePrerequisite>())
    }

    fn includes_non_fence_epoch(self, entry_epoch: GlobalAdmissionEpoch) -> bool {
        entry_epoch <= self.fence_epoch
            && self
                .latest_fence_epoch
                .is_none_or(|latest| entry_epoch > latest)
    }
}

pub(super) fn plan_fence_prerequisites(
    state: &LaneState,
    epoch: GlobalAdmissionEpoch,
) -> Option<FencePrerequisitePlan> {
    let latest_fence = state
        .active
        .iter()
        .filter(|entry| entry.fence && entry.epoch <= epoch)
        .map(|entry| (entry.epoch, entry.ticket.id()))
        .chain(
            state
                .queue
                .iter()
                .filter(|entry| entry.fence && entry.epoch <= epoch)
                .map(|entry| (entry.epoch, entry.ticket.id())),
        )
        .max_by_key(|(entry_epoch, _)| *entry_epoch);
    let latest_fence_epoch = latest_fence.map(|(entry_epoch, _)| entry_epoch);
    let after_latest_fence = |entry_epoch: GlobalAdmissionEpoch| {
        entry_epoch <= epoch && latest_fence_epoch.is_none_or(|latest| entry_epoch > latest)
    };
    let suspended = state
        .suspended
        .values()
        .filter(|entry| after_latest_fence(entry.epoch))
        .count();
    let active = usize::from(
        state
            .active
            .as_ref()
            .is_some_and(|entry| !entry.fence && after_latest_fence(entry.epoch)),
    );
    let queued = state
        .queue
        .iter()
        .filter(|entry| !entry.fence && after_latest_fence(entry.epoch))
        .count();
    let prerequisite_count = suspended
        .checked_add(active)?
        .checked_add(queued)?
        .checked_add(usize::from(latest_fence.is_some()))?;
    Some(FencePrerequisitePlan {
        latest_fence_ticket_id: latest_fence.map(|(_, ticket_id)| ticket_id),
        latest_fence_epoch,
        fence_epoch: epoch,
        prerequisite_count,
    })
}

pub(super) fn capture_fence_prerequisites(
    state: &LaneState,
    plan: FencePrerequisitePlan,
) -> Vec<FencePrerequisite> {
    let mut prerequisites = Vec::with_capacity(plan.prerequisite_count);
    if let Some(ticket_id) = plan.latest_fence_ticket_id {
        if let Some(entry) = state
            .active
            .as_ref()
            .filter(|entry| entry.ticket.id() == ticket_id)
        {
            prerequisites.push(active_fence_prerequisite(entry));
        } else if let Some(entry) = state
            .queue
            .iter()
            .find(|entry| entry.ticket.id() == ticket_id)
        {
            prerequisites.push(fence_prerequisite(entry));
        }
    }
    prerequisites.extend(
        state
            .suspended
            .values()
            .filter(|entry| plan.includes_non_fence_epoch(entry.epoch))
            .map(fence_prerequisite),
    );
    prerequisites.extend(
        state
            .active
            .iter()
            .filter(|entry| !entry.fence && plan.includes_non_fence_epoch(entry.epoch))
            .map(active_fence_prerequisite),
    );
    prerequisites.extend(
        state
            .queue
            .iter()
            .filter(|entry| !entry.fence && plan.includes_non_fence_epoch(entry.epoch))
            .map(fence_prerequisite),
    );
    debug_assert_eq!(prerequisites.len(), plan.prerequisite_count);
    prerequisites
}

pub(super) fn release_fence_pins(entry: &WorkEntry) {
    if !entry.fence {
        return;
    }
    for prerequisite in &entry.prerequisites {
        prerequisite.ticket.unpin_from_fence();
    }
}

pub(super) fn fence_prerequisite_failure(
    prerequisites: &[FencePrerequisite],
) -> Option<BoundedKeyedIoFailure> {
    prerequisites.iter().find_map(|prerequisite| {
        prerequisite_result(prerequisite, prerequisites, &mut HashSet::new()).err()
    })
}

fn fence_prerequisite(entry: &WorkEntry) -> FencePrerequisite {
    FencePrerequisite {
        key: entry.key.clone(),
        generation: entry.generation,
        ticket: entry.ticket.clone(),
    }
}

fn active_fence_prerequisite(entry: &super::ActiveEntry) -> FencePrerequisite {
    FencePrerequisite {
        key: entry.key.clone(),
        generation: entry.generation,
        ticket: entry.ticket.clone(),
    }
}

fn prerequisite_result(
    prerequisite: &FencePrerequisite,
    prerequisites: &[FencePrerequisite],
    visiting: &mut HashSet<u64>,
) -> Result<(), BoundedKeyedIoFailure> {
    if !visiting.insert(prerequisite.ticket.id()) {
        return Err(BoundedKeyedIoFailure::new("pre_fence_obligation_cycle"));
    }
    let result = match prerequisite.ticket.terminal() {
        Some(BoundedKeyedIoTerminal::Succeeded) => Ok(()),
        Some(BoundedKeyedIoTerminal::Superseded { successor }) => {
            later_generation_result(prerequisite, prerequisites, visiting, |next| {
                next.generation == successor
            })
            .unwrap_or_else(|| {
                Err(BoundedKeyedIoFailure::new(
                    "pre_fence_obligation_superseded",
                ))
            })
        }
        Some(BoundedKeyedIoTerminal::Failed(failure)) => {
            later_generation_result(prerequisite, prerequisites, visiting, |next| {
                next.generation > prerequisite.generation
            })
            .unwrap_or(Err(failure))
        }
        Some(BoundedKeyedIoTerminal::DeadlineBeforeStart) => Err(BoundedKeyedIoFailure::new(
            "pre_fence_obligation_deadline_before_start",
        )),
        Some(BoundedKeyedIoTerminal::CancelledBeforeStart) => Err(BoundedKeyedIoFailure::new(
            "pre_fence_obligation_cancelled_before_start",
        )),
        Some(BoundedKeyedIoTerminal::Shutdown) => {
            Err(BoundedKeyedIoFailure::new("pre_fence_obligation_shutdown"))
        }
        None => Err(BoundedKeyedIoFailure::new(
            "pre_fence_obligation_incomplete",
        )),
    };
    visiting.remove(&prerequisite.ticket.id());
    result
}

fn later_generation_result(
    prerequisite: &FencePrerequisite,
    prerequisites: &[FencePrerequisite],
    visiting: &mut HashSet<u64>,
    matches_generation: impl Fn(&FencePrerequisite) -> bool,
) -> Option<Result<(), BoundedKeyedIoFailure>> {
    prerequisite.key.as_ref()?;
    prerequisites
        .iter()
        .filter(|next| {
            next.ticket.id() != prerequisite.ticket.id()
                && next.key == prerequisite.key
                && matches_generation(next)
        })
        .map(|next| prerequisite_result(next, prerequisites, visiting))
        .find(Result::is_ok)
}
