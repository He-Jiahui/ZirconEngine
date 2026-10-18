use std::{hint::black_box, time::Instant};

use super::*;

const SAMPLE_PAIRS: usize = 21;
const QUERIES_PER_SAMPLE: usize = 40_000;
const THRESHOLD_PERCENT: u64 = 75;

#[derive(Clone, Copy)]
enum LookupKind {
    Any,
    Edge,
    Held,
}

fn legacy_lookup(keybinds: &Keybinds, value: &str, kind: LookupKind) -> Option<&'static str> {
    KEYBIND_ACTIONS
        .iter()
        .zip(&keybinds.slots)
        .find_map(|(action, slots)| {
            let matches = match kind {
                LookupKind::Any => slots.iter().flatten().any(|candidate| candidate == value),
                LookupKind::Edge => {
                    action.kind == KeyBindingKind::Edge
                        && slots.iter().flatten().any(|candidate| candidate == value)
                }
                LookupKind::Held => {
                    action.kind == KeyBindingKind::Held
                        && slots
                            .iter()
                            .flatten()
                            .any(|candidate| combo_code(candidate) == value)
                }
            };
            matches.then_some(action.id)
        })
}

fn query(sample: usize) -> (&'static str, LookupKind) {
    let action = (sample.wrapping_mul(37) + 19) % KEYBIND_ACTIONS.len();
    let value = if sample % 17 == 0 {
        "Missing"
    } else {
        KEYBIND_ACTIONS[action].defaults[0].expect("every action has a primary binding")
    };
    let kind = match sample % 3 {
        1 => LookupKind::Edge,
        2 => LookupKind::Held,
        _ => LookupKind::Any,
    };
    (value, kind)
}

fn measure_legacy(keybinds: &Keybinds) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for sample in 0..QUERIES_PER_SAMPLE {
        let (value, kind) = query(sample);
        let result = legacy_lookup(keybinds, black_box(value), kind)
            .map(str::len)
            .unwrap_or(127);
        checksum = checksum.rotate_left(5) ^ result.wrapping_add(sample);
    }
    black_box(checksum);
    started.elapsed().as_nanos() as u64
}

fn measure_indexed(keybinds: &Keybinds) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for sample in 0..QUERIES_PER_SAMPLE {
        let (value, kind) = query(sample);
        let action = match kind {
            LookupKind::Any => keybinds.action_for_combo(black_box(value)),
            LookupKind::Edge => keybinds.edge_action_for_combo(black_box(value)),
            LookupKind::Held => keybinds.held_action_for_code(black_box(value)),
        };
        checksum =
            checksum.rotate_left(5) ^ action.map(str::len).unwrap_or(127).wrapping_add(sample);
    }
    black_box(checksum);
    started.elapsed().as_nanos() as u64
}

fn sample_csv(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn reduction_percent(legacy: u64, indexed: u64) -> u64 {
    legacy.saturating_sub(indexed).saturating_mul(100) / legacy.max(1)
}

#[test]
#[ignore = "release performance evidence; run through the coordinator"]
fn woc_app04_keybind_reverse_index_release_benchmark_evidence() {
    let keybinds = Keybinds::default();
    for sample in 0..(KEYBIND_ACTIONS.len() * 3) {
        let (value, kind) = query(sample);
        let indexed = match kind {
            LookupKind::Any => keybinds.action_for_combo(value),
            LookupKind::Edge => keybinds.edge_action_for_combo(value),
            LookupKind::Held => keybinds.held_action_for_code(value),
        };
        assert_eq!(indexed, legacy_lookup(&keybinds, value, kind));
    }

    for _ in 0..4 {
        black_box(measure_legacy(&keybinds));
        black_box(measure_indexed(&keybinds));
    }

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&keybinds));
            indexed_ns.push(measure_indexed(&keybinds));
        } else {
            indexed_ns.push(measure_indexed(&keybinds));
            legacy_ns.push(measure_legacy(&keybinds));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let indexed_p50_ns = nearest_rank(&indexed_ns, 50);
    let indexed_p95_ns = nearest_rank(&indexed_ns, 95);
    let p50_reduction_percent = reduction_percent(legacy_p50_ns, indexed_p50_ns);
    let p95_reduction_percent = reduction_percent(legacy_p95_ns, indexed_p95_ns);

    println!(
        "WOC_APP04_KEYBIND_REVERSE_INDEX_PERF actions=61 queries_per_sample=40000 \
             sample_pairs=21 sample_order=alternating_legacy_first_even \
             percentile_method=nearest_rank threshold_percent=75 \
             legacy_p50_ns={legacy_p50_ns} indexed_p50_ns={indexed_p50_ns} \
             p50_reduction_percent={p50_reduction_percent} \
             legacy_p95_ns={legacy_p95_ns} indexed_p95_ns={indexed_p95_ns} \
             p95_reduction_percent={p95_reduction_percent} \
             legacy_ns={} indexed_ns={}",
        sample_csv(&legacy_ns),
        sample_csv(&indexed_ns)
    );

    assert!(
        p50_reduction_percent >= THRESHOLD_PERCENT,
        "reverse index must improve P50 by at least {THRESHOLD_PERCENT}%: \
             legacy={legacy_p50_ns}ns indexed={indexed_p50_ns}ns"
    );
    assert!(
        p95_reduction_percent >= THRESHOLD_PERCENT,
        "reverse index must improve P95 by at least {THRESHOLD_PERCENT}%: \
             legacy={legacy_p95_ns}ns indexed={indexed_p95_ns}ns"
    );
}
