use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_du_v2_layered_layout_preserves_precedence() {
    let mut self_layout = toml::map::Map::new();
    self_layout.insert("width".to_string(), Value::Integer(320));
    self_layout.insert("height".to_string(), Value::Integer(180));
    self_layout.insert("gap".to_string(), Value::Integer(4));
    let mut slot_layout = toml::map::Map::new();
    slot_layout.insert("width".to_string(), Value::Integer(640));
    slot_layout.insert("height".to_string(), Value::Integer(360));
    slot_layout.insert("gap".to_string(), Value::Integer(12));

    let horizontal = V2LayeredLayoutTable::new(
        Some(&self_layout),
        Some(&slot_layout),
        Some(UiContainerKind::HorizontalBox(Default::default())),
    );
    assert_eq!(
        horizontal.get("width").and_then(Value::as_integer),
        Some(320)
    );
    assert_eq!(
        horizontal.get("height").and_then(Value::as_integer),
        Some(360)
    );
    assert_eq!(horizontal.get("gap").and_then(Value::as_integer), Some(12));

    let vertical = V2LayeredLayoutTable::new(
        Some(&self_layout),
        Some(&slot_layout),
        Some(UiContainerKind::VerticalBox(Default::default())),
    );
    assert_eq!(vertical.get("width").and_then(Value::as_integer), Some(640));
    assert_eq!(
        vertical.get("height").and_then(Value::as_integer),
        Some(180)
    );
}

#[test]
fn optimization_batch_du_v2_layout_avoids_table_materialization() {
    let production = include_str!("../layout.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("V2 layout production source");
    let merge = production
        .split("fn merged_layout_table")
        .nth(1)
        .expect("V2 layered layout function");

    assert!(merge.contains("V2LayeredLayoutTable::new"));
    assert!(merge.contains("self.slot_layout"));
    assert!(!merge.contains("self_layout.clone()"));
    assert!(!merge.contains("value.clone()"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_du_v2_layered_layout_lookup_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const LOOKUPS_PER_SAMPLE: usize = 4_096;
    const EXTRA_ATTRIBUTES: usize = 96;

    let mut self_layout = toml::map::Map::new();
    let mut slot_layout = toml::map::Map::new();
    for index in 0..EXTRA_ATTRIBUTES {
        self_layout.insert(
            format!("self_attribute_{index:03}"),
            Value::String(format!("self_value_{index:03}")),
        );
        slot_layout.insert(
            format!("slot_attribute_{index:03}"),
            Value::String(format!("slot_value_{index:03}")),
        );
    }
    for (key, self_value, slot_value) in [
        ("width", 320, 640),
        ("height", 180, 360),
        ("gap", 4, 12),
        ("z_index", 2, 8),
        ("clip", 0, 1),
    ] {
        self_layout.insert(key.to_string(), Value::Integer(self_value));
        slot_layout.insert(key.to_string(), Value::Integer(slot_value));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_layout_lookups(
                &self_layout,
                &slot_layout,
                LOOKUPS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(measure_layout_lookups(
                &self_layout,
                &slot_layout,
                LOOKUPS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(measure_layout_lookups(
                &self_layout,
                &slot_layout,
                LOOKUPS_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(measure_layout_lookups(
                &self_layout,
                &slot_layout,
                LOOKUPS_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME429_V2_LAYERED_LAYOUT_LOOKUP_BENCH_V1 lookups_per_sample={LOOKUPS_PER_SAMPLE} extra_attributes={EXTRA_ATTRIBUTES} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "V2 layered layout lookup p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );

    fn measure_layout_lookups(
        self_layout: &toml::map::Map<String, Value>,
        slot_layout: &toml::map::Map<String, Value>,
        lookup_count: usize,
        optimized: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_i64;
        for _ in 0..lookup_count {
            if optimized {
                let layout = V2LayeredLayoutTable::new(
                    Some(self_layout),
                    Some(slot_layout),
                    Some(UiContainerKind::HorizontalBox(Default::default())),
                );
                checksum = checksum.wrapping_add(layout_checksum(&layout));
                black_box(layout);
            } else {
                let mut merged = self_layout.clone();
                for (key, value) in slot_layout {
                    merged.insert(key.clone(), value.clone());
                }
                if let Some(width) = self_layout.get("width") {
                    merged.insert("width".to_string(), width.clone());
                }
                checksum = checksum.wrapping_add(
                    ["width", "height", "gap", "z_index", "clip"]
                        .into_iter()
                        .filter_map(|key| merged.get(key).and_then(Value::as_integer))
                        .sum::<i64>(),
                );
                black_box(&merged);
            }
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn layout_checksum(layout: &V2LayeredLayoutTable<'_>) -> i64 {
        ["width", "height", "gap", "z_index", "clip"]
            .into_iter()
            .filter_map(|key| layout.get(key).and_then(Value::as_integer))
            .sum()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
