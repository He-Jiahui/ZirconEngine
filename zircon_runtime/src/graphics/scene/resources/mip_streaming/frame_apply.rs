use std::sync::Arc;

use crate::asset::TextureAsset;
use crate::core::framework::render::RenderFrameSubmissionTransaction;
use crate::core::resource::{ResourceHandle, ResourceKind, TextureMarker};
use crate::graphics::backend::RenderBackend;

use super::super::super::GpuTextureResource;
use super::super::ResourceStreamer;
use super::{MipStreamingSettings, MipStreamingTask};

impl ResourceStreamer {
    pub(in crate::graphics::scene::resources::resource_streamer) fn apply_texture_mip_streaming(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        mip_bias: u8,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) {
        let tasks = self.schedule_texture_mip_streaming(
            self.mip_streaming_visibility.clone(),
            MipStreamingSettings {
                mip_bias,
                max_resident_bytes: self.mip_streaming_residency_budget_bytes,
                ..Default::default()
            },
        );
        for task in tasks {
            let rebuilt = self.rebuild_texture_mip_streaming_task(
                backend,
                texture_layout,
                &task,
                submission_transaction,
            );
            let Some(resource) = rebuilt else {
                self.finish_texture_mip_streaming_task(&task, false);
                continue;
            };
            let range = self.finish_texture_mip_streaming_task(&task, true);
            if let (Some(range), Some(prepared)) = (range, self.textures.get_mut(&task.texture)) {
                crate::profile_counter!(
                    "render",
                    "texture_publication_before_upload_completion",
                    1
                );
                prepared.resource = resource;
                prepared.resident_mip_range = range;
            }
        }
    }

    fn rebuild_texture_mip_streaming_task(
        &self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        task: &MipStreamingTask,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Option<Arc<GpuTextureResource>> {
        let prepared = self.textures.get(&task.texture)?;
        if prepared.resident_mip_range != task.resident_mips {
            return None;
        }
        // Common mips come from the previous GPU object. Do not reload a CPU payload here:
        // a different active publication, even at the same revision, cannot certify those mips.
        let payload = {
            crate::profile_scope!("render", "texture", "load_snapshot");
            let resources = self.asset_manager().ok()?.resource_manager();
            let projection = resources.projection_snapshot();
            let catalog = projection.management().row_by_id(task.texture)?;
            if catalog.kind != ResourceKind::Texture
                || catalog.revision != prepared.revision
                || projection.active().row(task.texture)?.key != prepared.active_key
            {
                return None;
            }
            resources
                .snapshot_expected::<TextureMarker, TextureAsset>(
                    ResourceHandle::new(task.texture),
                    &prepared.active_key,
                )
                .ok()?
        };
        crate::profile_counter!("render", "texture_snapshot_load_count", 1);
        if payload.revision() != prepared.revision {
            return None;
        }
        crate::profile_counter!("render", "texture_cpu_payload_clone_bytes", 0);
        let resource = {
            crate::profile_scope!("render", "texture", "rebuild_resident_mips");
            GpuTextureResource::rebuild_resident_mips(
                &backend.device,
                texture_layout,
                task.texture,
                Arc::clone(payload.resource()),
                prepared.resource.as_ref(),
                prepared.resident_mip_range.clone(),
                task.wanted_mips.clone(),
            )
            .ok()?
        };
        self.enqueue_gpu_texture_upload_work_for_frame(
            backend,
            task.texture,
            resource,
            submission_transaction,
        )
        .ok()
    }
}

#[cfg(test)]
#[path = "tests/frame_apply.rs"]
mod tests;
