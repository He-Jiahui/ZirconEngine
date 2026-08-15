use std::collections::BTreeMap;
use std::sync::Arc;

use super::{HzbOcclusionCullParams, HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE};

#[derive(Default)]
pub(super) struct HzbOcclusionParamsWorkspace {
    entries: BTreeMap<u64, HzbOcclusionParamsEntry>,
}

struct HzbOcclusionParamsEntry {
    buffer: Arc<wgpu::Buffer>,
    args_count: u32,
    initialized: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HzbOcclusionParamsPrepareStats {
    pub(super) created_buffer_count: u32,
    pub(super) uploaded_byte_count: u64,
}

pub(super) struct PreparedHzbOcclusionParams {
    pub(super) buffer: Arc<wgpu::Buffer>,
    pub(super) stats: HzbOcclusionParamsPrepareStats,
}

impl HzbOcclusionParamsWorkspace {
    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        workspace_id: u64,
        args_count: u32,
    ) -> PreparedHzbOcclusionParams {
        let entry = self
            .entries
            .entry(workspace_id)
            .or_insert_with(|| HzbOcclusionParamsEntry {
                buffer: Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("zircon-hzb-occlusion-cull-params"),
                    size: HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })),
                args_count: 0,
                initialized: false,
            });
        let created_buffer_count = u32::from(!entry.initialized);
        let uploaded_byte_count = if !entry.initialized || entry.args_count != args_count {
            queue.write_buffer(
                &entry.buffer,
                0,
                bytemuck::bytes_of(&HzbOcclusionCullParams::new(args_count)),
            );
            entry.args_count = args_count;
            entry.initialized = true;
            HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE
        } else {
            0
        };
        PreparedHzbOcclusionParams {
            buffer: Arc::clone(&entry.buffer),
            stats: HzbOcclusionParamsPrepareStats {
                created_buffer_count,
                uploaded_byte_count,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_hzb_params_workspace_reuses_buffer_and_skips_upload() {
        let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
            return;
        };
        let mut workspace = HzbOcclusionParamsWorkspace::default();

        let first = workspace.prepare(&backend.device, &backend.queue, 7, 64);
        let second = workspace.prepare(&backend.device, &backend.queue, 7, 64);
        let changed = workspace.prepare(&backend.device, &backend.queue, 7, 65);

        assert_eq!(first.stats.created_buffer_count, 1);
        assert_eq!(
            first.stats.uploaded_byte_count,
            HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE
        );
        assert_eq!(second.stats, HzbOcclusionParamsPrepareStats::default());
        assert_eq!(changed.stats.created_buffer_count, 0);
        assert_eq!(
            changed.stats.uploaded_byte_count,
            HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE
        );
    }
}
