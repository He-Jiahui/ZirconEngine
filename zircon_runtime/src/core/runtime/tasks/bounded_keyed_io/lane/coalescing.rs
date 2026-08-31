use std::collections::VecDeque;

use super::{release_reservation, LaneState, TerminalNotification, WorkEntry};
use crate::core::runtime::tasks::bounded_keyed_io::BoundedKeyedIoTerminal;

pub(super) fn insert_ordered(queue: &mut VecDeque<WorkEntry>, entry: WorkEntry) {
    insert_ordered_by(queue, entry, |queued, entry| {
        queued.epoch > entry.epoch
            || (queued.epoch == entry.epoch
                && (queued.fence || queued.ticket.id() > entry.ticket.id()))
    });
}

fn insert_ordered_by<T>(
    queue: &mut VecDeque<T>,
    entry: T,
    mut should_insert_before: impl FnMut(&T, &T) -> bool,
) {
    let append_to_tail = match queue.back() {
        Some(queued) => !should_insert_before(queued, &entry),
        None => true,
    };
    if append_to_tail {
        queue.push_back(entry);
        return;
    }

    let insertion = queue
        .iter()
        .position(|queued| should_insert_before(queued, &entry))
        .expect("queue tail must follow an entry that cannot append");
    queue.insert(insertion, entry);
}

