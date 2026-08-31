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
    let mut visiting = HashSet::new();
    prerequisites.iter().find_map(|prerequisite| {
        visiting.clear();
        prerequisite_result(prerequisite, prerequisites, &mut visiting).err()
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

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::core::runtime::tasks::bounded_keyed_io::BoundedKeyedIoTicket;

    const BENCHMARK_PREREQUISITE_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 64;

    #[test]
    fn reused_visiting_set_preserves_fence_results() {
        let success = terminal_prerequisite(1, "asset:a", 1, BoundedKeyedIoTerminal::Succeeded);
        let failed = terminal_prerequisite(
            2,
            "asset:b",
            1,
            BoundedKeyedIoTerminal::Failed(BoundedKeyedIoFailure::new("write_failed")),
        );
        let superseded = terminal_prerequisite(
            3,
            "asset:c",
            1,
            BoundedKeyedIoTerminal::Superseded { successor: 2 },
        );
        let successor = terminal_prerequisite(4, "asset:c", 2, BoundedKeyedIoTerminal::Succeeded);

        for prerequisites in [
            vec![success.clone()],
            vec![success, failed],
            vec![superseded, successor],
        ] {
            assert_eq!(
                fence_prerequisite_failure(&prerequisites),
                retired_fence_prerequisite_failure(&prerequisites)
            );
        }
    }

    #[test]
    fn fence_failure_reuses_one_visiting_set_across_roots() {
        let source = include_str!("fence_prerequisites.rs");
        let implementation = source
            .split_once("pub(super) fn fence_prerequisite_failure")
            .expect("fence prerequisite failure function")
            .1
            .split_once("\n}\n\nfn fence_prerequisite")
            .expect("fence prerequisite failure function end")
            .0;
        let per_root_set = ["&mut HashSet", "::new()"].concat();

        assert!(!implementation.contains(&per_root_set));
        assert_eq!(implementation.matches("HashSet::new()").count(), 1);
        assert!(implementation.contains("visiting.clear()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn reused_fence_visiting_set_release_benchmark() {
        let prerequisites = (0..BENCHMARK_PREREQUISITE_COUNT)
            .map(|index| {
                terminal_prerequisite(
                    index as u64 + 1,
                    &format!("asset:{index:04}"),
                    1,
                    BoundedKeyedIoTerminal::Succeeded,
                )
            })
            .collect::<Vec<_>>();
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_failure_scan(|| {
                    retired_fence_prerequisite_failure(&prerequisites)
                }));
                optimized_samples.push(measure_failure_scan(|| {
                    fence_prerequisite_failure(&prerequisites)
                }));
            } else {
                optimized_samples.push(measure_failure_scan(|| {
                    fence_prerequisite_failure(&prerequisites)
                }));
                retired_samples.push(measure_failure_scan(|| {
                    retired_fence_prerequisite_failure(&prerequisites)
                }));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME59_REUSED_FENCE_VISITING_SET_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
prerequisites={BENCHMARK_PREREQUISITE_COUNT} \
retired_visiting_set_allocations_per_scan=4096 optimized_visiting_set_allocations_per_scan=1 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(75),
            "reused visiting set must reduce fence failure scan P95 by at least 25%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn terminal_prerequisite(
        id: u64,
        key: &str,
        generation: u64,
        terminal: BoundedKeyedIoTerminal,
    ) -> FencePrerequisite {
        let ticket = BoundedKeyedIoTicket::pending(id, generation, false);
        assert!(ticket.mark_terminal(terminal));
        FencePrerequisite {
            key: Some(Arc::<str>::from(key).into()),
            generation,
            ticket,
        }
    }

    fn measure_failure_scan(mut scan: impl FnMut() -> Option<BoundedKeyedIoFailure>) -> Duration {
        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            assert_eq!(black_box(scan()), None);
        }
        started.elapsed()
    }

    fn retired_fence_prerequisite_failure(
        prerequisites: &[FencePrerequisite],
    ) -> Option<BoundedKeyedIoFailure> {
        prerequisites.iter().find_map(|prerequisite| {
            prerequisite_result(prerequisite, prerequisites, &mut HashSet::new()).err()
        })
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
