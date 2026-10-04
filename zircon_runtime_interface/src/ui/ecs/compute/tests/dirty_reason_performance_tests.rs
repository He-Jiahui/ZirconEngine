use std::collections::BTreeSet;

use super::{ProjectionDirtyReasonMask, UiPipelineDirtyReason};

const FULL_DIRTY_REASONS: [UiPipelineDirtyReason; 8] = [
    UiPipelineDirtyReason::Input,
    UiPipelineDirtyReason::Focus,
    UiPipelineDirtyReason::WidgetBehavior,
    UiPipelineDirtyReason::Text,
    UiPipelineDirtyReason::Style,
    UiPipelineDirtyReason::Layout,
    UiPipelineDirtyReason::Picking,
    UiPipelineDirtyReason::Render,
];

fn collect_dirty_reasons_with_tree(node_count: usize) -> Vec<UiPipelineDirtyReason> {
    let mut reasons = BTreeSet::new();
    for _ in 0..node_count {
        for reason in FULL_DIRTY_REASONS {
            reasons.insert(reason);
        }
    }
    reasons.into_iter().collect()
}

fn collect_dirty_reasons_with_mask(node_count: usize) -> Vec<UiPipelineDirtyReason> {
    let mut reasons = ProjectionDirtyReasonMask::default();
    for _ in 0..node_count {
        for reason in FULL_DIRTY_REASONS {
            reasons.insert(reason);
        }
    }
    reasons.into_vec()
}

#[test]
fn runtime_interface03_batch72_73_fixed_dirty_reason_mask_preserves_tree_set_order_and_deduplication(
) {
    assert_eq!(
        collect_dirty_reasons_with_mask(4_096),
        collect_dirty_reasons_with_tree(4_096),
    );
}

#[test]
#[ignore = "release-only fixed ECS dirty reason mask benchmark"]
fn runtime_interface03_batch72_73_fixed_ecs_dirty_reason_mask_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const NODE_COUNT: usize = 4_096;
    const LOOKUP_COUNT: usize = 64;
    const SAMPLE_COUNT: usize = 11;
    let mut tree_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut mask_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_tree = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(collect_dirty_reasons_with_tree(black_box(NODE_COUNT)));
            }
            started.elapsed().as_nanos()
        };
        let measure_mask = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(collect_dirty_reasons_with_mask(black_box(NODE_COUNT)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            tree_samples.push(measure_tree());
            mask_samples.push(measure_mask());
        } else {
            mask_samples.push(measure_mask());
            tree_samples.push(measure_tree());
        }
    }

    tree_samples.sort_unstable();
    mask_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_FIXED_ECS_DIRTY_REASON_MASK_BENCH_V1 nodes={NODE_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} tree_p95_ns={} mask_p95_ns={}",
        tree_samples[p95], mask_samples[p95],
    );
    assert!(
        mask_samples[p95].saturating_mul(5) <= tree_samples[p95].saturating_mul(4),
        "fixed dirty reason mask must improve P95 by at least 20%: tree={}ns mask={}ns",
        tree_samples[p95],
        mask_samples[p95],
    );
}
