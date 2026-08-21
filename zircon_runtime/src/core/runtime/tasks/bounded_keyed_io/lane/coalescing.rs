use std::collections::VecDeque;

use super::{LaneState, TerminalNotification, WorkEntry, release_reservation};
use crate::core::runtime::tasks::bounded_keyed_io::BoundedKeyedIoTerminal;

pub(super) fn insert_ordered(queue: &mut VecDeque<WorkEntry>, entry: WorkEntry) {
    let insertion = queue
        .iter()
        .position(|queued| {
            queued.epoch > entry.epoch
                || (queued.epoch == entry.epoch
                    && (queued.fence || queued.ticket.id() > entry.ticket.id()))
        })
        .unwrap_or(queue.len());
    queue.insert(insertion, entry);
}

fn take_matching_entries<T>(
    queue: &mut VecDeque<T>,
    mut matches: impl FnMut(&T) -> bool,
) -> Vec<T> {
    let mut pending = std::mem::take(queue);
    queue.reserve(pending.len());
    let mut matching = Vec::new();
    while let Some(entry) = pending.pop_front() {
        if matches(&entry) {
            matching.push(entry);
        } else {
            queue.push_back(entry);
        }
    }
    matching
}

pub(super) fn coalesce_queued_generation(
    state: &mut LaneState,
    successor: &WorkEntry,
    notifications: &mut Vec<TerminalNotification>,
) -> bool {
    let Some(key) = successor.key.as_deref() else {
        return true;
    };
    let active_successor = state.active.as_ref().is_some_and(|active| {
        active.epoch == successor.epoch
            && active.key.as_deref() == Some(key)
            && active.generation > successor.generation
    });
    let queued_successor = state.queue.iter().any(|queued| {
        !queued.fence
            && queued.epoch == successor.epoch
            && queued.key.as_deref() == Some(key)
            && queued.generation > successor.generation
    });
    if active_successor || queued_successor {
        let successor_generation = state
            .active
            .iter()
            .filter(|active| active.epoch == successor.epoch && active.key.as_deref() == Some(key))
            .map(|active| active.generation)
            .chain(
                state
                    .queue
                    .iter()
                    .filter(|queued| {
                        !queued.fence
                            && queued.epoch == successor.epoch
                            && queued.key.as_deref() == Some(key)
                    })
                    .map(|queued| queued.generation),
            )
            .max()
            .unwrap_or(successor.generation);
        let terminal = BoundedKeyedIoTerminal::Superseded {
            successor: successor_generation,
        };
        successor.ticket.mark_terminal(terminal);
        notifications.push(TerminalNotification {
            observer: successor.terminal_observer.clone(),
            terminal,
        });
        release_reservation(state, successor.retained_bytes);
        state.superseded = state.superseded.saturating_add(1);
        state.coalesced = state.coalesced.saturating_add(1);
        return false;
    }

    let superseded_entries = take_matching_entries(&mut state.queue, |queued| {
        !queued.fence && queued.epoch == successor.epoch && queued.key.as_deref() == Some(key)
    });
    for queued in superseded_entries {
        let terminal = BoundedKeyedIoTerminal::Superseded {
            successor: successor.generation,
        };
        queued.ticket.mark_terminal(terminal);
        notifications.push(TerminalNotification {
            observer: queued.terminal_observer,
            terminal,
        });
        release_reservation(state, queued.retained_bytes);
        state.superseded = state.superseded.saturating_add(1);
        state.coalesced = state.coalesced.saturating_add(1);
    }
    true
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const BENCH_ENTRY_COUNT: usize = 8_192;
    const BENCH_MATCHED_ENTRIES: usize = BENCH_ENTRY_COUNT / 2;
    const BENCH_SAMPLE_PAIRS: usize = 21;

    #[test]
    fn bounded_keyed_io_matching_queue_partition_preserves_retained_and_removed_order() {
        let mut queue = VecDeque::from([1, 2, 3, 2, 4, 2]);

        let removed = take_matching_entries(&mut queue, |value| *value == 2);

        assert_eq!(queue, VecDeque::from([1, 3, 4]));
        assert_eq!(removed, vec![2, 2, 2]);
    }

    #[test]
    #[ignore = "release performance gate; run through the managed Runtime45 validator"]
    fn bounded_keyed_io_linear_queued_generation_coalescing_release_benchmark() {
        let source = (0..BENCH_ENTRY_COUNT as u32).collect::<VecDeque<_>>();
        let first_match = (BENCH_ENTRY_COUNT - BENCH_MATCHED_ENTRIES) as u32;
        let mut legacy_queue = source.clone();
        let mut optimized_queue = source.clone();
        let legacy_removed =
            legacy_take_matching_entries(&mut legacy_queue, |value| *value >= first_match);
        let optimized_removed =
            take_matching_entries(&mut optimized_queue, |value| *value >= first_match);
        assert_eq!(optimized_queue, legacy_queue);
        assert_eq!(optimized_removed, legacy_removed);

        let mut legacy_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
        for pair_index in 0..BENCH_SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&source, first_match));
                optimized_samples.push(measure_linear(&source, first_match));
            } else {
                optimized_samples.push(measure_linear(&source, first_match));
                legacy_samples.push(measure_legacy(&source, first_match));
            }
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let legacy_queue_element_moves =
            BENCH_MATCHED_ENTRIES.saturating_mul(BENCH_MATCHED_ENTRIES - 1) / 2;
        println!(
            "BOUNDED_KEYED_IO_QUEUE_COALESCING_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank entry_count={} matched_entries={} legacy_queue_element_moves={} optimized_linear_visits={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
            BENCH_SAMPLE_PAIRS,
            BENCH_ENTRY_COUNT,
            BENCH_MATCHED_ENTRIES,
            legacy_queue_element_moves,
            BENCH_ENTRY_COUNT,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95,
            join_samples(&legacy_samples),
            join_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(4) <= legacy_p95,
            "linear queue coalescing P95 must be at most 25% of legacy: legacy={legacy_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn measure_legacy(source: &VecDeque<u32>, first_match: u32) -> u128 {
        let mut queue = source.clone();
        let started = Instant::now();
        black_box(legacy_take_matching_entries(
            black_box(&mut queue),
            |value| *value >= first_match,
        ));
        started.elapsed().as_nanos()
    }

    fn measure_linear(source: &VecDeque<u32>, first_match: u32) -> u128 {
        let mut queue = source.clone();
        let started = Instant::now();
        black_box(take_matching_entries(black_box(&mut queue), |value| {
            *value >= first_match
        }));
        started.elapsed().as_nanos()
    }

    fn legacy_take_matching_entries<T>(
        queue: &mut VecDeque<T>,
        mut matches: impl FnMut(&T) -> bool,
    ) -> Vec<T> {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < queue.len() {
            if matches(&queue[index]) {
                removed.push(queue.remove(index).expect("matched queue entry must exist"));
            } else {
                index += 1;
            }
        }
        removed
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
        ordered[rank.saturating_sub(1)]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
