use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactRequest, ProceduralSkyParams,
};
use crate::graphics::backend::RenderBackend;

use super::super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
use super::super::ibl_bake_wgpu_command_plan::{
    ibl_bake_wgpu_command_plan_for_request, IblBakeWgpuCommandPlan,
};
use super::*;

#[test]
fn pipeline_cache_reuses_pmrem_shader_and_pipeline_across_mips() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let request = request(16, 5, IblBakeArtifactContents::PMREM_SH9);
    let plan = ibl_bake_wgpu_command_plan_for_request(&request);
    let pmrem_mip0 = command_for_kind(
        &plan.commands,
        IblBakeComputeKernelKind::Pmrem { mip_level: 0 },
    );
    let pmrem_mip1 = command_for_kind(
        &plan.commands,
        IblBakeComputeKernelKind::Pmrem { mip_level: 1 },
    );
    let sh9 = command_for_kind(&plan.commands, IblBakeComputeKernelKind::IrradianceSh9);
    let mut cache = IblBakeWgpuPipelineCache::new(&backend.device);

    let _ = cache.ensure_compute_pipeline(&backend.device, pmrem_mip0);
    assert_eq!(
        cache.stats(),
        IblBakeWgpuPipelineCacheStats {
            shader_module_count: 1,
            pipeline_layout_count: 1,
            compute_pipeline_count: 1,
        }
    );

    let _ = cache.ensure_compute_pipeline(&backend.device, pmrem_mip0);
    let _ = cache.ensure_compute_pipeline(&backend.device, pmrem_mip1);
    assert_eq!(
        cache.stats(),
        IblBakeWgpuPipelineCacheStats {
            shader_module_count: 1,
            pipeline_layout_count: 1,
            compute_pipeline_count: 1,
        }
    );

    let _ = cache.ensure_compute_pipeline(&backend.device, sh9);
    assert_eq!(
        cache.stats(),
        IblBakeWgpuPipelineCacheStats {
            shader_module_count: 2,
            pipeline_layout_count: 2,
            compute_pipeline_count: 2,
        }
    );
}

#[test]
fn pipeline_cache_owns_the_production_source_sampler() {
    let source = include_str!("../ibl_bake_wgpu_dispatch.rs");
    let realtime = include_str!("../realtime_ibl_wgpu_recorder.rs");

    assert!(!source.contains("create_ibl_bake_wgpu_source_sampler"));
    assert!(!realtime.contains("create_ibl_bake_wgpu_source_sampler"));
}

fn request(
    face_size: u32,
    mip_count: u32,
    contents: IblBakeArtifactContents,
) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        face_size,
        mip_count,
    )
    .with_required_contents(contents)
}

fn command_for_kind(
    commands: &[IblBakeWgpuCommandPlan],
    kind: IblBakeComputeKernelKind,
) -> &IblBakeWgpuCommandPlan {
    commands
        .iter()
        .find(|command| command.kind == kind)
        .unwrap_or_else(|| panic!("IBL bake command {kind:?} should exist"))
}
