use std::collections::HashMap;
use std::sync::Arc;
use zr_rhi_wgpu::WgpuBufferUpload;

use super::{HzbOcclusionCullParams, HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE};

#[derive(Default)]
pub(super) struct HzbOcclusionParamsWorkspace {
    entries: HashMap<u64, HzbOcclusionParamsEntry>,
    next_buffer_revision: u64,
}

struct HzbOcclusionParamsEntry {
    buffer: Arc<wgpu::Buffer>,
    buffer_revision: u64,
    committed_args_count: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct HzbOcclusionParamsCommit {
    workspace_id: u64,
    buffer_revision: u64,
    args_count: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HzbOcclusionParamsPrepareStats {
    pub(super) created_buffer_count: u32,
    pub(super) uploaded_byte_count: u64,
}

pub(super) struct PreparedHzbOcclusionParams {
    pub(super) buffer: Arc<wgpu::Buffer>,
    pub(super) upload: Option<WgpuBufferUpload>,
    pub(super) commit: Option<HzbOcclusionParamsCommit>,
    pub(super) stats: HzbOcclusionParamsPrepareStats,
}

impl HzbOcclusionParamsWorkspace {
    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        workspace_id: u64,
        args_count: u32,
    ) -> PreparedHzbOcclusionParams {
        let created = !self.entries.contains_key(&workspace_id);
        if created {
            self.next_buffer_revision = self.next_buffer_revision.wrapping_add(1).max(1);
            self.entries.insert(
                workspace_id,
                HzbOcclusionParamsEntry {
                    buffer: Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("zircon-hzb-occlusion-cull-params"),
                        size: HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    })),
                    buffer_revision: self.next_buffer_revision,
                    committed_args_count: None,
                },
            );
        }
        let entry = self
            .entries
            .get(&workspace_id)
            .expect("HZB params workspace entry must exist after materialization");
        let needs_upload = entry.committed_args_count != Some(args_count);
        let (upload, commit, uploaded_byte_count) = if needs_upload {
            (
                Some(WgpuBufferUpload::from_bytes(
                    entry.buffer.as_ref().clone(),
                    0,
                    bytemuck::bytes_of(&HzbOcclusionCullParams::new(args_count)),
                )),
                Some(HzbOcclusionParamsCommit {
                    workspace_id,
                    buffer_revision: entry.buffer_revision,
                    args_count,
                }),
                HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE,
            )
        } else {
            (None, None, 0)
        };
        PreparedHzbOcclusionParams {
            buffer: Arc::clone(&entry.buffer),
            upload,
            commit,
            stats: HzbOcclusionParamsPrepareStats {
                created_buffer_count: u32::from(created),
                uploaded_byte_count,
            },
        }
    }

    pub(super) fn commit(&mut self, commit: HzbOcclusionParamsCommit) -> bool {
        let Some(entry) = self.entries.get_mut(&commit.workspace_id) else {
            return false;
        };
        if entry.buffer_revision != commit.buffer_revision {
            return false;
        }
        entry.committed_args_count = Some(commit.args_count);
        true
    }
}

#[cfg(test)]
#[path = "tests/params_workspace.rs"]
mod tests;

#[cfg(test)]
#[path = "params_workspace/tests/hash_index_tests.rs"]
mod hash_index_tests;
