use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetHeader, UiV2AssetKind, UiV2StyleRule, UiV2StyleSheet,
    UI_V2_ASSET_SCHEMA_VERSION,
};

use super::{collect_rules, retain_rules_by_pseudo_state};

#[test]
fn runtime785_v2_style_rule_capacity_preserves_order() {
    let mut document = document();
    document.stylesheets = vec![UiV2StyleSheet {
        id: "shared".to_owned(),
        rules: ["Button", "Button", "Button"]
            .into_iter()
            .map(|selector| UiV2StyleRule {
                id: None,
                selector: selector.to_owned(),
                set: Default::default(),
            })
            .collect(),
    }];

    let rules = collect_rules(&document).expect("valid rule selectors");
    assert_eq!(rules.len(), 3);
    assert_eq!(
        rules.iter().map(|rule| rule.order).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    black_box(rules);
}

#[test]
fn runtime853_v2_style_rule_filter_retain_preserves_order() {
    let mut document = document();
    document.stylesheets = vec![UiV2StyleSheet {
        id: "mixed".to_owned(),
        rules: ["Button", ":hover", "Label", ":active"]
            .into_iter()
            .map(|selector| UiV2StyleRule {
                id: None,
                selector: selector.to_owned(),
                set: Default::default(),
            })
            .collect(),
    }];

    let all_rules = collect_rules(&document).expect("valid mixed rule selectors");
    let static_orders = all_rules
        .iter()
        .filter(|rule| !rule.uses_pseudo_state())
        .map(|rule| rule.order)
        .collect::<Vec<_>>();
    let runtime_orders = all_rules
        .iter()
        .filter(|rule| rule.uses_pseudo_state())
        .map(|rule| rule.order)
        .collect::<Vec<_>>();
    let source_capacity = all_rules.capacity();

    let mut static_rules = all_rules.clone();
    retain_rules_by_pseudo_state(&mut static_rules, false);
    assert_eq!(
        static_rules
            .iter()
            .map(|rule| rule.order)
            .collect::<Vec<_>>(),
        static_orders
    );
    assert!(static_rules.capacity() >= source_capacity);

    let mut runtime_rules = all_rules;
    retain_rules_by_pseudo_state(&mut runtime_rules, true);
    assert_eq!(
        runtime_rules
            .iter()
            .map(|rule| rule.order)
            .collect::<Vec<_>>(),
        runtime_orders
    );
    assert!(runtime_rules.capacity() >= source_capacity);
}

fn document() -> UiV2AssetDocument {
    UiV2AssetDocument {
        asset: UiV2AssetHeader {
            kind: UiV2AssetKind::View,
            id: "runtime785-rule-capacity".to_owned(),
            version: UI_V2_ASSET_SCHEMA_VERSION,
            display_name: String::new(),
        },
        imports: Default::default(),
        tokens: BTreeMap::new(),
        root: None,
        nodes: BTreeMap::new(),
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime785_v2_style_rule_capacity_release_benchmark() {
    const RULES_PER_BUILD: usize = 2_048;
    const BUILDS_PER_SAMPLE: usize = 2_048;
    const SAMPLE_PAIRS: usize = 17;
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_rule_growth(
                RULES_PER_BUILD,
                BUILDS_PER_SAMPLE,
                true,
            ));
            optimized_ns.push(measure_rule_growth(
                RULES_PER_BUILD,
                BUILDS_PER_SAMPLE,
                false,
            ));
        } else {
            optimized_ns.push(measure_rule_growth(
                RULES_PER_BUILD,
                BUILDS_PER_SAMPLE,
                false,
            ));
            legacy_ns.push(measure_rule_growth(
                RULES_PER_BUILD,
                BUILDS_PER_SAMPLE,
                true,
            ));
        }
    }
    let legacy_growth_events = growth_events(RULES_PER_BUILD);
    let optimized_growth_events = 0;
    assert!(legacy_growth_events > optimized_growth_events);
    println!(
        "RUNTIME785_V2_STYLE_RULE_CAPACITY_BENCH_V1 rules_per_build={RULES_PER_BUILD} builds_per_sample={BUILDS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_p95_ns={} optimized_p95_ns={} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime853_v2_style_rule_filter_retain_release_benchmark() {
    const RULES_PER_BUILD: usize = 4_096;
    const BUILDS_PER_SAMPLE: usize = 1_024;
    const SAMPLE_PAIRS: usize = 17;
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_filter(RULES_PER_BUILD, BUILDS_PER_SAMPLE, true));
            optimized_ns.push(measure_filter(RULES_PER_BUILD, BUILDS_PER_SAMPLE, false));
        } else {
            optimized_ns.push(measure_filter(RULES_PER_BUILD, BUILDS_PER_SAMPLE, false));
            legacy_ns.push(measure_filter(RULES_PER_BUILD, BUILDS_PER_SAMPLE, true));
        }
    }
    println!(
        "RUNTIME853_V2_STYLE_RULE_FILTER_RETAIN_BENCH_V1 rules_per_build={RULES_PER_BUILD} builds_per_sample={BUILDS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_filter_buffers_per_build=1 optimized_filter_buffers_per_build=0 legacy_p50_ns={} legacy_p95_ns={} legacy_p99_ns={} optimized_p50_ns={} optimized_p95_ns={} optimized_p99_ns={} legacy_raw_ns={} optimized_raw_ns={}",
        percentile(&legacy_ns, 50),
        percentile(&legacy_ns, 95),
        percentile(&legacy_ns, 99),
        percentile(&optimized_ns, 50),
        percentile(&optimized_ns, 95),
        percentile(&optimized_ns, 99),
        csv(&legacy_ns),
        csv(&optimized_ns),
    );
}

fn measure_filter(rule_count: usize, builds: usize, legacy: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..builds {
        let mut rules = (0..rule_count).collect::<Vec<_>>();
        if legacy {
            let filtered = rules
                .iter()
                .copied()
                .filter(|rule| *rule % 2 == 0)
                .collect::<Vec<_>>();
            checksum = checksum.wrapping_add(filtered.len());
            black_box(filtered);
        } else {
            rules.retain(|rule| *rule % 2 == 0);
            checksum = checksum.wrapping_add(rules.len());
            black_box(rules);
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_rule_growth(rule_count: usize, builds: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..builds {
        let mut rules = if optimized {
            Vec::with_capacity(rule_count)
        } else {
            Vec::new()
        };
        for index in 0..rule_count {
            rules.push(index);
        }
        checksum = checksum.wrapping_add(rules.len());
        black_box(rules);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(rule_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=rule_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
