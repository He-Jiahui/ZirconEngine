use crate::core::resource::ResourceId;

use super::upload::{ReflectionProbeAssetError, ReflectionProbeAssetRejection};

#[derive(Clone, Copy)]
pub(super) struct PendingReflectionProbeUpload {
    pub(super) cubemap: ResourceId,
    pub(super) revision: u64,
    pub(super) slot: u32,
    pub(super) prepare_epoch: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct ReflectionProbeUploadReport {
    pub(in crate::graphics::scene::scene_renderer) extracted_probe_count: usize,
    pub(in crate::graphics::scene::scene_renderer) camera_layer_candidate_count: usize,
    pub(in crate::graphics::scene::scene_renderer) attempted_candidate_count: usize,
    pub(in crate::graphics::scene::scene_renderer) active_probe_count: usize,
    pub(in crate::graphics::scene::scene_renderer) capacity_dropped_candidate_count: usize,
    pub(in crate::graphics::scene::scene_renderer) scheduled_cubemap_upload_count: usize,
    pub(in crate::graphics::scene::scene_renderer) scheduled_cubemap_upload_bytes: u64,
    pub(in crate::graphics::scene::scene_renderer) scheduled_texture_write_count: usize,
    pub(in crate::graphics::scene::scene_renderer) asset_load_call_count: usize,
    pub(in crate::graphics::scene::scene_renderer) asset_load_cpu_time_us: u64,
    pub(in crate::graphics::scene::scene_renderer) rejected_cubemap_count: usize,
    pub(in crate::graphics::scene::scene_renderer) first_rejection:
        Option<ReflectionProbeAssetRejection>,
}

pub(super) fn record_probe_asset_rejection(
    report: &mut ReflectionProbeUploadReport,
    error: ReflectionProbeAssetError,
) {
    report.rejected_cubemap_count += 1;
    if report.first_rejection.is_none() {
        report.first_rejection = Some(error.rejection());
    }
}
