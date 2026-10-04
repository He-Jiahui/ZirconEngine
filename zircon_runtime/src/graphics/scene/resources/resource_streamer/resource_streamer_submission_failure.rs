use std::collections::HashSet;

use zr_rhi::SubmissionStatus;

use crate::core::framework::render::{
    RenderFrameSubmissionFailureReceipt, RenderFrameSubmissionProducer,
};
use crate::core::resource::ResourceId;

use super::super::prepared::{
    PreparedMaterial, PreparedMaterialBundle, PreparedMaterialCandidateIdentity,
    PreparedMaterialTextureDependency,
};
use super::ResourceStreamer;

impl ResourceStreamer {
    /// Revokes cache publication whose GPU upload never reached the native queue.
    ///
    /// This is a cold frame-failure path. Stable frames do not scan resource caches.
    pub(crate) fn rollback_failed_frame_submissions(
        &mut self,
        receipt: &RenderFrameSubmissionFailureReceipt,
    ) {
        let invalid_texture_ids = invalid_texture_resource_ids(receipt);
        if invalid_texture_ids.is_empty() {
            return;
        }

        self.textures
            .retain(|texture_id, _| !invalid_texture_ids.contains(texture_id));
        self.post_process_lut_textures
            .retain(|texture_id, _| !invalid_texture_ids.contains(texture_id));
        self.mip_streaming_states
            .retain(|texture_id, _| !invalid_texture_ids.contains(texture_id));
        self.mip_streaming_visibility
            .retain(|visibility| !invalid_texture_ids.contains(&visibility.texture));
        self.last_ui_texture_prepare_receipt = None;

        let mut invalid_material_ids = Vec::new();
        self.materials.retain(|material_id, prepared| {
            let retain = !prepared_material_uses_any_texture(prepared, &invalid_texture_ids);
            if !retain {
                invalid_material_ids.push(*material_id);
            }
            retain
        });
        for material_id in invalid_material_ids {
            self.active_staged_material_ids.remove(&material_id);
        }
    }
}

// 只撤销纹理上传的失败、取消或 device-lost 终态；已 Submitted/Completed 的记录仍有效，依赖撤销由上层遍历各材质代际。
fn invalid_texture_resource_ids(
    receipt: &RenderFrameSubmissionFailureReceipt,
) -> HashSet<ResourceId> {
    receipt
        .pre_scene_submissions()
        .iter()
        .filter(|record| {
            matches!(
                record.producer(),
                RenderFrameSubmissionProducer::TexturePreUpload
                    | RenderFrameSubmissionProducer::TextureCopyUpload
                    | RenderFrameSubmissionProducer::TexturePostUpload
            ) && matches!(
                record.status(),
                SubmissionStatus::Failed
                    | SubmissionStatus::Cancelled
                    | SubmissionStatus::DeviceLost
            )
        })
        .filter_map(|record| record.resource_id())
        .collect()
}

fn prepared_material_uses_any_texture(
    prepared: &PreparedMaterial,
    texture_ids: &HashSet<ResourceId>,
) -> bool {
    [
        prepared.published.as_ref(),
        prepared.previous_published.as_ref(),
        prepared.staged_candidate.as_ref(),
    ]
    .into_iter()
    .flatten()
    .any(|bundle| prepared_material_bundle_uses_any_texture(bundle, texture_ids))
        || prepared
            .rejected_candidate
            .as_ref()
            .and_then(|candidate| candidate.identity.as_ref())
            .is_some_and(|identity| {
                prepared_material_identity_uses_any_texture(identity, texture_ids)
            })
}

fn prepared_material_bundle_uses_any_texture(
    bundle: &PreparedMaterialBundle,
    texture_ids: &HashSet<ResourceId>,
) -> bool {
    texture_dependencies_use_any(&bundle.texture_dependencies, texture_ids)
}

fn prepared_material_identity_uses_any_texture(
    identity: &PreparedMaterialCandidateIdentity,
    texture_ids: &HashSet<ResourceId>,
) -> bool {
    texture_dependencies_use_any(&identity.texture_dependencies, texture_ids)
}

fn texture_dependencies_use_any(
    dependencies: &[PreparedMaterialTextureDependency],
    texture_ids: &HashSet<ResourceId>,
) -> bool {
    dependencies
        .iter()
        .filter_map(|dependency| dependency.id)
        .any(|texture_id| texture_ids.contains(&texture_id))
}

#[cfg(test)]
#[path = "tests/resource_streamer_submission_failure.rs"]
mod tests;
