use std::ops::Range;
use std::sync::Arc;

use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::IblRealtimeBufferSlot;
use crate::graphics::scene::scene_renderer::primitives::SceneEnvironmentSh9;
use crate::graphics::types::GraphicsError;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::SceneRendererCore;

pub(in crate::graphics::scene::scene_renderer::core) struct PendingSceneEnvironmentBindings {
    bind_group: wgpu::BindGroup,
    realtime_ibl_slot: Option<IblRealtimeBufferSlot>,
    sh9_buffer: Option<wgpu::Buffer>,
}

pub(in crate::graphics::scene::scene_renderer::core) struct SceneEnvironmentFrame<'a> {
    core: &'a mut SceneRendererCore,
}

impl SceneEnvironmentFrame<'_> {
    pub(in crate::graphics::scene::scene_renderer::core) fn core(
        &mut self,
    ) -> &mut SceneRendererCore {
        self.core
    }
}

impl Drop for SceneEnvironmentFrame<'_> {
    fn drop(&mut self) {
        self.core.pending_scene_environment_bindings = None;
        self.core.scene_environment_cubemap.discard_pending_upload();
    }
}

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core) fn begin_scene_environment_frame(
        &mut self,
    ) -> SceneEnvironmentFrame<'_> {
        self.pending_scene_environment_bindings = None;
        self.scene_environment_cubemap.begin_frame();
        SceneEnvironmentFrame { core: self }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn frame_scene_bind_group(
        &self,
    ) -> &wgpu::BindGroup {
        self.pending_scene_environment_bindings
            .as_ref()
            .map_or(&self.scene_bind_group, |pending| &pending.bind_group)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn frame_environment_sh9_buffer(
        &self,
    ) -> &wgpu::Buffer {
        self.pending_scene_environment_bindings
            .as_ref()
            .and_then(|pending| pending.sh9_buffer.as_ref())
            .unwrap_or(&self.scene_environment_sh9_buffer)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn prepare_realtime_environment_bindings(
        &mut self,
        bind_group: wgpu::BindGroup,
        slot: IblRealtimeBufferSlot,
    ) {
        self.pending_scene_environment_bindings = Some(PendingSceneEnvironmentBindings {
            bind_group,
            realtime_ibl_slot: Some(slot),
            sh9_buffer: None,
        });
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn encode_environment_sh9_upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        payload: Arc<[u8]>,
        source_range: Range<usize>,
        uploads: &mut WgpuBufferUploadBatch,
    ) -> Result<(), GraphicsError> {
        let invalid_range = || GraphicsError::InvalidBufferUploadRange {
            label: "scene-environment-sh9",
        };
        if payload
            .get(source_range.clone())
            .is_none_or(|bytes| bytes.len() as u64 != SceneEnvironmentSh9::byte_len())
        {
            return Err(invalid_range());
        }
        let destination = self.frame_environment_sh9_buffer().clone();
        let staging = self
            .scene_environment_sh9_staging_buffer
            .get_or_insert_with(|| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("zircon-scene-environment-sh9-staging"),
                    size: SceneEnvironmentSh9::byte_len(),
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            });
        uploads.push(
            WgpuBufferUpload::new(staging.clone(), 0, payload, source_range)
                .ok_or_else(invalid_range)?,
        );
        // Upload admission changes only staging. SH9 and cubemap copies share graphics admission.
        encoder.copy_buffer_to_buffer(staging, 0, &destination, 0, SceneEnvironmentSh9::byte_len());
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn prepare_static_environment_bindings(
        &mut self,
        device: &wgpu::Device,
    ) {
        // Upload admission may precede graphics admission, so a rebind must isolate SH9 too.
        let sh9_buffer = self
            .scene_environment_cubemap
            .has_pending_rebind()
            .then(|| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("zircon-scene-environment-sh9-candidate"),
                    size: SceneEnvironmentSh9::byte_len(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-scene-bind-group"),
            layout: &self.scene_bind_group_layout,
            entries: &self.scene_environment_cubemap.bind_group_entries(
                &self.scene_uniform_buffer,
                &self.scene_environment_brdf_lut,
                sh9_buffer
                    .as_ref()
                    .unwrap_or(&self.scene_environment_sh9_buffer),
            ),
        });
        self.pending_scene_environment_bindings = Some(PendingSceneEnvironmentBindings {
            bind_group,
            realtime_ibl_slot: None,
            sh9_buffer,
        });
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn commit_scene_environment_frame(
        &mut self,
    ) {
        self.scene_environment_cubemap.commit_pending_upload();
        if let Some(pending) = self.pending_scene_environment_bindings.take() {
            self.scene_bind_group = pending.bind_group;
            self.scene_bind_group_realtime_ibl_slot = pending.realtime_ibl_slot;
            if let Some(sh9_buffer) = pending.sh9_buffer {
                self.scene_environment_sh9_buffer = sh9_buffer;
            }
        }
    }
}

#[cfg(test)]
#[path = "environment_frame/tests/cases.rs"]
mod tests;
