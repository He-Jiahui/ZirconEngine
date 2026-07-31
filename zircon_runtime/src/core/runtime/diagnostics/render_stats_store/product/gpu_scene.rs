use crate::core::framework::render::{RenderGpuSceneUploadPath, RenderStats};

use super::{DiagnosticStore, record_bool, record_bytes, record_count};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.gpu_scene.primitive_count",
        frame_index,
        stats.last_gpu_scene_primitive_count as usize,
        &["render", "gpu_scene"],
    );
    record_count(
        store,
        "render.gpu_scene.instance_count",
        frame_index,
        stats.last_gpu_scene_instance_count as usize,
        &["render", "gpu_scene"],
    );
    record_count(
        store,
        "render.gpu_scene.dirty_entry_count",
        frame_index,
        stats.last_gpu_scene_dirty_entry_count,
        &["render", "gpu_scene", "upload"],
    );
    record_bytes(
        store,
        "render.gpu_scene.uploaded_bytes",
        frame_index,
        stats.last_gpu_scene_uploaded_bytes,
        &["render", "gpu_scene", "upload"],
    );
    record_bool(
        store,
        "render.gpu_scene.upload_path.direct_queue_write",
        frame_index,
        stats.last_gpu_scene_upload_path == RenderGpuSceneUploadPath::DirectQueueWrite,
        &["render", "gpu_scene", "upload", "direct_queue_write"],
    );
    record_count(
        store,
        "render.gpu_scene.free_span_count",
        frame_index,
        stats.last_gpu_scene_free_span_count,
        &["render", "gpu_scene", "allocator"],
    );
    record_count(
        store,
        "render.gpu_scene.primitive_upload_range_count",
        frame_index,
        stats.last_gpu_scene_primitive_upload_range_count,
        &["render", "gpu_scene", "upload", "primitive"],
    );
    record_count(
        store,
        "render.gpu_scene.instance_upload_range_count",
        frame_index,
        stats.last_gpu_scene_instance_upload_range_count,
        &["render", "gpu_scene", "upload", "instance"],
    );
}
