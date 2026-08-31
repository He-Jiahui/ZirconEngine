use crate::core::framework::render::RenderStats;

use super::super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
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
