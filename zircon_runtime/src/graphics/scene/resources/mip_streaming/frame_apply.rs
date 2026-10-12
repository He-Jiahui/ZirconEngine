use std::sync::Arc;

use crate::core::framework::render::RenderFrameSubmissionTransaction;
use crate::graphics::backend::RenderBackend;

use super::super::super::prepared::PreparedTexture;
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
            match rebuilt {
                Some((revision, resource, capture_sample_rgba)) => {
                    if let Some(resident_mip_range) =
                        self.finish_texture_mip_streaming_task(&task, true)
                    {
                        crate::profile_counter!(
                            "render",
                            "texture_publication_before_upload_completion",
                            1
                        );
                        self.textures.insert(
                            task.texture,
                            PreparedTexture {
                                revision,
                                resource,
                                capture_sample_rgba,
                                resident_mip_range,
                            },
                        );
                    }
                }
                None => {
                    self.finish_texture_mip_streaming_task(&task, false);
                }
            }
        }
    }

    fn rebuild_texture_mip_streaming_task(
        &self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        task: &MipStreamingTask,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Option<(u64, Arc<GpuTextureResource>, Option<[f32; 4]>)> {
        let prepared = self.textures.get(&task.texture)?;
        if prepared.resident_mip_range != task.resident_mips {
            return None;
        }
        let revision = prepared.revision;
        let capture_sample_rgba = prepared.capture_sample_rgba;
        let previous = Arc::clone(&prepared.resource);
        let previous_range = prepared.resident_mip_range.clone();
        let payload = {
            crate::profile_scope!("render", "texture", "load_snapshot");
            self.asset_manager()
                .ok()?
                .load_texture_asset_snapshot(task.texture)
                .ok()?
        };
        crate::profile_counter!("render", "texture_snapshot_load_count", 1);
        if payload.revision() != revision {
            return None;
        }
        let payload_bytes = match &payload.payload {
            crate::asset::TexturePayload::Rgba8 => payload.rgba.len(),
            crate::asset::TexturePayload::Container { bytes, .. } => bytes.len(),
        };
        crate::profile_counter!("render", "texture_cpu_payload_clone_bytes", payload_bytes);
        let resource = {
            crate::profile_scope!("render", "texture", "rebuild_resident_mips");
            GpuTextureResource::rebuild_resident_mips(
                &backend.device,
                texture_layout,
                task.texture,
                (*payload).clone(),
                previous.as_ref(),
                previous_range,
                task.wanted_mips.clone(),
            )
            .ok()?
        };
        let resource = self
            .enqueue_gpu_texture_upload_work_for_frame(
                backend,
                task.texture,
                resource,
                submission_transaction,
            )
            .ok()?;
        Some((revision, resource, capture_sample_rgba))
    }
}

#[cfg(test)]
#[path = "tests/frame_apply.rs"]
mod tests;
