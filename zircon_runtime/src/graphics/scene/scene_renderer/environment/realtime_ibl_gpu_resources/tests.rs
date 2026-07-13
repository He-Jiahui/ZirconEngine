use super::*;
use crate::core::framework::render::ProceduralSkyParams;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::append_realtime_ibl_graph_plan;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::{
    RealtimeIblTimeSliceConfig, RealtimeIblTimeSliceScheduler,
};
use crate::render_graph::RenderGraphBuilder;

#[test]
fn gpu_resources_bind_only_live_compiled_graph_views_without_duplicate_textures() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let params = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(params.ibl_bake_key(), 16, 5);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(5, 2).expect("realtime config"),
    );
    scheduler.request_rebake(params.ibl_bake_key());
    let batch = scheduler.begin_frame(1).expect("initial full batch");
    let mut builder = RenderGraphBuilder::new("realtime-ibl-gpu-resources");
    let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime IBL graph");
    let gpu_resources = RealtimeIblGpuResources::new(&backend.device, &request);
    let mut execution_resources = RenderGraphExecutionResources::new();

    gpu_resources
        .bind_graph_plan(&plan, &graph, &mut execution_resources)
        .expect("live A/B resources should bind");
    let report = execution_resources
        .validate_materialized_graph_resources(&graph)
        .expect("resource binding report");

    assert!(report.is_complete(), "{report:?}");
    for resource in plan
        .ready
        .resources()
        .into_iter()
        .chain(plan.work.resources())
    {
        let is_live = graph
            .resource_lifetimes()
            .iter()
            .any(|lifetime| lifetime.name == resource.name);
        assert_eq!(
            execution_resources.has_bound_resource(&resource.name),
            is_live,
            "resource `{}` binding should match its compiled lifetime",
            resource.name
        );
    }
    assert_eq!(
        gpu_resources.slot_a.source.sampled_mips.len(),
        request.source_mip_count() as usize
    );
    assert_eq!(
        gpu_resources.slot_a.source.storage_mips.len(),
        request.source_mip_count() as usize
    );
    assert_eq!(
        gpu_resources.slot_a.pmrem.storage_mips.len(),
        request.pmrem_mip_count() as usize
    );
    assert_eq!(
        gpu_resources.slot_b.source.sampled_mips.len(),
        request.source_mip_count() as usize
    );
    assert_eq!(
        gpu_resources.slot_b.pmrem.storage_mips.len(),
        request.pmrem_mip_count() as usize
    );
}

#[test]
fn graph_and_gpu_mip_count_mismatch_is_rejected_before_submission() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let params = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(params.ibl_bake_key(), 16, 5);
    let gpu_request = IblBakeArtifactRequest::new(params.ibl_bake_key(), 8, 4);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(5, 2).expect("realtime config"),
    );
    scheduler.request_rebake(params.ibl_bake_key());
    let batch = scheduler.begin_frame(1).expect("initial full batch");
    let mut builder = RenderGraphBuilder::new("realtime-ibl-gpu-mismatch");
    let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
        .expect("realtime IBL graph plan");
    let gpu_resources = RealtimeIblGpuResources::new(&backend.device, &gpu_request);
    let mut execution_resources = RenderGraphExecutionResources::new();

    let graph = builder.compile().expect("realtime IBL graph");
    let error = gpu_resources
        .bind_graph_plan(&plan, &graph, &mut execution_resources)
        .expect_err("mismatched view counts must fail");

    assert!(error.contains("view counts do not match"), "{error}");
}
