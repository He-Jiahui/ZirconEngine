use super::*;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_graph_plan::append_realtime_ibl_graph_plan;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::{
    RealtimeIblPrefilterDispatchSlice, RealtimeIblTimeSliceConfig, RealtimeIblTimeSliceScheduler,
};
use crate::render_graph::RenderGraphBuilder;

#[test]
fn invalid_pmrem_slice_reports_its_exact_bounds() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let slice = RealtimeIblPrefilterDispatchSlice {
        mip_level: request.pmrem_mip_count() as u8,
        first_face: 5,
        face_count: 2,
    };

    let error = prefilter_command(&request, slice).expect_err("slice must be rejected");

    assert!(error.contains("mip=8"), "{error}");
    assert!(error.contains("first_face=5"), "{error}");
    assert!(error.contains("face_count=2"), "{error}");
}

#[test]
fn realtime_ibl_timestamp_encoding_requires_explicit_gpu_timing() {
    let source = include_str!("../realtime_ibl_wgpu_recorder.rs");
    let normalized = source.split_whitespace().collect::<String>();
    let start = source
        .find("pub(in crate::graphics) fn record_graph_plan")
        .expect("realtime IBL recorder entrypoint");
    let record_graph_plan = &source[start..];

    assert!(record_graph_plan.contains("gpu_timing_enabled: bool"));
    assert!(record_graph_plan.contains("Self::timestamp_recorder("));
    assert!(record_graph_plan.contains("timestamp_recorder.map("));
    assert!(source.contains("timestamps: None"));
    assert!(normalized.contains("gpu_timing_enabled&&timestamps.is_none()"));
}

#[test]
#[ignore = "time-sliced realtime IBL WGPU product validation"]
fn capture_ticket_records_and_submits_without_wgpu_validation_errors() {
    let Ok(RenderBackend { device, queue, .. }) = RenderBackend::new_offscreen() else {
        return;
    };
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("realtime config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let batch = scheduler.begin_frame(1).expect("initial batch");
    let mut builder = RenderGraphBuilder::new("realtime-ibl-wgpu-product");
    let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
        .expect("realtime graph plan");
    let resources = RealtimeIblGpuResources::new(&device, &request);
    let mut recorder = RealtimeIblWgpuRecorder::new(&device);
    let mut pipeline_cache = IblBakeWgpuPipelineCache::new(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-realtime-ibl-wgpu-product"),
    });
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    let result = recorder
        .record_graph_plan(
            &device,
            &mut encoder,
            true,
            &request,
            &sky,
            &plan,
            &plan.passes,
            &resources,
            &mut pipeline_cache,
        )
        .expect("realtime IBL WGPU recording");
    queue.submit([encoder.finish()]);

    let validation_error = pollster::block_on(error_scope.pop());
    assert!(validation_error.is_none(), "{validation_error:?}");
    assert_eq!(result.report.pass_count, plan.passes.len());
    assert_eq!(result.report.dispatch_count, plan.passes.len());
    assert_eq!(result.report.dispatch_groups.last(), Some(&[2, 2, 2]));
    assert_eq!(
        result.timestamp_readback.is_some(),
        device.features().contains(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
        )
    );
}
