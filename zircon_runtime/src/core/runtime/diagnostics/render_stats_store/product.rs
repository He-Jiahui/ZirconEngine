mod camera;
mod effect_stack;
mod gpu_scene;
mod hzb;
mod light;
mod light_grid;
mod material;
mod mesh_queue;
mod sprite;
mod ui;
mod visibility;

use crate::core::framework::render::RenderStats;

use super::{record_bool, record_bytes, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    camera::record(store, stats);
    visibility::record(store, stats);
    hzb::record(store, stats);
    light_grid::record(store, stats);
    material::record(store, stats);
    light::record(store, stats);
    mesh_queue::record(store, stats);
    gpu_scene::record(store, stats);
    sprite::record(store, stats);
    effect_stack::record(store, stats);
    ui::record(store, stats);
}

#[cfg(test)]
mod tests {
    use crate::core::diagnostics::DiagnosticStore;
    use crate::core::framework::render::{
        RenderCameraTargetGraphImportReport, RenderCameraTargetKind,
        RenderCameraTargetWritebackReport, RenderCaptureReport, RenderCaptureSource,
        RenderGpuSceneUploadPath, RenderStats,
    };
    use crate::core::math::UVec2;

    use super::record;

    #[test]
    fn render_product_diagnostics_record_texture_conversion_writeback_marker() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_camera_target_writeback: RenderCameraTargetWritebackReport::converted(UVec2::new(
                72, 40,
            )),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.camera.target.writeback.converted",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.converted_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.conversion_debug_marker_emitted",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.debug_marker_emitted",
            0.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.width",
            72.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.height",
            40.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_texture_direct_graph_import_readiness() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_camera_target_graph_import:
                RenderCameraTargetGraphImportReport::ready_for_direct_import(UVec2::new(96, 54)),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.camera.target.graph_import.ready_for_direct_import",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.direct_imported",
            0.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.requires_conversion_writeback",
            0.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.direct_import_count",
            0.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.conversion_writeback_count",
            0.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.width",
            96.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.height",
            54.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_texture_direct_graph_import_execution() {
        let mut store = DiagnosticStore::default();
        let stats =
            RenderStats {
                submitted_frames: 12,
                last_camera_target_graph_import:
                    RenderCameraTargetGraphImportReport::direct_imported(UVec2::new(96, 54)),
                last_camera_target_writeback:
                    RenderCameraTargetWritebackReport::skipped_direct_import(UVec2::new(96, 54)),
                ..RenderStats::default()
            };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.camera.target.graph_import.ready_for_direct_import",
            0.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.direct_imported",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.direct_import_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.skipped_direct_import",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.copy_count",
            0.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_capture_source_report() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_capture_report: RenderCaptureReport::new(
                RenderCameraTargetKind::Texture,
                RenderCaptureSource::TextureWritebackConversion,
                UVec2::new(72, 40),
                crate::core::framework::render::RenderCameraTargetGraphImportStatus::RequiresConversionWriteback,
                crate::core::framework::render::RenderCameraTargetWritebackStatus::Converted,
            ),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.capture.source.texture_writeback_conversion",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.capture.source.texture_direct_graph_import",
            0.0,
            "bool",
        );
        assert_series(&store, "render.capture.width", 72.0, "count");
        assert_series(&store, "render.capture.height", 40.0, "count");
    }

    #[test]
    fn render_product_diagnostics_record_visibility_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_visibility_view_count: 2,
            last_visibility_input_count: 8,
            last_visibility_layer_filtered_count: 1,
            last_visibility_frustum_culled_count: 3,
            last_visibility_occlusion_culled_count: 1,
            last_visibility_visible_count: 3,
            last_visibility_static_index_full_rebuild_count: 0,
            last_visibility_static_index_incremental_update_count: 1,
            last_visibility_static_index_inserted_count: 2,
            last_visibility_static_index_updated_count: 3,
            last_visibility_static_index_removed_count: 4,
            last_visibility_static_index_indexed_entity_count: 10,
            last_visibility_static_index_occupied_cell_count: 7,
            last_visibility_static_index_main_view_prefilter_used: true,
            last_visibility_static_index_main_view_static_input_count: 12,
            last_visibility_static_index_main_view_static_candidate_count: 5,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.visibility.view_count", 2.0, "count");
        assert_series(&store, "render.visibility.input_count", 8.0, "count");
        assert_series(
            &store,
            "render.visibility.layer_filtered_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.frustum_culled_count",
            3.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.occlusion_culled_count",
            1.0,
            "count",
        );
        assert_series(&store, "render.visibility.visible_count", 3.0, "count");
        assert_series(
            &store,
            "render.visibility.static_index.full_rebuild_count",
            0.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.incremental_update_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.inserted_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.updated_count",
            3.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.removed_count",
            4.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.indexed_entity_count",
            10.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.occupied_cell_count",
            7.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.main_view_prefilter_used",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.visibility.static_index.main_view_static_input_count",
            12.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.main_view_static_candidate_count",
            5.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_hzb_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_hzb_mip_count: 10,
            last_hzb_graph_executed_pass_count: 1,
            last_hzb_occlusion_reported: true,
            last_hzb_occlusion_candidate_arg_count: 6,
            last_hzb_occlusion_candidate_instance_count: 42,
            last_hzb_occlusion_dispatch_group_count: 2,
            last_hzb_occlusion_dispatched_phase_count: 1,
            last_hzb_occlusion_history_available: true,
            last_hzb_occlusion_readback_available: true,
            last_hzb_occlusion_tested_arg_count: 6,
            last_hzb_occlusion_tested_instance_count: 42,
            last_hzb_occlusion_culled_arg_count: 2,
            last_hzb_occlusion_culled_instance_count: 18,
            last_hzb_occlusion_indirect_args_readback_available: true,
            last_hzb_occlusion_readback_arg_count: 6,
            last_hzb_occlusion_compacted_draw_count: 4,
            last_hzb_occlusion_zero_instance_arg_count: 2,
            last_hzb_occlusion_remaining_instance_count: 24,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.hzb.mip_count", 10.0, "count");
        assert_series(&store, "render.hzb.graph_executed_pass_count", 1.0, "count");
        assert_series(&store, "render.hzb.occlusion.reported", 1.0, "bool");
        assert_series(
            &store,
            "render.hzb.occlusion.candidate_arg_count",
            6.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.candidate_instance_count",
            42.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.dispatch_group_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.dispatched_phase_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.history_available",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.readback_available",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.tested_arg_count",
            6.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.tested_instance_count",
            42.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.culled_arg_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.culled_instance_count",
            18.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.indirect_args_readback_available",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.readback_arg_count",
            6.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.compacted_draw_count",
            4.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.zero_instance_arg_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.remaining_instance_count",
            24.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_light_grid_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_light_grid_reported: true,
            last_light_grid_light_count: 9,
            last_light_grid_tile_count: 64,
            last_light_grid_zbin_count: 32,
            last_light_grid_non_empty_tile_count: 11,
            last_light_grid_non_empty_zbin_count: 7,
            last_light_grid_non_empty_cluster_count: 23,
            last_light_grid_peak_lights_per_cluster: 5,
            last_light_grid_average_lights_per_cluster_milli: 375,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.light_grid.reported", 1.0, "bool");
        assert_series(&store, "render.light_grid.light_count", 9.0, "count");
        assert_series(&store, "render.light_grid.tile_count", 64.0, "count");
        assert_series(&store, "render.light_grid.zbin_count", 32.0, "count");
        assert_series(
            &store,
            "render.light_grid.non_empty_tile_count",
            11.0,
            "count",
        );
        assert_series(
            &store,
            "render.light_grid.non_empty_zbin_count",
            7.0,
            "count",
        );
        assert_series(
            &store,
            "render.light_grid.non_empty_cluster_count",
            23.0,
            "count",
        );
        assert_series(
            &store,
            "render.light_grid.peak_lights_per_cluster",
            5.0,
            "count",
        );
        assert_series(
            &store,
            "render.light_grid.average_lights_per_cluster",
            0.375,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_skinned_mesh_queue_count() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_mesh_skinned_draw_count: 3,
            last_mesh_skinned_palette_upload_count: 2,
            last_mesh_skinned_previous_palette_upload_count: 1,
            last_mesh_skinned_gpu_source_candidate_count: 1,
            last_mesh_skinned_gpu_cpu_morphed_source_candidate_count: 1,
            last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count: 1,
            last_mesh_skinned_gpu_skinning_draw_count: 1,
            last_mesh_skinned_gpu_velocity_draw_count: 1,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.mesh.queue.skinned_draw_count", 3.0, "count");
        assert_series(
            &store,
            "render.mesh.queue.skinned_palette_upload_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_previous_palette_upload_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_source_candidate_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_cpu_morphed_source_candidate_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_skinning_draw_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_velocity_draw_count",
            1.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_gpu_scene_upload_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_gpu_scene_primitive_count: 5,
            last_gpu_scene_instance_count: 7,
            last_gpu_scene_dirty_entry_count: 3,
            last_gpu_scene_uploaded_bytes: 128,
            last_gpu_scene_upload_path: RenderGpuSceneUploadPath::DirectQueueWrite,
            last_gpu_scene_free_span_count: 2,
            last_gpu_scene_primitive_upload_range_count: 1,
            last_gpu_scene_instance_upload_range_count: 4,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.gpu_scene.primitive_count", 5.0, "count");
        assert_series(&store, "render.gpu_scene.instance_count", 7.0, "count");
        assert_series(&store, "render.gpu_scene.dirty_entry_count", 3.0, "count");
        assert_series(&store, "render.gpu_scene.uploaded_bytes", 128.0, "bytes");
        assert_series(
            &store,
            "render.gpu_scene.upload_path.direct_queue_write",
            1.0,
            "bool",
        );
        assert_series(&store, "render.gpu_scene.free_span_count", 2.0, "count");
        assert_series(
            &store,
            "render.gpu_scene.primitive_upload_range_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.gpu_scene.instance_upload_range_count",
            4.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_mesh_indirect_batch_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_indirect_batch_count: 2,
            last_indirect_batched_draw_count: 5,
            last_indirect_fallback_draw_count: 4,
            last_indirect_args_count: 5,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.mesh.queue.indirect_batch_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.indirect_batched_draw_count",
            5.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.indirect_fallback_draw_count",
            4.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.indirect_args_count",
            5.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_mesh_lod_queue_count() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_mesh_lod_draw_count: 4,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.mesh.queue.lod_draw_count", 4.0, "count");
    }

    #[test]
    fn render_product_diagnostics_record_taa_reactive_mask_queue_count() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_mesh_taa_reactive_mask_command_count: 3,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.mesh.queue.taa_reactive_mask_command_count",
            3.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_mesh_command_cache_counts() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_mesh_command_count: 9,
            last_mesh_cached_command_hit_count: 4,
            last_mesh_command_rebuild_count: 5,
            last_mesh_dynamic_command_count: 2,
            last_mesh_command_cache_miss_count: 1,
            last_mesh_command_cache_invalidated_transform_count: 0,
            last_mesh_command_cache_invalidated_geometry_count: 1,
            last_mesh_command_cache_invalidated_material_count: 2,
            last_mesh_replay_state_change_count: 6,
            last_mesh_replay_bind_skip_count: 7,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.mesh.queue.command_count", 9.0, "count");
        assert_series(
            &store,
            "render.mesh.queue.cached_command_hit_count",
            4.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.command_rebuild_count",
            5.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.dynamic_command_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.command_cache_miss_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.command_cache_invalidated_transform_count",
            0.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.command_cache_invalidated_geometry_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.command_cache_invalidated_material_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.replay_state_change_count",
            6.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.replay_bind_skip_count",
            7.0,
            "count",
        );
    }

    fn assert_series(store: &DiagnosticStore, path: &str, value: f64, unit: &str) {
        let snapshot = store.snapshot();
        let series = snapshot
            .series
            .iter()
            .find(|series| series.path.as_str() == path)
            .unwrap_or_else(|| panic!("missing diagnostic series `{path}`"));
        assert_eq!(series.current, Some(value));
        assert_eq!(series.unit.as_deref(), Some(unit));
        assert_eq!(series.history.len(), 1);
        assert_eq!(series.history[0].frame_index, 12);
        assert_eq!(series.history[0].value, value);
    }
}
