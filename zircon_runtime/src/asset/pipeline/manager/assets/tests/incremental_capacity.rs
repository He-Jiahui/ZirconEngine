use std::hint::black_box;
use std::time::{Duration, Instant};

const ENTRY_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;
const INCREMENTAL_PREPARE_SIGNATURE: &str =
    "pub(super) fn prepare_incremental_project_resource_sync(";
const INCREMENTAL_COMMIT_SIGNATURE: &str =
    "pub(super) fn commit_incremental_project_resource_sync<T>";
const CAPACITY_RESERVATIONS: [&str; 4] = [
    "HashMap::with_capacity(updated_records.len())",
    "Vec::with_capacity(previous_source_records.len())",
    "HashMap::with_capacity(records_by_id.len())",
    "HashSet::with_capacity(previous_source_records.len())",
];
const PROJECTION_CONTRACT: [&str; 4] = [
    "records_by_id.insert(record.id(), record)",
    "record_updates: records_by_id.into_values().collect()",
    "source_path_updates: source_path_updates.into_iter().collect()",
    "source_path_removals.into_iter().collect::<Vec<_>>()",
];

fn geometric_growth_events(len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let mut capacity = 4usize;
    let mut growth_events = 1usize;
    while capacity < len {
        capacity = capacity.saturating_mul(2).max(4);
        growth_events += 1;
    }
    growth_events
}

fn reserved_growth_events(_len: usize) -> usize {
    0
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn incremental_projection_body(source: &str) -> Option<&str> {
    if source.matches(INCREMENTAL_PREPARE_SIGNATURE).count() != 1
        || source.matches(INCREMENTAL_COMMIT_SIGNATURE).count() != 1
    {
        return None;
    }
    let (_, preparation) = source.split_once(INCREMENTAL_PREPARE_SIGNATURE)?;
    let (preparation, _) = preparation.split_once(INCREMENTAL_COMMIT_SIGNATURE)?;
    let opening = preparation.find('{')?;
    let mut depth = 0usize;
    for (offset, character) in preparation.char_indices() {
        if offset < opening {
            continue;
        }
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(&preparation[opening + 1..offset]);
                }
            }
            _ => {}
        }
    }
    None
}

fn incremental_capacity_source_contract(source: &str) -> Result<(), &'static str> {
    let body = incremental_projection_body(source)
        .ok_or("missing or unterminated incremental preparation body")?;
    for expression in CAPACITY_RESERVATIONS.into_iter().chain(PROJECTION_CONTRACT) {
        if !body.contains(expression) {
            return Err(expression);
        }
    }
    for unreserved in ["HashMap::new()", "HashSet::new()"] {
        if body.contains(unreserved) {
            return Err(unreserved);
        }
    }
    Ok(())
}

#[test]
fn optimization_batch_20260915_runtime786_incremental_publication_capacity_source_shape() {
    let source = include_str!("../resource_publication.rs");
    let production = source.split("#[cfg(test)]").next().expect("production");
    // Runtime206 reserves the four slice-bounded incremental projections. Sibling
    // publication methods have their own allocation contracts.
    assert_eq!(incremental_capacity_source_contract(production), Ok(()));

    let body = CAPACITY_RESERVATIONS
        .into_iter()
        .chain(PROJECTION_CONTRACT)
        .collect::<Vec<_>>()
        .join(";\n");
    let preparation = format!("{INCREMENTAL_PREPARE_SIGNATURE}) {{\n{body}\n}}");
    let commit = format!("{INCREMENTAL_COMMIT_SIGNATURE}() {{}}");
    let sibling = "fn sibling() { let _ = HashMap::new(); let _ = HashSet::new(); }";
    let fixture = format!("{preparation}\n{sibling}\n{commit}");
    assert_eq!(incremental_capacity_source_contract(&fixture), Ok(()));

    for (expression, unreserved) in CAPACITY_RESERVATIONS.into_iter().zip([
        "HashMap::new()",
        "Vec::new()",
        "HashMap::new()",
        "HashSet::new()",
    ]) {
        let regressed = preparation.replacen(expression, unreserved, 1);
        let fixture = format!("{regressed}\n{sibling}\n{commit}");
        assert_eq!(
            incremental_capacity_source_contract(&fixture),
            Err(expression)
        );
    }
    for expression in CAPACITY_RESERVATIONS.into_iter().chain(PROJECTION_CONTRACT) {
        let relocated = preparation.replacen(expression, "", 1);
        let fixture = format!("{relocated}\nfn sibling() {{ {expression}; }}\n{commit}");
        assert_eq!(
            incremental_capacity_source_contract(&fixture),
            Err(expression)
        );
    }
    for unreserved in ["HashMap::new()", "HashSet::new()"] {
        let regressed = preparation.replacen('{', &format!("{{ let _ = {unreserved};"), 1);
        let fixture = format!("{regressed}\n{sibling}\n{commit}");
        assert_eq!(
            incremental_capacity_source_contract(&fixture),
            Err(unreserved)
        );
    }
    for signature in [INCREMENTAL_PREPARE_SIGNATURE, INCREMENTAL_COMMIT_SIGNATURE] {
        let missing = fixture.replacen(signature, "fn other", 1);
        assert!(incremental_capacity_source_contract(&missing).is_err());
        let duplicated = format!("{fixture}\n{signature}");
        assert!(incremental_capacity_source_contract(&duplicated).is_err());
    }
    let unterminated = format!("{INCREMENTAL_PREPARE_SIGNATURE}) {{\n{body}\n{commit}");
    assert!(incremental_capacity_source_contract(&unterminated).is_err());
}

