use crate::graphics::feature::{RenderFeatureDescriptor, RenderFeaturePassDescriptor};
use crate::graphics::pipeline::declarations::RenderPassStage;

pub(in crate::graphics::pipeline) fn stage_pass_descriptors(
    stage: RenderPassStage,
    descriptors: &[RenderFeatureDescriptor],
) -> Vec<RenderFeaturePassDescriptor> {
    let pass_capacity = descriptors
        .iter()
        .map(|descriptor| descriptor.stage_passes.len())
        .sum();
    let mut passes = Vec::with_capacity(pass_capacity);
    passes.extend(
        descriptors
            .iter()
            .flat_map(|descriptor| descriptor.stage_passes.iter())
            .filter(|descriptor| descriptor.stage == stage)
            .cloned(),
    );
    passes
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::stage_pass_descriptors;
    use crate::graphics::feature::{RenderFeatureDescriptor, RenderFeaturePassDescriptor};
    use crate::graphics::pipeline::declarations::RenderPassStage;
    use crate::render_graph::QueueLane;

    #[test]
    fn optimization_batch_20260830cu_stage_pass_capacity_preserves_filtering_and_order() {
        let descriptors = test_descriptors();
        let selected = stage_pass_descriptors(RenderPassStage::Lighting, &descriptors);

        assert_eq!(selected.len(), 6);
        assert!(
            selected
                .iter()
                .all(|pass| pass.stage == RenderPassStage::Lighting)
        );
        assert_eq!(
            selected
                .iter()
                .map(|pass| pass.pass_name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "feature-0-pass-1",
                "feature-0-pass-3",
                "feature-1-pass-1",
                "feature-1-pass-3",
                "feature-2-pass-1",
                "feature-2-pass-3"
            ]
        );
    }

    #[test]
    fn optimization_batch_20260830cu_stage_pass_capacity_source_contract() {
        let source = include_str!("stage_pass_descriptors.rs");
        assert!(source.contains("let pass_capacity = descriptors"));
        assert!(source.contains("Vec::with_capacity(pass_capacity)"));
        assert!(source.contains("passes.extend("));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260830cu_runtime_stage_pass_capacity_p95() {
        let descriptors = benchmark_descriptors();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut optimized_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            for _ in 0..8 {
                black_box(legacy_stage_pass_descriptors(
                    RenderPassStage::Lighting,
                    &descriptors,
                ));
            }
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            for _ in 0..8 {
                black_box(stage_pass_descriptors(
                    RenderPassStage::Lighting,
                    &descriptors,
                ));
            }
            optimized_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let optimized_p95 = optimized_samples[16];
        println!(
            "RUNTIME396_STAGE_PASS_CAPACITY_BENCH_V1 descriptors={} passes_per_descriptor={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
            descriptors.len(),
            descriptors[0].stage_passes.len(),
            legacy_p95,
            optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "optimized stage-pass projection P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
        );
    }

    fn legacy_stage_pass_descriptors(
        stage: RenderPassStage,
        descriptors: &[RenderFeatureDescriptor],
    ) -> Vec<RenderFeaturePassDescriptor> {
        descriptors
            .iter()
            .flat_map(|descriptor| descriptor.stage_passes.iter())
            .filter(|descriptor| descriptor.stage == stage)
            .cloned()
            .collect()
    }

    fn test_descriptors() -> Vec<RenderFeatureDescriptor> {
        (0..3)
            .map(|feature_index| {
                RenderFeatureDescriptor::new(
                    format!("feature-{feature_index}"),
                    Vec::new(),
                    Vec::new(),
                    (0..4)
                        .map(|pass_index| {
                            let stage = if pass_index % 2 == 1 {
                                RenderPassStage::Lighting
                            } else {
                                RenderPassStage::Opaque3d
                            };
                            RenderFeaturePassDescriptor::new(
                                stage,
                                format!("feature-{feature_index}-pass-{pass_index}"),
                                QueueLane::Graphics,
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }

    fn benchmark_descriptors() -> Vec<RenderFeatureDescriptor> {
        (0..32)
            .map(|feature_index| {
                RenderFeatureDescriptor::new(
                    format!("bench-feature-{feature_index}"),
                    Vec::new(),
                    Vec::new(),
                    (0..256)
                        .map(|pass_index| {
                            let stage = if pass_index % 2 == 1 {
                                RenderPassStage::Lighting
                            } else {
                                RenderPassStage::Opaque3d
                            };
                            RenderFeaturePassDescriptor::new(stage, "", QueueLane::Graphics)
                        })
                        .collect(),
                )
            })
            .collect()
    }
}
