use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::TemplateActionPaneKey;

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 500_000;

#[test]
fn optimization_batch_js_editor658_action_lookup_reuses_the_slot_pane_key() {
    let registry_source = include_str!("../../template_action_registry.rs");
    let lookup = registry_source
        .split("pub(super) fn action_for_token")
        .nth(1)
        .and_then(|source| source.split("fn action_source_is_disabled").next())
        .expect("template action lookup implementation");
    let slot_source = include_str!("../../template_action_slot.rs");

    assert!(slot_source.contains("pane_key: TemplateActionPaneKey"));
    assert!(slot_source.contains("pub(super) fn pane_key(&self)"));
    assert!(lookup.contains(".get(slot.pane_key())"));
    assert!(!lookup.contains("TemplateActionPaneKey::new("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_js_editor658_reused_template_action_pane_key_bench() {
    let stored_key = TemplateActionPaneKey::new("scene.viewport", "scene.viewport.panel", None);
    let attributes = BTreeMap::from([(stored_key.clone(), 73_usize)]);
    for _ in 0..4 {
        black_box(measure(&attributes, &stored_key, false));
        black_box(measure(&attributes, &stored_key, true));
    }

    let mut allocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reused_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            allocated_samples.push(measure(&attributes, &stored_key, false));
            reused_samples.push(measure(&attributes, &stored_key, true));
        } else {
            reused_samples.push(measure(&attributes, &stored_key, true));
            allocated_samples.push(measure(&attributes, &stored_key, false));
        }
    }

    let allocated_p95_ns = percentile(&allocated_samples, 95);
    let reused_p95_ns = percentile(&reused_samples, 95);
    println!(
        "EDITOR658_REUSED_TEMPLATE_ACTION_PANE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} allocated_p95_ns={allocated_p95_ns} \
reused_p95_ns={reused_p95_ns} allocated_raw_ns={} reused_raw_ns={}",
        sample_csv(&allocated_samples),
        sample_csv(&reused_samples),
    );

    assert!(reused_p95_ns.saturating_mul(100) <= allocated_p95_ns.saturating_mul(40));
}

fn measure(
    attributes: &BTreeMap<TemplateActionPaneKey, usize>,
    stored_key: &TemplateActionPaneKey,
    reused: bool,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let value = if reused {
            attributes.get(black_box(stored_key))
        } else {
            let temporary = TemplateActionPaneKey::new(
                black_box("scene.viewport"),
                black_box("scene.viewport.panel"),
                None,
            );
            attributes.get(black_box(&temporary))
        };
        checksum ^= *value.expect("benchmark pane key should resolve");
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