#[test]
fn optimization_batch_20260915_runtime786_incremental_publication_capacity_upper_bound() {
    for length in [0, 1, 4, 65_536] {
        let mut values = Vec::with_capacity(length);
        values.extend(0..length);
        assert!(values.capacity() >= length);
        assert_eq!(reserved_growth_events(values.len()), 0);
    }
}

fn measure_projection_shape(
    updated_count: usize,
    previous_count: usize,
    reserved: bool,
) -> Duration {
    let started = Instant::now();
    let mut records_by_id = if reserved {
        std::collections::HashMap::with_capacity(updated_count)
    } else {
        std::collections::HashMap::new()
    };
    for id in 0..updated_count {
        records_by_id.insert(black_box(id), id);
    }
    let mut removed_locators = if reserved {
        Vec::with_capacity(previous_count)
    } else {
        Vec::new()
    };
    for id in 0..previous_count {
        removed_locators.push(black_box(id));
    }
    let mut source_path_updates = if reserved {
        std::collections::HashMap::with_capacity(records_by_id.len())
    } else {
        std::collections::HashMap::new()
    };
    for id in records_by_id.keys().copied() {
        source_path_updates.insert(id, id);
    }
    let mut source_path_removals = if reserved {
        std::collections::HashSet::with_capacity(previous_count)
    } else {
        std::collections::HashSet::new()
    };
    for id in 0..previous_count {
        source_path_removals.insert(id);
    }
    black_box((
        records_by_id,
        removed_locators,
        source_path_updates,
        source_path_removals,
    ));
    started.elapsed().max(Duration::from_nanos(1))
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260915_runtime786_incremental_publication_capacity_p95() {
    let input_lengths = [
        ENTRY_COUNT,
        ENTRY_COUNT / 2,
        ENTRY_COUNT / 4,
        ENTRY_COUNT / 8,
    ];
    let legacy_events: usize = input_lengths
        .iter()
        .map(|length| geometric_growth_events(*length) * 4)
        .sum();
    let optimized_events: usize = input_lengths
        .iter()
        .map(|length| reserved_growth_events(*length))
        .sum();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let (legacy_nanos, optimized_nanos) = if sample % 2 == 0 {
            let legacy = measure_projection_shape(ENTRY_COUNT, ENTRY_COUNT, false);
            let optimized = measure_projection_shape(ENTRY_COUNT, ENTRY_COUNT, true);
            (legacy, optimized)
        } else {
            let optimized = measure_projection_shape(ENTRY_COUNT, ENTRY_COUNT, true);
            let legacy = measure_projection_shape(ENTRY_COUNT, ENTRY_COUNT, false);
            (legacy, optimized)
        };
        legacy_samples.push(legacy_nanos);
        optimized_samples.push(optimized_nanos);
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME786_INCREMENTAL_PUBLICATION_CAPACITY_BENCH_V1 entries={ENTRY_COUNT} \
         collection_families=4 legacy_growth_events={} optimized_growth_events={} \
         legacy_p95_ns={} optimized_p95_ns={}",
        legacy_events,
        reserved_growth_events(ENTRY_COUNT),
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(legacy_events > optimized_events);
    assert_eq!(optimized_events, 0);
}
