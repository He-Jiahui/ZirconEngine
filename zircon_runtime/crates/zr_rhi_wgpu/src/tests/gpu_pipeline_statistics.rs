use super::{
    gpu_pipeline_statistics_supported, insert_completed_frame_in_order,
    GpuPipelineStatisticsFrameResult, GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES,
};
use std::collections::VecDeque;

#[test]
fn pipeline_statistics_capability_requires_the_negotiated_wgpu_feature() {
    assert!(gpu_pipeline_statistics_supported(
        GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES
    ));
    assert!(!gpu_pipeline_statistics_supported(wgpu::Features::empty()));
}

#[test]
fn completed_statistics_frames_are_drained_in_renderer_generation_order() {
    let mut completed = VecDeque::new();
    for frame_generation in [9, 7, 8] {
        insert_completed_frame_in_order(
            &mut completed,
            GpuPipelineStatisticsFrameResult {
                frame_generation,
                pass_statistics: Vec::new(),
            },
        );
    }

    assert_eq!(completed.pop_front().unwrap().frame_generation, 7);
    assert_eq!(completed.pop_front().unwrap().frame_generation, 8);
    assert_eq!(completed.pop_front().unwrap().frame_generation, 9);
}

#[test]
fn pipeline_statistics_collector_only_drains_results_after_the_readback_owner_polls() {
    let source = include_str!("../gpu_pipeline_statistics.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(source.contains("pub fn accept_product_query_delivery"));
    assert!(source.contains("pub fn try_collect(&mut self)"));
    assert!(!source.contains("GpuReadbackQueue"));
    assert!(!source.contains("resolve_query_set"));
    assert!(!source.contains("map_async"));
}
