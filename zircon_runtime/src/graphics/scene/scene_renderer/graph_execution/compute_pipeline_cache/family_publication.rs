use std::collections::HashMap;

use crate::render_graph::RenderGraphComputePipelineFamily;

use super::ComputePipelineBindingLayout;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ComputePipelineFamilyKey {
    family: RenderGraphComputePipelineFamily,
    entry_point: String,
    expected_workgroup_size: [u32; 3],
    bindings: Vec<ComputePipelineBindingLayout>,
}

impl ComputePipelineFamilyKey {
    pub(super) fn new(
        family: RenderGraphComputePipelineFamily,
        entry_point: &str,
        expected_workgroup_size: [u32; 3],
        bindings: &[ComputePipelineBindingLayout],
    ) -> Self {
        Self {
            family,
            entry_point: entry_point.to_string(),
            expected_workgroup_size,
            bindings: bindings.to_vec(),
        }
    }

    pub(super) const fn family(&self) -> &RenderGraphComputePipelineFamily {
        &self.family
    }
}

struct PublishedComputePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    artifact_fingerprint: u64,
    last_used: u64,
}

pub(super) struct PublishedComputePipelineResolution {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub artifact_fingerprint: u64,
}

pub(super) struct ComputePipelineFamilyPublicationCache {
    capacity: usize,
    families: HashMap<ComputePipelineFamilyKey, PublishedComputePipeline>,
}

impl ComputePipelineFamilyPublicationCache {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            families: HashMap::new(),
        }
    }

    pub(super) fn clear(&mut self) {
        self.families.clear();
    }

    pub(super) fn publish(
        &mut self,
        family_key: ComputePipelineFamilyKey,
        pipeline: &wgpu::ComputePipeline,
        bind_group_layout: &wgpu::BindGroupLayout,
        artifact_fingerprint: u64,
        use_counter: u64,
    ) {
        if !self.families.contains_key(&family_key) && self.families.len() >= self.capacity {
            self.evict_lru();
        }
        self.families.insert(
            family_key,
            PublishedComputePipeline {
                pipeline: pipeline.clone(),
                bind_group_layout: bind_group_layout.clone(),
                artifact_fingerprint,
                last_used: use_counter,
            },
        );
    }

    pub(super) fn resolve(
        &mut self,
        family_key: &ComputePipelineFamilyKey,
        use_counter: u64,
    ) -> Option<PublishedComputePipelineResolution> {
        let published = self.families.get_mut(family_key)?;
        published.last_used = use_counter;
        Some(PublishedComputePipelineResolution {
            pipeline: published.pipeline.clone(),
            bind_group_layout: published.bind_group_layout.clone(),
            artifact_fingerprint: published.artifact_fingerprint,
        })
    }

    fn evict_lru(&mut self) {
        let Some(key) = self
            .families
            .iter()
            .min_by_key(|(_, published)| published.last_used)
            .map(|(key, _)| key.clone())
        else {
            return;
        };
        self.families.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use crate::render_graph::RenderGraphComputePipelineFamily;

    use super::{ComputePipelineBindingLayout, ComputePipelineFamilyKey};

    #[test]
    fn family_key_isolates_interface_workgroup_and_binding_abi() {
        let baseline = ComputePipelineFamilyKey::new(
            RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 2),
            "cs_main",
            [8, 8, 1],
            &[ComputePipelineBindingLayout::uniform_buffer(0)],
        );
        let same = ComputePipelineFamilyKey::new(
            RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 2),
            "cs_main",
            [8, 8, 1],
            &[ComputePipelineBindingLayout::uniform_buffer(0)],
        );
        let changed_interface = ComputePipelineFamilyKey::new(
            RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 3),
            "cs_main",
            [8, 8, 1],
            &[ComputePipelineBindingLayout::uniform_buffer(0)],
        );
        let changed_workgroup = ComputePipelineFamilyKey::new(
            RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 2),
            "cs_main",
            [16, 8, 1],
            &[ComputePipelineBindingLayout::uniform_buffer(0)],
        );
        let changed_binding = ComputePipelineFamilyKey::new(
            RenderGraphComputePipelineFamily::new("ambient-occlusion.evaluate", 2),
            "cs_main",
            [8, 8, 1],
            &[ComputePipelineBindingLayout::storage_buffer_read(0)],
        );

        assert_eq!(baseline, same);
        assert_ne!(baseline, changed_interface);
        assert_ne!(baseline, changed_workgroup);
        assert_ne!(baseline, changed_binding);
    }
}
