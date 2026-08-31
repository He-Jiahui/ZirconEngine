use std::collections::HashMap;

use crate::core::framework::render::{
    RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile, RenderViewportHandle,
};
use crate::graphics::RenderPipelineAsset;

use super::super::capability_validation::{
    validate_compiled_pipeline_capabilities, validate_quality_profile_capabilities,
};
use super::super::register_pipeline_asset::compile_pipeline_for_validation;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn set_quality_profile(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    profile: RenderQualityProfile,
) -> Result<(), RenderFrameworkError> {
    let _operation_guard = framework.lock_operation();
    let (capabilities, effective_pipeline, pipeline_asset) = {
        let state = framework.lock_state();
        let active_pipeline = state
            .viewports
            .get(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?
            .pipeline();
        let effective_pipeline = active_pipeline.or(profile.pipeline_override);
        let pipeline_asset = pipeline_asset_for_profile(&state.pipelines, effective_pipeline)?;
        (
            state.stats.capabilities.clone(),
            effective_pipeline,
            pipeline_asset,
        )
    };
    let compiled = pipeline_asset
        .as_ref()
        .map(compile_pipeline_for_validation)
        .transpose()?;
    let profile_name = profile.name.clone();
    let mut state = framework.lock_state();
    if let Some((pipeline, compiled)) = effective_pipeline.zip(compiled.as_ref()) {
        state
            .renderer
            .validate_compiled_pipeline_executors(compiled)
            .map_err(|message| RenderFrameworkError::GraphCompileFailure {
                pipeline: pipeline.raw(),
                message,
            })?;
        validate_compiled_pipeline_capabilities(compiled, &capabilities)?;
    }
    validate_quality_profile_capabilities(effective_pipeline, &profile, &capabilities)?;
    let record = state
        .viewports
        .get_mut(&viewport)
        .expect("viewport checked above");
    record.set_quality_profile(profile);
    state.stats.last_quality_profile = Some(profile_name);
    Ok(())
}

fn pipeline_asset_for_profile(
    pipelines: &HashMap<RenderPipelineHandle, RenderPipelineAsset>,
    pipeline: Option<RenderPipelineHandle>,
) -> Result<Option<RenderPipelineAsset>, RenderFrameworkError> {
    pipeline
        .map(|pipeline| {
            pipelines
                .get(&pipeline)
                .cloned()
                .ok_or(RenderFrameworkError::UnknownPipeline {
                    pipeline: pipeline.raw(),
                })
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::core::framework::render::{
        RenderFramework, RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile,
        RenderViewportDescriptor,
    };
    use crate::core::math::UVec2;
    use crate::graphics::{
        BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassStage, RenderPipelineAsset, WgpuRenderFramework,
    };
    use crate::render_graph::QueueLane;

    use super::{pipeline_asset_for_profile, set_quality_profile};

    #[test]
    fn optimization_batch_dl_pipeline_asset_lookup_preserves_profile_semantics() {
        let handle = RenderPipelineHandle::new(73);
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline.handle = handle;
        let pipelines = HashMap::from([(handle, pipeline)]);

        assert_eq!(
            pipeline_asset_for_profile(&pipelines, Some(handle))
                .unwrap()
                .expect("registered pipeline")
                .handle,
            handle
        );
        assert!(pipeline_asset_for_profile(&pipelines, None)
            .unwrap()
            .is_none());
        assert_eq!(
            pipeline_asset_for_profile(&pipelines, Some(RenderPipelineHandle::new(74)))
                .unwrap_err(),
            RenderFrameworkError::UnknownPipeline { pipeline: 74 }
        );
    }

    #[test]
    fn optimization_batch_dl_quality_profile_uses_one_pipeline_lookup() {
        let source = include_str!("set_quality_profile.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("quality profile production source");

        assert!(!production.contains("state.pipelines.contains_key(&pipeline)"));
        assert!(
            production.contains("pipeline_asset_for_profile(&state.pipelines, effective_pipeline)")
        );
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dl_single_quality_pipeline_lookup_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const LOOKUPS_PER_SAMPLE: usize = 262_144;
        const PIPELINE_COUNT: usize = 4_096;

        let pipelines = (0..PIPELINE_COUNT as u64)
            .map(|pipeline| (pipeline, pipeline.wrapping_mul(17)))
            .collect::<HashMap<_, _>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_pipeline_lookups(
                    &pipelines,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
                optimized_samples.push(measure_pipeline_lookups(
                    &pipelines,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples.push(measure_pipeline_lookups(
                    &pipelines,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
                legacy_samples.push(measure_pipeline_lookups(
                    &pipelines,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME420_SINGLE_QUALITY_PIPELINE_LOOKUP_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "single quality pipeline lookup p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_pipeline_lookups(
        pipelines: &HashMap<u64, u64>,
        lookup_count: usize,
        legacy: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for index in 0..lookup_count {
            let pipeline = black_box((index % pipelines.len()) as u64);
            let value = if legacy {
                black_box(pipelines.contains_key(&pipeline))
                    .then(|| pipelines.get(&pipeline).copied())
                    .flatten()
            } else {
                pipelines.get(&pipeline).copied()
            };
            checksum = checksum.wrapping_add(black_box(value.unwrap_or_default()));
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    #[test]
    fn set_quality_profile_compiles_outside_framework_state_lock() {
        let source = include_str!("set_quality_profile.rs");
        let compile = source
            .find(concat!("let compiled = ", "pipeline_asset"))
            .expect("quality profile should compile the validation graph");
        let snapshot = source[..compile]
            .rfind(concat!(
                "let (capabilities, effective_pipeline, pipeline_asset) = ",
                "{"
            ))
            .expect("pipeline asset should be snapshotted in a short lock scope");
        let relock = compile
            + source[compile..]
                .find(concat!("let mut state = framework.", "lock_state();"))
                .expect("framework state should be reacquired after compilation");

        assert!(snapshot < compile && compile < relock);
    }

    #[test]
    fn set_quality_profile_revalidates_override_graph_executor_contract() {
        let framework =
            WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
        let viewport = framework
            .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
            .unwrap();
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline.handle = RenderPipelineHandle::new(91);
        pipeline.name = "profile-override-invalid-executor-pipeline".to_string();
        let bloom = pipeline
            .renderer
            .features
            .iter_mut()
            .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
            .expect("default pipeline should include bloom");
        *bloom = bloom
            .clone()
            .with_descriptor_override(RenderFeatureDescriptor::new(
                "profile-override-invalid-executor-feature",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::PostProcess,
                    "profile-override-invalid-executor-pass",
                    QueueLane::Graphics,
                )
                .with_executor_id("custom.profile-override-missing-executor")
                .with_side_effects()],
            ));
        framework
            .lock_state()
            .pipelines
            .insert(pipeline.handle, pipeline);

        let error = set_quality_profile(
            &framework,
            viewport,
            RenderQualityProfile::new("profile-override-stale")
                .with_pipeline_asset(RenderPipelineHandle::new(91)),
        )
        .expect_err("quality profile override should re-run graph executor validation");

        assert_eq!(
            error,
            RenderFrameworkError::GraphCompileFailure {
                pipeline: 91,
                message:
                    "render pass `profile-override-invalid-executor-pass` references unregistered executor `custom.profile-override-missing-executor`"
                        .to_string(),
            }
        );
    }
}
