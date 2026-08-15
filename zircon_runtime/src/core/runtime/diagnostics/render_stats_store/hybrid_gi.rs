use crate::core::framework::render::{
    RenderHybridGiPayloadSource, RenderHybridGiRadianceCacheGpuStage, RenderStats,
};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_probe_and_cache(store, stats);
    record_scene_and_surface_cache(store, stats);
    record_voxel_cache(store, stats);
    record_global_sdf(store, stats);
    record_payload_source(store, stats);
}

fn record_probe_and_cache(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hybrid_gi.active_probe_count",
        frame_index,
        stats.last_hybrid_gi_active_probe_count,
        &["render", "hybrid_gi", "probe"],
    );
    record_count(
        store,
        "render.hybrid_gi.requested_probe_count",
        frame_index,
        stats.last_hybrid_gi_requested_probe_count,
        &["render", "hybrid_gi", "probe", "request"],
    );
    record_count(
        store,
        "render.hybrid_gi.dirty_probe_count",
        frame_index,
        stats.last_hybrid_gi_dirty_probe_count,
        &["render", "hybrid_gi", "probe", "dirty"],
    );
    record_count(
        store,
        "render.hybrid_gi.cache_entry_count",
        frame_index,
        stats.last_hybrid_gi_cache_entry_count,
        &["render", "hybrid_gi", "cache"],
    );
    record_count(
        store,
        "render.hybrid_gi.resident_probe_count",
        frame_index,
        stats.last_hybrid_gi_resident_probe_count,
        &["render", "hybrid_gi", "probe", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.pending_update_count",
        frame_index,
        stats.last_hybrid_gi_pending_update_count,
        &["render", "hybrid_gi", "update", "pending"],
    );
    record_count(
        store,
        "render.hybrid_gi.scheduled_trace_region_count",
        frame_index,
        stats.last_hybrid_gi_scheduled_trace_region_count,
        &["render", "hybrid_gi", "trace"],
    );
}

