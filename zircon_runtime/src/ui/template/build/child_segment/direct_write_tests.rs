use std::hint::black_box;
use std::time::Instant;

use super::child_segment;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const SAMPLE_PAIRS: usize = 21;
const OPERATIONS_PER_SAMPLE: usize = 2_048;

#[test]
fn optimization_batch_20260826hb_runtime248_preserves_segment_source_and_sanitization() {
    let control_node = UiTemplateNode {
        control_id: Some("menu/root item:active#1".to_string()),
        component: Some("IgnoredComponent".to_string()),
        ..UiTemplateNode::default()
    };
    assert_eq!(
        child_segment(&control_node, 42),
        "menu_root_item_active_1_42"
    );

    let component_node = UiTemplateNode {
        component: Some("Panel\\Body".to_string()),
        ..UiTemplateNode::default()
    };
    assert_eq!(child_segment(&component_node, 0), "Panel_Body_0");
    assert_eq!(child_segment(&UiTemplateNode::default(), 7), "node_7");
}

#[test]
fn optimization_batch_20260826hb_runtime248_writes_the_final_buffer_directly() {
    let source = include_str!("../child_segment.rs");
    let start = source
        .find("fn child_segment(")
        .expect("child_segment function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let body = &source[start..end];

    assert!(body.contains("String::with_capacity"));
    assert!(body.contains("write!("));
    assert!(!body.contains("collect::<String>()"));
    assert!(!body.contains("format!("));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hb_runtime248_child_segment_direct_write_release_benchmark() {
    let node = UiTemplateNode {
        control_id: Some(
            (0..128)
                .map(|index| format!("control/{index:03}:active#slot "))
                .collect::<String>(),
        ),
        ..UiTemplateNode::default()
    };
    assert_eq!(
        child_segment(&node, usize::MAX),
        legacy_child_segment(&node, usize::MAX)
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for index in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_child_segment(black_box(&node), black_box(index)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for index in 0..OPERATIONS_PER_SAMPLE {
                black_box(child_segment(black_box(&node), black_box(index)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME248_CHILD_SEGMENT_DIRECT_WRITE_BENCH_V1 input_bytes={} \
         operations_per_sample={OPERATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
         legacy_ns={} optimized_ns={}",
        node.control_id.as_deref().unwrap().len(),
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_child_segment(node: &UiTemplateNode, index: usize) -> String {
    let raw = node
        .control_id
        .as_deref()
        .or(node.component.as_deref())
        .unwrap_or("node");
    let sanitized = raw
        .chars()
        .map(|character| match character {
            '/' | '\\' | ' ' | ':' | '#' => '_',
            _ => character,
        })
        .collect::<String>();
    format!("{sanitized}_{index}")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
