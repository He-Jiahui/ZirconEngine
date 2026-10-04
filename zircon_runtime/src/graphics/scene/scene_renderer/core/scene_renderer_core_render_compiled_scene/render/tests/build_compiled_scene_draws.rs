use super::*;

#[test]
fn compiled_scene_draws_report_virtual_geometry_indirect_counts_without_buffers() {
    let mut draws = CompiledSceneDraws {
        draws: Vec::new(),
        prepared_mesh_queue_stats: PreparedMeshQueueStats::default(),
        prebuilt_mesh_pass_command_buffers: MeshPassCommandBuffers::default(),
        gpu_scene_prepared_upload: None,
        gpu_scene_upload_report: GpuSceneUploadReport::default(),
        indirect_segment_count: 2,
        indirect_args_count: 3,
        indirect_args_buffer: None,
        indirect_submission_buffer: None,
        indirect_authority_buffer: None,
        indirect_draw_ref_buffer: None,
        indirect_segment_buffer: None,
        pending_command_cache_plan_stats: PendingMeshCommandCachePlanStats::default(),
        pending_command_cache_extraction_stats: Default::default(),
        material_pipeline_requirements: Default::default(),
    };

    let stats = draws.virtual_geometry_indirect_stats();

    assert_eq!(stats.draw_count, 3);
    assert_eq!(stats.args_count, 3);
    assert_eq!(stats.segment_count, 2);
    assert_eq!(stats.buffer_count, 0);
    assert!(matches!(
        draws.take_gpu_scene_prepared_upload(),
        Err(GraphicsError::MissingPreparedGpuSceneUpload)
    ));
}
