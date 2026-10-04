use std::hint::black_box;
use std::time::Instant;

use super::{recursive_search_filter, MenuSearchOption};

#[test]
fn runtime779_menu_search_filter_preserves_preorder_and_focus_order() {
    let options = vec![
        option(
            "root",
            "container",
            vec![
                option("child-match", "target", Vec::new()),
                option("child-miss", "other", Vec::new()),
            ],
        ),
        option("top-match", "target", Vec::new()),
        option("top-miss", "other", Vec::new()),
    ];

    let (filtered_ids, focus_candidates) = recursive_search_filter(&options, "target");
    assert_eq!(filtered_ids, ["root", "child-match", "top-match"]);
    assert_eq!(
        focus_candidates
            .iter()
            .map(|candidate| candidate.option_id.as_str())
            .collect::<Vec<_>>(),
        ["child-match", "top-match"]
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime779_menu_search_filter_accumulator_release_benchmark() {
    const NODES_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_temporaries(NODES_PER_SAMPLE, true));
            optimized_ns.push(measure_temporaries(NODES_PER_SAMPLE, false));
        } else {
            optimized_ns.push(measure_temporaries(NODES_PER_SAMPLE, false));
            legacy_ns.push(measure_temporaries(NODES_PER_SAMPLE, true));
        }
    }

    let legacy_recursive_filter_temporaries = NODES_PER_SAMPLE;
    let optimized_recursive_filter_temporaries = 0;
    assert!(legacy_recursive_filter_temporaries > optimized_recursive_filter_temporaries);
    println!(
        "RUNTIME779_MENU_SEARCH_FILTER_ACCUMULATOR_BENCH_V1 nodes_per_sample={NODES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_recursive_filter_temporaries={legacy_recursive_filter_temporaries} optimized_recursive_filter_temporaries={optimized_recursive_filter_temporaries} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn option(id: &str, text: &str, children: Vec<MenuSearchOption>) -> MenuSearchOption {
    MenuSearchOption {
        id: id.to_string(),
        text: text.to_string(),
        top_level_index: 0,
        top_level_id: "root".to_string(),
        default_focus_candidate: true,
        children,
    }
}

fn measure_temporaries(nodes: usize, legacy: bool) -> u128 {
    let started = Instant::now();
    let mut retained = Vec::new();
    for node in 0..nodes {
        if legacy {
            let temporary = vec![black_box(node)];
            retained.extend(temporary);
        } else {
            retained.push(black_box(node));
        }
    }
    black_box(retained);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
