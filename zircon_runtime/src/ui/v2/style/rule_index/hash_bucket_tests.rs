use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::template::UiSelector;
use zircon_runtime_interface::ui::v2::UiV2StyleDeclarationBlock;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 4_096;

#[test]
fn optimization_batch_20260826cd_selector_hash_buckets_preserve_candidate_order() {
    let rules = ["Button", ".primary", "#save", ":hover", "Label"]
        .into_iter()
        .enumerate()
        .map(|(order, selector)| resolved_rule(selector, order))
        .collect::<Vec<_>>();
    let node = SelectorPathNode {
        component: "Button".to_owned(),
        control_id: Some("save".to_owned()),
        classes: vec!["primary".to_owned()],
        states: vec!["hover".to_owned()],
        is_host: false,
    };

    let index = ResolvedRuleTerminalIndex::from_rules(&rules);
    let mut candidates = Vec::new();
    index.collect_candidate_indices(&node, &mut candidates);

    assert_eq!(candidates, vec![0, 1, 2, 3]);
}

#[test]
fn optimization_batch_20260826cd_selector_hash_buckets_keep_explicit_order_projection() {
    let source = include_str!("../rule_index.rs");

    assert!(source.contains("use std::collections::HashMap;"));
    assert!(source.contains("by_type: HashMap<String, Vec<usize>>"));
    assert!(source.contains("buckets: &mut HashMap<String, Vec<usize>>"));
    assert!(source.contains("candidates.sort_unstable();"));
    assert!(source.contains("candidates.dedup();"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826cd_selector_hash_buckets_p95() {
    const KEYS: usize = 16_384;
    let keys = (0..KEYS)
        .map(|index| {
            format!(
                "zircon.ui.selector.terminal.shared-prefix-for-realistic-style-names.{index:05}"
            )
        })
        .collect::<Vec<_>>();
    let legacy = keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, key)| (key, index))
        .collect::<BTreeMap<_, _>>();
    let optimized = keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, key)| (key, index))
        .collect::<HashMap<_, _>>();
    let target = keys.last().expect("benchmark key set must not be empty");

    let mut legacy_lookup = || repeated_lookup(&legacy, target);
    let mut optimized_lookup = || repeated_lookup(&optimized, target);
    assert_eq!(black_box(legacy_lookup()), black_box(optimized_lookup()));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(&mut legacy_lookup));
            optimized_ns.push(measure_ns(&mut optimized_lookup));
        } else {
            optimized_ns.push(measure_ns(&mut optimized_lookup));
            legacy_ns.push(measure_ns(&mut legacy_lookup));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "selector hash bucket lookup P95 must be at least 30% below BTreeMap: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME73_SELECTOR_HASH_BUCKETS_BENCH_V1 keys={KEYS} lookups_per_sample={LOOKUPS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn resolved_rule(selector: &str, order: usize) -> ResolvedRule {
    let selector = UiSelector::parse(selector).expect("test selector must parse");
    ResolvedRule {
        specificity: selector.specificity(),
        order,
        selector,
        set: UiV2StyleDeclarationBlock::default(),
        style_tokens: BTreeMap::new(),
    }
}

fn repeated_lookup<M>(map: &M, key: &str) -> usize
where
    M: LookupMap,
{
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum = checksum.wrapping_add(black_box(map.lookup(black_box(key))));
    }
    black_box(checksum)
}

trait LookupMap {
    fn lookup(&self, key: &str) -> usize;
}

impl LookupMap for BTreeMap<String, usize> {
    fn lookup(&self, key: &str) -> usize {
        *self.get(key).expect("legacy benchmark key must exist")
    }
}

impl LookupMap for HashMap<String, usize> {
    fn lookup(&self, key: &str) -> usize {
        *self.get(key).expect("optimized benchmark key must exist")
    }
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
