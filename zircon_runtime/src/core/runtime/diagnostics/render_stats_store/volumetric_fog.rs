use crate::core::framework::render::RenderStats;

use super::{record_bytes, record_count, DiagnosticStore};

/// 将体积雾计算调度规模与上传量放在同一提交帧下，关联 GPU 工作量和 CPU 供给。
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.volumetric_fog.compute_dispatch_count",
        frame_index,
        stats.last_volumetric_fog_compute_dispatch_count,
        &["render", "volumetric_fog", "compute"],
    );
    record_count(
        store,
        "render.volumetric_fog.compute_dispatch_group_count",
        frame_index,
        stats.last_volumetric_fog_compute_dispatch_group_count,
        &["render", "volumetric_fog", "compute", "workgroup"],
    );
    record_bytes(
        store,
        "render.volumetric_fog.uploaded_bytes",
        frame_index,
        stats.last_volumetric_fog_uploaded_bytes,
        &["render", "volumetric_fog", "upload"],
    );
}

#[cfg(test)]
#[path = "tests/volumetric_fog.rs"]
mod tests;
