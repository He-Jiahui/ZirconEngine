use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use super::{IblBakeWgpuComputePipelineCacheKey, IblBakeWgpuPipelineCache};
use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactRequest, ProceduralSkyParams,
};
use crate::graphics::backend::RenderBackend;

use super::super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
use super::super::ibl_bake_wgpu_command_plan::{
    ibl_bake_wgpu_command_plan_for_request, IblBakeWgpuCommandPlan,
};

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ENTRY_COUNT: usize = 256;

fn request() -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        16,
        5,
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9)
}

fn pmrem_command(commands: &[IblBakeWgpuCommandPlan]) -> &IblBakeWgpuCommandPlan {
    commands
        .iter()
        .find(|command| command.kind == (IblBakeComputeKernelKind::Pmrem { mip_level: 0 }))
        .expect("PMREM mip zero command should exist")
}

#[test]
fn optimization_batch_20260826br_ibl_pipeline_fast_hit_reuses_cached_pipeline() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let plan = ibl_bake_wgpu_command_plan_for_request(&request());
    let command = pmrem_command(&plan.commands);
    let mut cache = IblBakeWgpuPipelineCache::new(&backend.device);

    let first = cache.ensure_compute_pipeline(&backend.device, command);
    let second = cache.ensure_compute_pipeline(&backend.device, command);

    assert_eq!(first, second);
    assert_eq!(cache.stats().shader_module_count, 1);
    assert_eq!(cache.stats().pipeline_layout_count, 1);
    assert_eq!(cache.stats().compute_pipeline_count, 1);
}

#[test]
fn optimization_batch_20260826br_ibl_pipeline_fast_hit_precedes_component_cache_probes() {
    let source = include_str!("../ibl_bake_wgpu_pipeline_cache.rs");
    let ensure = source
        .split_once("fn ensure_compute_pipeline(")
        .unwrap()
        .1
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let fast_hit = ensure
        .find("self.compute_pipelines.get(&pipeline_key)")
        .unwrap();
    let shader_probe = ensure
        .find("self.shader_modules.contains_key(&shader_key)")
        .unwrap();

    assert!(fast_hit < shader_probe);
    assert!(!ensure.contains("self.compute_pipelines.contains_key(&pipeline_key)"));
    assert_eq!(
        ensure
            .matches("self.compute_pipelines.get(&pipeline_key)")
            .count(),
        1
    );
}

fn run_legacy_workload(
    shader_modules: &HashMap<crate::core::framework::render::ComputePipelineCacheKey, usize>,
    compute_pipelines: &HashMap<IblBakeWgpuComputePipelineCacheKey, usize>,
    key: &IblBakeWgpuComputePipelineCacheKey,
) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(shader_modules.contains_key(&key.pipeline));
        black_box(compute_pipelines.contains_key(key));
        black_box(compute_pipelines.get(key));
    }
    started.elapsed().as_nanos().max(1)
}

fn run_fast_workload(
    compute_pipelines: &HashMap<IblBakeWgpuComputePipelineCacheKey, usize>,
    key: &IblBakeWgpuComputePipelineCacheKey,
) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(compute_pipelines.get(key));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &mut [u128], numerator: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * numerator).div_ceil(100).saturating_sub(1);
    samples[rank]
}

#[test]
#[ignore = "release performance gate; managed validation only"]
fn optimization_batch_20260826br_ibl_pipeline_fast_hit_p95() {
    let plan = ibl_bake_wgpu_command_plan_for_request(&request());
    let command = pmrem_command(&plan.commands);
    let mut base = command.pipeline_key.clone();
    base.kernel = "ibl-pipeline-shared-kernel-prefix/".repeat(24);
    let mut shader_modules = HashMap::with_capacity(ENTRY_COUNT);
    let mut compute_pipelines = HashMap::with_capacity(ENTRY_COUNT);
    for index in 0..ENTRY_COUNT {
        let mut pipeline = base.clone();
        pipeline.content_hash = index as u64;
        let key = IblBakeWgpuComputePipelineCacheKey {
            pipeline: pipeline.clone(),
            output_kind: command.bind_group_layout_kind,
        };
        shader_modules.insert(pipeline, index);
        compute_pipelines.insert(key, index);
    }
    let mut target_pipeline = base;
    target_pipeline.content_hash = (ENTRY_COUNT - 1) as u64;
    let target = IblBakeWgpuComputePipelineCacheKey {
        pipeline: target_pipeline,
        output_kind: command.bind_group_layout_kind,
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut fast_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(run_legacy_workload(
                &shader_modules,
                &compute_pipelines,
                &target,
            ));
            fast_samples.push(run_fast_workload(&compute_pipelines, &target));
        } else {
            fast_samples.push(run_fast_workload(&compute_pipelines, &target));
            legacy_samples.push(run_legacy_workload(
                &shader_modules,
                &compute_pipelines,
                &target,
            ));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples.clone(), 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let fast_p50 = percentile(&mut fast_samples.clone(), 50);
    let fast_p95 = percentile(&mut fast_samples, 95);
    println!(
        "RUNTIME09F1_IBL_PIPELINE_FAST_HIT_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} fast_p50_ns={fast_p50} fast_p95_ns={fast_p95} probes_before={} probes_after={HIT_COUNT}",
        HIT_COUNT * 3
    );
    assert!(
        fast_p95 * 100 <= legacy_p95 * 50,
        "fast-hit P95 must be at least 50% below legacy probes: legacy={legacy_p95}ns fast={fast_p95}ns"
    );
}
