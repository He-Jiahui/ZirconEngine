use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::parse_canvas_placement;

const PLACEMENT_KEYS: [&str; 6] = [
    "anchor",
    "anchor_max",
    "pivot",
    "position",
    "offset",
    "auto_size",
];

#[test]
fn optimization_batch_dm_canvas_placement_presence_semantics_are_preserved() {
    let empty = toml::map::Map::new();
    assert!(parse_canvas_placement(&empty, "root").unwrap().is_none());

    let mut auto_size_only = toml::map::Map::new();
    auto_size_only.insert("auto_size".to_string(), Value::Boolean(true));
    assert!(parse_canvas_placement(&auto_size_only, "root")
        .unwrap()
        .is_some());
}

#[test]
fn optimization_batch_dm_both_canvas_parsers_lookup_each_placement_field_once() {
    let template_source = include_str!("../slot_contract.rs");
    let v2_source = include_str!("../../../v2/surface_tree/slot.rs");

    for source in [template_source, v2_source] {
        let body = source
            .split("fn parse_canvas_placement")
            .nth(1)
            .expect("canvas placement parser")
            .split("fn parse_grid_placement")
            .next()
            .expect("canvas placement body");
        assert!(!body.contains("layout.contains_key"));
        for key in PLACEMENT_KEYS {
            let lookup = format!("layout.get(\"{key}\")");
            assert_eq!(
                body.matches(lookup.as_str()).count(),
                1,
                "placement field {key} should be looked up once"
            );
        }
    }
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dm_cached_canvas_placement_fields_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const PARSES_PER_SAMPLE: usize = 131_072;

    let mut layout = (0..64_u64)
        .map(|index| (format!("filler_{index:02}"), index))
        .collect::<BTreeMap<_, _>>();
    layout.insert("auto_size".to_string(), 1);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_canvas_field_lookups(
                &layout,
                PARSES_PER_SAMPLE,
                true,
            ));
            optimized_samples.push(measure_canvas_field_lookups(
                &layout,
                PARSES_PER_SAMPLE,
                false,
            ));
        } else {
            optimized_samples.push(measure_canvas_field_lookups(
                &layout,
                PARSES_PER_SAMPLE,
                false,
            ));
            legacy_samples.push(measure_canvas_field_lookups(
                &layout,
                PARSES_PER_SAMPLE,
                true,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME421_CACHED_CANVAS_PLACEMENT_FIELDS_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "cached canvas placement fields p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn measure_canvas_field_lookups(
    layout: &BTreeMap<String, u64>,
    parse_count: usize,
    legacy: bool,
) -> u128 {
    let started_at = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..parse_count {
        let values = if legacy {
            let has_placement = PLACEMENT_KEYS
                .iter()
                .any(|key| black_box(layout.contains_key(*key)));
            has_placement.then(|| PLACEMENT_KEYS.map(|key| layout.get(key).copied()))
        } else {
            let values = PLACEMENT_KEYS.map(|key| layout.get(key).copied());
            values.iter().any(Option::is_some).then_some(values)
        };
        checksum = checksum.wrapping_add(
            values
                .into_iter()
                .flatten()
                .flatten()
                .map(|value| black_box(value))
                .sum::<u64>(),
        );
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
