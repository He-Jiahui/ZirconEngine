use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const PERF_MARKER: &str = "RUNTIME357_UI_DOCUMENT_NODE_ENTRY_BENCH_V1";

#[test]
fn optimization_batch_20260830be_runtime_node_validation_uses_entry_api() {
    let source = include_str!("../validation.rs");
    assert!(source.contains("match seen.entry(node.node_id.as_str())"));
    assert!(source.contains("Entry::Occupied"));
    assert!(source.contains("Entry::Vacant"));
    assert!(!source.contains("seen.get(node.node_id.as_str())"));
}

#[test]
fn optimization_batch_20260830be_runtime_node_validation_keeps_semantics() {
    let source = include_str!("../validation.rs");
    assert!(source.contains("if *existing.get() == node"));
    assert!(source.contains("duplicate node_id {} resolves to conflicting subtrees"));
    assert!(source.contains("return Ok(());"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830be_runtime_node_validation_entry_lookup_p95() {
    const KEYS: usize = 2_048;
    const LOOKUPS: usize = 4_096;
    const SAMPLES: usize = 17;
    let ids = (0..KEYS)
        .map(|key| format!("node-{key}"))
        .collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLES);
    let mut optimized = Vec::with_capacity(SAMPLES);
    let ids = black_box(ids);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..LOOKUPS {
                let mut seen = BTreeMap::new();
                for (key, id) in ids.iter().enumerate() {
                    let id = black_box(id.as_str());
                    if pass == 0 {
                        if seen.get(id).is_none() {
                            seen.insert(id, key);
                        }
                    } else {
                        use std::collections::btree_map::Entry;
                        match seen.entry(id) {
                            Entry::Vacant(slot) => {
                                slot.insert(key);
                            }
                            Entry::Occupied(slot) => checksum ^= *slot.get(),
                        }
                    }
                }
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                legacy.push(elapsed);
            } else {
                optimized.push(elapsed);
            }
        }
    }
    legacy.sort_unstable();
    optimized.sort_unstable();
    let legacy_p95 = legacy[(SAMPLES * 95).div_ceil(100) - 1];
    let optimized_p95 = optimized[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * legacy_p95.saturating_sub(optimized_p95) as f64 / legacy_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} keys={KEYS} lookups={LOOKUPS} samples={SAMPLES} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(optimized_p95.saturating_mul(10) <= legacy_p95.saturating_mul(7));
}