fn latest_generation_above(
    successor_generation: u64,
    generations: impl Iterator<Item = u64>,
) -> Option<u64> {
    generations
        .filter(|generation| *generation > successor_generation)
        .max()
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
    let Some(key) = successor.key.as_ref() else {
        return true;
    };
    let successor_generation = latest_generation_above(
        successor.generation,
        state
            .active
            .iter()
            .filter(|active| active.epoch == successor.epoch && active.key.as_ref() == Some(key))
            .map(|active| active.generation)
            .chain(
                state
                    .queue
                    .iter()
                    .filter(|queued| {
                        !queued.fence
                            && queued.epoch == successor.epoch
                            && queued.key.as_ref() == Some(key)
                    })
                    .map(|queued| queued.generation),
            ),
    );
    if let Some(successor_generation) = successor_generation {
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
        !queued.fence && queued.epoch == successor.epoch && queued.key.as_ref() == Some(key)
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
    use std::cell::Cell;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const BENCH_ENTRY_COUNT: usize = 8_192;
    const BENCH_MATCHED_ENTRIES: usize = BENCH_ENTRY_COUNT / 2;
    const BENCH_SAMPLE_PAIRS: usize = 21;
    const ORDERED_INSERT_ENTRY_COUNT: usize = 4_096;
    const ORDERED_INSERT_ITERATIONS: usize = 512;
    const SUCCESSOR_SCAN_ENTRY_COUNT: usize = 4_096;
    const SUCCESSOR_SCAN_ITERATIONS: usize = 256;
    const OPTIMIZATION_BENCH_SAMPLE_PAIRS: usize = 11;

    #[test]
    fn bounded_keyed_io_matching_queue_partition_preserves_retained_and_removed_order() {
        let mut queue = VecDeque::from([1, 2, 3, 2, 4, 2]);

        let removed = take_matching_entries(&mut queue, |value| *value == 2);

        assert_eq!(queue, VecDeque::from([1, 3, 4]));
        assert_eq!(removed, vec![2, 2, 2]);
    }

    #[test]
    fn runtime59_coalescing_fast_tail_ordered_insertion_preserves_tail_and_middle_order() {
        let comparisons = Cell::new(0);
        let mut tail_queue = VecDeque::from([1, 3, 5, 7]);
        insert_ordered_by(&mut tail_queue, 9, |queued, incoming| {
            comparisons.set(comparisons.get() + 1);
            queued > incoming
        });
        assert_eq!(tail_queue, VecDeque::from([1, 3, 5, 7, 9]));
        assert_eq!(comparisons.get(), 1);

        let mut middle_queue = VecDeque::from([1, 3, 5, 7]);
        insert_ordered_by(&mut middle_queue, 4, |queued, incoming| queued > incoming);
        assert_eq!(middle_queue, VecDeque::from([1, 3, 4, 5, 7]));
    }

    #[test]
    fn runtime59_coalescing_single_pass_successor_generation_selects_latest_newer_generation() {
        let visits = Cell::new(0);
        let generations = [7_u64, 11, 9, 13, 12];

        let latest = latest_generation_above(
            10,
            generations
                .into_iter()
                .inspect(|_| visits.set(visits.get() + 1)),
        );

        assert_eq!(latest, Some(13));
        assert_eq!(visits.get(), generations.len());
        assert_eq!(latest_generation_above(13, generations.into_iter()), None);
    }

    #[test]
    #[ignore = "release performance gate; run through the managed Runtime59 validator"]
    fn runtime59_coalescing_fast_tail_ordered_insertion_release_benchmark() {
        let source = (0..ORDERED_INSERT_ENTRY_COUNT as u32).collect::<VecDeque<_>>();
        let incoming = ORDERED_INSERT_ENTRY_COUNT as u32;
        let mut retired_samples = Vec::with_capacity(OPTIMIZATION_BENCH_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(OPTIMIZATION_BENCH_SAMPLE_PAIRS);
        for pair_index in 0..OPTIMIZATION_BENCH_SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                retired_samples.push(measure_retired_tail_insertion(&source, incoming));
                optimized_samples.push(measure_fast_tail_insertion(&source, incoming));
            } else {
                optimized_samples.push(measure_fast_tail_insertion(&source, incoming));
                retired_samples.push(measure_retired_tail_insertion(&source, incoming));
            }
        }

        let retired_p95 = nearest_rank(&retired_samples, 95);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95
                .saturating_mul(10_000)
                .checked_div(retired_p95)
                .unwrap_or(0),
        );
        println!(
            "RUNTIME59_FAST_TAIL_ORDERED_INSERTION_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank entry_count={} iterations={} retired_comparisons={} optimized_comparisons={} retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
            OPTIMIZATION_BENCH_SAMPLE_PAIRS,
            ORDERED_INSERT_ENTRY_COUNT,
            ORDERED_INSERT_ITERATIONS,
            ORDERED_INSERT_ENTRY_COUNT * ORDERED_INSERT_ITERATIONS,
            ORDERED_INSERT_ITERATIONS,
            retired_p95,
            optimized_p95,
            reduction_basis_points,
            join_samples(&retired_samples),
            join_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(4) <= retired_p95,
            "tail insertion P95 must be at most 25% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
        );
    }

    #[test]
    #[ignore = "release performance gate; run through the managed Runtime59 validator"]
    fn runtime59_coalescing_single_pass_successor_generation_release_benchmark() {
        let generations = (0..SUCCESSOR_SCAN_ENTRY_COUNT as u64).collect::<Vec<_>>();
        let successor_generation = SUCCESSOR_SCAN_ENTRY_COUNT as u64 - 2;
        assert_eq!(
            retired_latest_generation_above(successor_generation, &generations),
            latest_generation_above(successor_generation, generations.iter().copied())
        );

        let mut retired_samples = Vec::with_capacity(OPTIMIZATION_BENCH_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(OPTIMIZATION_BENCH_SAMPLE_PAIRS);
        for pair_index in 0..OPTIMIZATION_BENCH_SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                retired_samples.push(measure_retired_successor_scan(
                    &generations,
                    successor_generation,
                ));
                optimized_samples.push(measure_single_pass_successor_scan(
                    &generations,
                    successor_generation,
                ));
            } else {
                optimized_samples.push(measure_single_pass_successor_scan(
                    &generations,
                    successor_generation,
                ));
                retired_samples.push(measure_retired_successor_scan(
                    &generations,
                    successor_generation,
                ));
            }
        }

        let retired_p95 = nearest_rank(&retired_samples, 95);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95
                .saturating_mul(10_000)
                .checked_div(retired_p95)
                .unwrap_or(0),
        );
        println!(
            "RUNTIME59_SINGLE_PASS_SUCCESSOR_GENERATION_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank entry_count={} iterations={} retired_generation_visits={} optimized_generation_visits={} retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
            OPTIMIZATION_BENCH_SAMPLE_PAIRS,
            SUCCESSOR_SCAN_ENTRY_COUNT,
            SUCCESSOR_SCAN_ITERATIONS,
            SUCCESSOR_SCAN_ENTRY_COUNT * 2,
            SUCCESSOR_SCAN_ENTRY_COUNT,
            retired_p95,
            optimized_p95,
            reduction_basis_points,
            join_samples(&retired_samples),
            join_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(4) <= retired_p95.saturating_mul(3),
            "single-pass successor scan P95 must be at most 75% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
        );
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

    fn measure_retired_tail_insertion(source: &VecDeque<u32>, incoming: u32) -> u128 {
        let mut queue = source.clone();
        let started = Instant::now();
        for _ in 0..ORDERED_INSERT_ITERATIONS {
            retired_insert_ordered_by(
                black_box(&mut queue),
                black_box(incoming),
                |queued, entry| queued > entry,
            );
            let _ = black_box(queue.pop_back());
        }
        started.elapsed().as_nanos()
    }

    fn measure_fast_tail_insertion(source: &VecDeque<u32>, incoming: u32) -> u128 {
        let mut queue = source.clone();
        let started = Instant::now();
        for _ in 0..ORDERED_INSERT_ITERATIONS {
            insert_ordered_by(
                black_box(&mut queue),
                black_box(incoming),
                |queued, entry| queued > entry,
            );
            let _ = black_box(queue.pop_back());
        }
        started.elapsed().as_nanos()
    }

    fn measure_retired_successor_scan(generations: &[u64], successor_generation: u64) -> u128 {
        let started = Instant::now();
        for _ in 0..SUCCESSOR_SCAN_ITERATIONS {
            let _ = black_box(retired_latest_generation_above(
                black_box(successor_generation),
                black_box(generations),
            ));
        }
        started.elapsed().as_nanos()
    }

    fn measure_single_pass_successor_scan(generations: &[u64], successor_generation: u64) -> u128 {
        let started = Instant::now();
        for _ in 0..SUCCESSOR_SCAN_ITERATIONS {
            let _ = black_box(latest_generation_above(
                black_box(successor_generation),
                black_box(generations).iter().copied(),
            ));
        }
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

    fn retired_insert_ordered_by<T>(
        queue: &mut VecDeque<T>,
        entry: T,
        should_insert_before: impl Fn(&T, &T) -> bool,
    ) {
        let insertion = queue
            .iter()
            .position(|queued| should_insert_before(queued, &entry))
            .unwrap_or(queue.len());
        queue.insert(insertion, entry);
    }

    fn retired_latest_generation_above(
        successor_generation: u64,
        generations: &[u64],
    ) -> Option<u64> {
        generations
            .iter()
            .any(|generation| *generation > successor_generation)
            .then(|| generations.iter().copied().max())
            .flatten()
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