fn record_scene_and_surface_cache(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hybrid_gi.scene.card_count",
        frame_index,
        stats.last_hybrid_gi_scene_card_count,
        &["render", "hybrid_gi", "scene"],
    );
    record_count(
        store,
        "render.hybrid_gi.scene.screen_probe_count",
        frame_index,
        stats.last_hybrid_gi_scene_screen_probe_count,
        &["render", "hybrid_gi", "scene", "screen_probe"],
    );
    record_count(
        store,
        "render.hybrid_gi.scene.radiance_cache_entry_count",
        frame_index,
        stats.last_hybrid_gi_scene_radiance_cache_entry_count,
        &["render", "hybrid_gi", "scene", "radiance_cache"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.resident_probe_count",
        frame_index,
        stats.last_hybrid_gi_radiance_cache_resident_probe_count,
        &["render", "hybrid_gi", "radiance_cache", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.update_probe_count",
        frame_index,
        stats.last_hybrid_gi_radiance_cache_update_probe_count,
        &["render", "hybrid_gi", "radiance_cache", "update"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.truncated_demand_count",
        frame_index,
        stats.last_hybrid_gi_radiance_cache_truncated_demand_count,
        &["render", "hybrid_gi", "radiance_cache", "budget"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.generation",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_radiance_cache_generation).unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "radiance_cache", "generation"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.scroll_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_radiance_cache_scroll_count).unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "radiance_cache", "scroll"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.history_clear_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_radiance_cache_history_clear_count)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "radiance_cache", "history"],
    );
    for (stage, dispatch_count) in RenderHybridGiRadianceCacheGpuStage::ALL.into_iter().zip(
        stats
            .last_hybrid_gi_radiance_cache_gpu_stage_dispatch_counts
            .iter()
            .copied(),
    ) {
        record_count(
            store,
            match stage {
                RenderHybridGiRadianceCacheGpuStage::Mark => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.mark"
                }
                RenderHybridGiRadianceCacheGpuStage::Allocate => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.allocate"
                }
                RenderHybridGiRadianceCacheGpuStage::Trace => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.trace"
                }
                RenderHybridGiRadianceCacheGpuStage::Filter => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.filter"
                }
                RenderHybridGiRadianceCacheGpuStage::BorderMip => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.border_mip"
                }
                RenderHybridGiRadianceCacheGpuStage::Consume => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.consume"
                }
            },
            frame_index,
            usize::try_from(dispatch_count).unwrap_or(usize::MAX),
            &["render", "hybrid_gi", "radiance_cache", "gpu_dispatch"],
        );
    }
    record_count(
        store,
        "render.hybrid_gi.surface_cache.resident_page_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_resident_page_count,
        &["render", "hybrid_gi", "surface_cache", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.dirty_page_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_dirty_page_count,
        &["render", "hybrid_gi", "surface_cache", "dirty"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.feedback_card_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_feedback_card_count,
        &["render", "hybrid_gi", "surface_cache", "feedback"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.capture_slot_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_capture_slot_count,
        &["render", "hybrid_gi", "surface_cache", "capture"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.invalidated_page_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_invalidated_page_count,
        &["render", "hybrid_gi", "surface_cache", "invalidation"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.depth_sample_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_depth_sample_count,
        &["render", "hybrid_gi", "surface_cache", "depth"],
    );
    record_count(
        store,
        "render.hybrid_gi.probe_trace.tile_count",
        frame_index,
        stats.last_hybrid_gi_probe_trace_tile_count,
        &["render", "hybrid_gi", "probe_trace", "tile"],
    );
    for (axis, group_count) in stats
        .last_hybrid_gi_probe_trace_dispatch_group_count
        .iter()
        .copied()
        .enumerate()
    {
        record_count(
            store,
            match axis {
                0 => "render.hybrid_gi.probe_trace.dispatch_group_count.x",
                1 => "render.hybrid_gi.probe_trace.dispatch_group_count.y",
                _ => "render.hybrid_gi.probe_trace.dispatch_group_count.z",
            },
            frame_index,
            group_count,
            &["render", "hybrid_gi", "probe_trace", "dispatch"],
        );
    }
}

fn record_voxel_cache(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hybrid_gi.voxel.resident_clipmap_count",
        frame_index,
        stats.last_hybrid_gi_voxel_resident_clipmap_count,
        &["render", "hybrid_gi", "voxel", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.voxel.dirty_clipmap_count",
        frame_index,
        stats.last_hybrid_gi_voxel_dirty_clipmap_count,
        &["render", "hybrid_gi", "voxel", "dirty"],
    );
    record_count(
        store,
        "render.hybrid_gi.voxel.invalidated_clipmap_count",
        frame_index,
        stats.last_hybrid_gi_voxel_invalidated_clipmap_count,
        &["render", "hybrid_gi", "voxel", "invalidation"],
    );
}

fn record_global_sdf(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hybrid_gi.global_sdf.cpu_prepare_time_us",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_cpu_prepare_time_us).unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "cpu", "prepare"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.cpu_mesh_object_collection_time_us",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_cpu_mesh_object_collection_time_us)
            .unwrap_or(usize::MAX),
        &[
            "render",
            "hybrid_gi",
            "global_sdf",
            "cpu",
            "object",
            "collection",
        ],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.cpu_mesh_scene_sync_time_us",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_cpu_mesh_scene_sync_time_us)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "cpu", "mesh", "sync"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.cpu_residency_time_us",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_cpu_residency_time_us)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "cpu", "residency"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.cpu_influence_update_time_us",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_cpu_influence_update_time_us)
            .unwrap_or(usize::MAX),
        &[
            "render",
            "hybrid_gi",
            "global_sdf",
            "cpu",
            "influence",
            "update",
        ],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.cpu_candidate_build_time_us",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_cpu_candidate_build_time_us)
            .unwrap_or(usize::MAX),
        &[
            "render",
            "hybrid_gi",
            "global_sdf",
            "cpu",
            "candidate",
            "build",
        ],
    );
    record_bool(
        store,
        "render.hybrid_gi.global_sdf.mesh_projection.cache_hit",
        frame_index,
        stats.last_hybrid_gi_global_sdf_mesh_projection_cache_hit,
        &[
            "render",
            "hybrid_gi",
            "global_sdf",
            "mesh_projection",
            "cache",
        ],
    );
    record_bool(
        store,
        "render.hybrid_gi.global_sdf.mesh_projection.rebuilt",
        frame_index,
        !stats.last_hybrid_gi_global_sdf_mesh_projection_cache_hit,
        &[
            "render",
            "hybrid_gi",
            "global_sdf",
            "mesh_projection",
            "cache",
        ],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.object_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_object_count,
        &["render", "hybrid_gi", "global_sdf", "object"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.resident_page_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_resident_page_count,
        &["render", "hybrid_gi", "global_sdf", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.sampleable_page_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_sampleable_page_count,
        &["render", "hybrid_gi", "global_sdf", "sampleable"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.dirty_page_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_dirty_page_count,
        &["render", "hybrid_gi", "global_sdf", "dirty"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.dispatched_page_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_dispatched_page_count,
        &["render", "hybrid_gi", "global_sdf", "dispatch"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.uploaded_page_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_uploaded_page_count,
        &["render", "hybrid_gi", "global_sdf", "upload"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.deferred_page_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_deferred_page_count,
        &["render", "hybrid_gi", "global_sdf", "budget"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.candidate_overflow_page_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_candidate_overflow_page_count,
        &["render", "hybrid_gi", "global_sdf", "candidate", "overflow"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.candidate_contributor_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_candidate_contributor_count,
        &[
            "render",
            "hybrid_gi",
            "global_sdf",
            "candidate",
            "contributor",
        ],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.clipmap_fallback_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_clipmap_fallback_count,
        &["render", "hybrid_gi", "global_sdf", "clipmap", "fallback"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.candidate_bucket_capacity_bytes",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_candidate_bucket_capacity_bytes)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "candidate", "capacity"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.persistent_resource_byte_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_persistent_resource_byte_count)
            .unwrap_or(usize::MAX),
        &[
            "render",
            "hybrid_gi",
            "global_sdf",
            "resource",
            "persistent",
        ],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.transient_buffer_creation_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_transient_buffer_creation_count,
        &["render", "hybrid_gi", "global_sdf", "resource", "transient"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.transient_bind_group_creation_count",
        frame_index,
        stats.last_hybrid_gi_global_sdf_transient_bind_group_creation_count,
        &["render", "hybrid_gi", "global_sdf", "resource", "transient"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.transient_parameter_upload_byte_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_transient_parameter_upload_byte_count)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "resource", "upload"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.transient_page_upload_byte_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_transient_page_upload_byte_count)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "resource", "upload"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.transient_mesh_upload_byte_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_transient_mesh_upload_byte_count)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "resource", "upload"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.transient_completion_upload_byte_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_transient_completion_upload_byte_count)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "resource", "upload"],
    );
    record_count(
        store,
        "render.hybrid_gi.global_sdf.transient_upload_byte_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_global_sdf_transient_upload_byte_count)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "global_sdf", "resource", "upload"],
    );
}

fn record_payload_source(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let source = stats.last_hybrid_gi_payload_source;
    record_bool(
        store,
        "render.hybrid_gi.payload.source.none",
        frame_index,
        source == RenderHybridGiPayloadSource::None,
        &["render", "hybrid_gi", "payload", "source"],
    );
    record_bool(
        store,
        "render.hybrid_gi.payload.source.scene_representation",
        frame_index,
        source == RenderHybridGiPayloadSource::SceneRepresentation,
        &["render", "hybrid_gi", "payload", "source"],
    );
}
