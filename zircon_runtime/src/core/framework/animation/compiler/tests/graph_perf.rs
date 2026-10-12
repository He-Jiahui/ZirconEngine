use super::*;

const SYNTHETIC_NODE_COUNT: usize = 32_768;

#[test]
fn optimization_batch_20260830cn_runtime_collect_nodes_reserves_exact_partitions() {
    let asset = AnimationGraphAsset {
        name: Some("capacity-contract".to_owned()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Output {
                source: "blend-a".to_owned(),
            },
            AnimationGraphNodeAsset::Blend {
                id: "blend-a".to_owned(),
                inputs: vec!["mask-b".to_owned()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Output {
                source: "mask-b".to_owned(),
            },
            AnimationGraphNodeAsset::Mask {
                id: "mask-b".to_owned(),
                input: "blend-a".to_owned(),
                target_ids: Vec::new(),
            },
        ],
    };
    let mut diagnostics = Vec::new();

    let (nodes, indexes, outputs) = collect_nodes(&asset, &mut diagnostics);

    assert!(diagnostics.is_empty());
    assert_eq!(
        nodes.iter().map(|node| node_id(node)).collect::<Vec<_>>(),
        ["blend-a", "mask-b"]
    );
    assert_eq!(indexes.get("blend-a"), Some(&0));
    assert_eq!(indexes.get("mask-b"), Some(&1));
    assert_eq!(outputs, ["blend-a", "mask-b"]);
    assert_eq!(nodes.capacity(), nodes.len());
    assert_eq!(outputs.capacity(), outputs.len());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cn_runtime_collect_nodes_capacity_evidence() {
    let output_flags = (0..SYNTHETIC_NODE_COUNT)
        .map(|index| index % 8 == 0)
        .collect::<Vec<_>>();
    let legacy_growth_events = collect_partition_growth_events(&output_flags, false);
    let optimized_growth_events = collect_partition_growth_events(&output_flags, true);

    println!(
        "RUNTIME501_ANIMATION_GRAPH_NODE_CAPACITY_BENCH_V1 nodes={SYNTHETIC_NODE_COUNT} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn collect_partition_growth_events(output_flags: &[bool], reserve_exact: bool) -> usize {
    let output_count = reserve_exact
        .then(|| output_flags.iter().filter(|is_output| **is_output).count())
        .unwrap_or_default();
    let node_count = reserve_exact
        .then(|| output_flags.len().saturating_sub(output_count))
        .unwrap_or_default();
    let mut nodes = Vec::with_capacity(node_count);
    let mut outputs = Vec::with_capacity(output_count);
    let mut growth_events = 0;
    for (index, is_output) in output_flags.iter().copied().enumerate() {
        let target = if is_output { &mut outputs } else { &mut nodes };
        let capacity = target.capacity();
        target.push(index);
        growth_events += usize::from(target.capacity() != capacity);
    }
    std::hint::black_box((nodes, outputs));
    growth_events
}
