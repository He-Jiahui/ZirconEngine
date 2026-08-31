use std::collections::HashMap;

use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshIndirectDrawExecution, MeshIndirectResourceIdentity,
};

use super::HzbSampledResourceIdentity;

const MAX_HZB_OCCLUSION_BIND_GROUPS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct HzbOcclusionBindGroupKey {
    sampled_resource_id: u64,
    indirect_resources: MeshIndirectResourceIdentity,
}

impl HzbOcclusionBindGroupKey {
    const fn new(
        sampled_resource_identity: HzbSampledResourceIdentity,
        indirect_resources: MeshIndirectResourceIdentity,
    ) -> Self {
        Self {
            sampled_resource_id: sampled_resource_identity.get(),
            indirect_resources,
        }
    }
}

struct HzbOcclusionBindGroupEntry {
    bind_group: wgpu::BindGroup,
    last_used: u64,
}

#[derive(Default)]
pub(super) struct HzbOcclusionBindGroupCache {
    entries: HashMap<HzbOcclusionBindGroupKey, HzbOcclusionBindGroupEntry>,
    access_generation: u64,
}

pub(super) struct PreparedHzbOcclusionBindGroup<'a> {
    pub(super) bind_group: &'a wgpu::BindGroup,
    pub(super) created: bool,
}

impl HzbOcclusionBindGroupCache {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare<'a>(
        &'a mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampled_resource_identity: HzbSampledResourceIdentity,
        previous_hzb_view: &wgpu::TextureView,
        params_buffer: &wgpu::Buffer,
        stats_buffer: &wgpu::Buffer,
        execution: &MeshIndirectDrawExecution,
    ) -> PreparedHzbOcclusionBindGroup<'a> {
        let key =
            HzbOcclusionBindGroupKey::new(sampled_resource_identity, execution.resource_identity());
        let access_generation = self.next_access_generation();
        if self.entries.contains_key(&key) {
            let entry = self
                .entries
                .get_mut(&key)
                .expect("present HZB bind-group cache entry");
            entry.last_used = access_generation;
            return PreparedHzbOcclusionBindGroup {
                bind_group: &entry.bind_group,
                created: false,
            };
        }

        if self.entries.len() >= MAX_HZB_OCCLUSION_BIND_GROUPS {
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key)
            {
                self.entries.remove(&oldest_key);
            }
        }
        self.entries.insert(
            key,
            HzbOcclusionBindGroupEntry {
                bind_group: create_bind_group(
                    device,
                    layout,
                    previous_hzb_view,
                    params_buffer,
                    stats_buffer,
                    execution,
                ),
                last_used: access_generation,
            },
        );
        PreparedHzbOcclusionBindGroup {
            bind_group: &self
                .entries
                .get(&key)
                .expect("prepared HZB bind group")
                .bind_group,
            created: true,
        }
    }

    fn next_access_generation(&mut self) -> u64 {
        if self.access_generation == u64::MAX {
            let mut entries = self.entries.values_mut().collect::<Vec<_>>();
            entries.sort_unstable_by_key(|entry| entry.last_used);
            for (index, entry) in entries.into_iter().enumerate() {
                entry.last_used = index as u64 + 1;
            }
            self.access_generation = self.entries.len() as u64;
        }
        self.access_generation += 1;
        self.access_generation
    }
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    previous_hzb_view: &wgpu::TextureView,
    params_buffer: &wgpu::Buffer,
    stats_buffer: &wgpu::Buffer,
    execution: &MeshIndirectDrawExecution,
) -> wgpu::BindGroup {
    let compaction_resources = execution.compaction_resources();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hzb-occlusion-cull-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(previous_hzb_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: execution.args_buffer().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: compaction_resources.metadata_buffer().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: compaction_resources
                    .visible_instance_index_buffer()
                    .as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: compaction_resources.draw_count_buffer().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: compaction_resources
                    .compacted_indirect_args_buffer()
                    .as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: stats_buffer.as_entire_binding(),
            },
        ],
    })
}

#[cfg(test)]
mod hash_lru_tests;

#[cfg(test)]
mod tests {
    use super::{HzbOcclusionBindGroupKey, MAX_HZB_OCCLUSION_BIND_GROUPS};
    use crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectResourceIdentity;

    #[test]
    fn hzb_bind_group_cache_is_bounded() {
        assert_eq!(MAX_HZB_OCCLUSION_BIND_GROUPS, 64);
    }

    #[test]
    fn hzb_bind_group_key_tracks_sampled_texture_and_indirect_resource_revision() {
        let sampled_a = HzbSampledResourceIdentity::new();
        let sampled_b = HzbSampledResourceIdentity::new();
        let indirect_a = MeshIndirectResourceIdentity::new(7, 1);
        let indirect_b = MeshIndirectResourceIdentity::new(7, 2);

        let key = HzbOcclusionBindGroupKey::new(sampled_a, indirect_a);

        assert_ne!(key, HzbOcclusionBindGroupKey::new(sampled_b, indirect_a));
        assert_ne!(key, HzbOcclusionBindGroupKey::new(sampled_a, indirect_b));
        assert_eq!(key, HzbOcclusionBindGroupKey::new(sampled_a, indirect_a));
    }
}
