use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime::core::framework::animation::AnimationGraphNodeAsset;
use zircon_runtime::core::resource::{AssetReference, ResourceLocator};

use super::graph_node_label;

const PERFORMANCE_MARKER: &str = "EDITOR890_ANIMATION_GRAPH_LABEL_SINGLE_BUFFER_BENCH_V1";
const SAMPLE_PAIRS: usize = 101;
const LABELS_PER_SAMPLE: usize = 4_096;

#[test]
fn editor890_animation_graph_label_single_buffer_preserves_exact_text() {
    let nodes = vec![
        AnimationGraphNodeAsset::Clip {
            id: "idle".to_string(),
            clip: AssetReference::from_locator(
                ResourceLocator::parse("res://animation/hero_idle.clip")
                    .expect("fixture locator should be canonical"),
            ),
            playback_speed: 1.0,
            looping: true,
        },
        AnimationGraphNodeAsset::Blend {
            id: "empty".to_string(),
            inputs: Vec::new(),
            weight_parameter: None,
        },
        AnimationGraphNodeAsset::Blend {
            id: "locomotion".to_string(),
            inputs: vec!["idle".to_string(), String::new(), "run".to_string()],
            weight_parameter: Some("speed".to_string()),
        },
        AnimationGraphNodeAsset::Additive {
            id: "aim".to_string(),
            base: "locomotion".to_string(),
            additive: "upper_body".to_string(),
            weight_parameter: None,
        },
        AnimationGraphNodeAsset::Mask {
            id: "empty_mask".to_string(),
            input: "aim".to_string(),
            target_ids: Vec::new(),
        },
        AnimationGraphNodeAsset::Mask {
            id: "masked".to_string(),
            input: "aim".to_string(),
            target_ids: vec!["spine".to_string(), String::new(), "head".to_string()],
        },
        AnimationGraphNodeAsset::Output {
            source: "masked".to_string(),
        },
    ];

    for node in &nodes {
        assert_eq!(graph_node_label(node), legacy_graph_node_label(node));
    }
    assert_eq!(
        graph_node_label(&nodes[2]),
        "Blend locomotion • idle, , run"
    );
    assert_eq!(graph_node_label(&nodes[4]), "Mask empty_mask • aim []");
}

#[test]
#[ignore = "release-only animation graph label performance gate"]
fn editor890_animation_graph_label_single_buffer_release_performance() {
    let node = AnimationGraphNodeAsset::Blend {
        id: "dense_blend".to_string(),
        inputs: (0..64)
            .map(|index| format!("animation_graph_input_{index:04}"))
            .collect(),
        weight_parameter: Some("blend_weight".to_string()),
    };
    assert_eq!(graph_node_label(&node), legacy_graph_node_label(&node));

    for _ in 0..8 {
        black_box(render_batch(&node, legacy_graph_node_label));
        black_box(render_batch(&node, graph_node_label));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| render_batch(&node, legacy_graph_node_label)));
            optimized_samples.push(measure(|| render_batch(&node, graph_node_label)));
        } else {
            optimized_samples.push(measure(|| render_batch(&node, graph_node_label)));
            legacy_samples.push(measure(|| render_batch(&node, legacy_graph_node_label)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p99_ns={optimized_p99_ns} sample_pairs={SAMPLE_PAIRS} labels_per_sample={LABELS_PER_SAMPLE} ids_per_label=64 legacy_join_outputs_per_sample=4096 optimized_join_outputs_per_sample=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(110),
        "single-buffer P95 {optimized_p95_ns}ns must be at most 110% of join/format P95 {legacy_p95_ns}ns"
    );
}

fn legacy_graph_node_label(node: &AnimationGraphNodeAsset) -> String {
    match node {
        AnimationGraphNodeAsset::Clip { id, clip, .. } => {
            format!("Clip {id} • {}", clip.locator)
        }
        AnimationGraphNodeAsset::Blend { id, inputs, .. } => {
            if inputs.is_empty() {
                format!("Blend {id}")
            } else {
                format!("Blend {id} • {}", inputs.join(", "))
            }
        }
        AnimationGraphNodeAsset::Additive {
            id, base, additive, ..
        } => format!("Additive {id} • {base} + {additive}"),
        AnimationGraphNodeAsset::Mask {
            id,
            input,
            target_ids,
        } => format!("Mask {id} • {input} [{}]", target_ids.join(", ")),
        AnimationGraphNodeAsset::Output { source } => format!("Output <- {source}"),
    }
}

fn render_batch(
    node: &AnimationGraphNodeAsset,
    render: fn(&AnimationGraphNodeAsset) -> String,
) -> usize {
    (0..LABELS_PER_SAMPLE)
        .map(|_| black_box(render(black_box(node))).len())
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
