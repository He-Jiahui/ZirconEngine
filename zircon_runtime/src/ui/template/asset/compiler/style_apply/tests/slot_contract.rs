use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_dy_borrowed_mui_slot_name_preserves_precedence_and_trimming() {
    let mut node = UiTemplateNode::default();
    node.attributes.insert(
        "mui_slot".to_owned(),
        Value::String(" attribute-slot ".to_owned()),
    );
    node.slot_attributes.insert(
        "mui_slot".to_owned(),
        Value::String("  primary-slot  ".to_owned()),
    );

    let slot = mui_slot_name(&node).expect("slot name");
    let stored = node
        .slot_attributes
        .get("mui_slot")
        .and_then(Value::as_str)
        .expect("stored slot")
        .trim();

    assert_eq!(slot, "primary-slot");
    assert_eq!(slot.as_ptr(), stored.as_ptr());
}

#[test]
fn optimization_batch_dy_mui_slot_name_uses_borrowed_storage() {
    let production = include_str!("../slot_contract.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("slot contract production source");
    let lookup = production
        .split("fn mui_slot_name")
        .nth(1)
        .expect("MUI slot-name lookup");

    assert!(lookup.contains("Option<&str>"));
    assert!(lookup.contains("borrowed_string_from_map"));
    assert!(!lookup.contains("Option<String>"));
    assert!(!lookup.contains("str::to_string"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dy_borrowed_mui_slot_name_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const LOOKUPS_PER_SAMPLE: usize = 16;
    const NODE_COUNT: usize = 2_048;

    let nodes = (0..NODE_COUNT)
        .map(|index| {
            let mut node = UiTemplateNode::default();
            node.slot_attributes.insert(
                "mui_slot".to_owned(),
                Value::String(format!(
                    "  slot-{index:04}-{}  ",
                    "long_component_slot_name/".repeat(16)
                )),
            );
            node
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_slot_lookups(&nodes, LOOKUPS_PER_SAMPLE, false));
            optimized_samples.push(measure_slot_lookups(&nodes, LOOKUPS_PER_SAMPLE, true));
        } else {
            optimized_samples.push(measure_slot_lookups(&nodes, LOOKUPS_PER_SAMPLE, true));
            legacy_samples.push(measure_slot_lookups(&nodes, LOOKUPS_PER_SAMPLE, false));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME433_BORROWED_MUI_SLOT_NAME_BENCH_V1 lookups_per_sample={LOOKUPS_PER_SAMPLE} node_count={NODE_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "borrowed MUI slot-name p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn measure_slot_lookups(nodes: &[UiTemplateNode], lookup_count: usize, optimized: bool) -> u128 {
    let started_at = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..lookup_count {
        for node in nodes {
            if optimized {
                let slot = mui_slot_name(node).expect("optimized slot name");
                checksum = checksum.wrapping_add(black_box(slot).len());
            } else {
                let slot = legacy_mui_slot_name(node).expect("legacy slot name");
                checksum = checksum.wrapping_add(black_box(slot).len());
            }
        }
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn legacy_mui_slot_name(node: &UiTemplateNode) -> Option<String> {
    legacy_string_from_map(&node.slot_attributes, "mui_slot")
        .or_else(|| legacy_string_from_map(&node.attributes, "mui_slot"))
}

fn legacy_string_from_map(values: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    values
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
