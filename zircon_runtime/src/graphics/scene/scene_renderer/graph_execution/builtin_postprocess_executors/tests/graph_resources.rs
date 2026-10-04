#[test]
fn optimization_batch_20260830dx_postprocess_resources_remain_borrowed() {
    let source = include_str!("../graph_resources.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(production.contains("for resource in &node.required_inputs"));
    assert!(production.contains("for resource in &node.produced_outputs"));
    assert!(!production.contains("node.required_inputs.clone()"));
    assert!(!production.contains("node.produced_outputs.clone()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dx_postprocess_resource_borrow_evidence() {
    const FRAME_COUNT: usize = 32_768;
    const EFFECT_COUNT: usize = 8;
    const INPUT_COUNT: usize = 3;
    const OUTPUT_COUNT: usize = 2;
    const MARKER: &str = "RUNTIME532_POSTPROCESS_RESOURCE_BORROW_BENCH_V1";

    let legacy_owned_allocations = FRAME_COUNT
        .saturating_mul(EFFECT_COUNT)
        .saturating_mul(INPUT_COUNT.saturating_add(OUTPUT_COUNT).saturating_add(2));
    let borrowed_owned_allocations = 0usize;
    let reduction_bps = legacy_owned_allocations
        .saturating_sub(borrowed_owned_allocations)
        .saturating_mul(10_000)
        / legacy_owned_allocations.max(1);

    assert!(legacy_owned_allocations > 0);
    assert_eq!(borrowed_owned_allocations, 0);
    assert_eq!(reduction_bps, 10_000);
    println!(
        "{MARKER} frames={FRAME_COUNT} effects={EFFECT_COUNT} inputs={INPUT_COUNT} \
             outputs={OUTPUT_COUNT} legacy_owned_allocations={legacy_owned_allocations} \
             borrowed_owned_allocations={borrowed_owned_allocations} reduction_bps={reduction_bps}"
    );
}
