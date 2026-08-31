use zr_rhi::{
    GpuMemoryClass, GpuMemorySnapshot, RhiError, SubmissionTicket, TransientAllocatorStats,
};

use crate::resource_validation::texture_storage_size;

use super::{WgpuResourceRegistry, WgpuRetiredResource, WgpuRetirement};

impl WgpuResourceRegistry {
    pub(super) fn ensure_buffer_capacity(&self, requested_bytes: u64) -> Result<(), RhiError> {
        let snapshot = self.memory_snapshot();
        ensure_memory_capacity(
            GpuMemoryClass::Buffer,
            snapshot
                .active_buffer_bytes
                .saturating_add(snapshot.retired_buffer_bytes),
            requested_bytes,
            self.memory_budget.transient_buffer_bytes(),
        )
    }

    pub(super) fn ensure_texture_capacity(&self, requested_bytes: u64) -> Result<(), RhiError> {
        let snapshot = self.memory_snapshot();
        ensure_memory_capacity(
            GpuMemoryClass::Texture,
            snapshot
                .active_texture_bytes
                .saturating_add(snapshot.retired_texture_bytes),
            requested_bytes,
            self.memory_budget.transient_texture_bytes(),
        )
    }
    pub(super) fn retire_native(
        &mut self,
        resource: WgpuRetiredResource,
        last_uses: Vec<SubmissionTicket>,
    ) {
        if !last_uses.is_empty() {
            self.retired.push(WgpuRetirement {
                after: last_uses,
                resource,
            });
        }
    }

    /// Drops only native objects whose every observed ticket is terminal.
    pub(crate) fn reap_retired(
        &mut self,
        mut is_terminal: impl FnMut(SubmissionTicket) -> bool,
    ) -> usize {
        let before = self.retired.len();
        self.retired
            .retain(|retirement| !retirement.after.iter().all(|ticket| is_terminal(*ticket)));
        before.saturating_sub(self.retired.len())
    }

    /// Removes completed ticket dependencies from live resources before a
    /// later destroy transfers their remaining uses to the retirement queue.
    pub(crate) fn prune_terminal_uses(
        &mut self,
        mut is_terminal: impl FnMut(SubmissionTicket) -> bool,
    ) {
        for resource in self.buffers.values_mut() {
            resource.last_uses.retain(|ticket| !is_terminal(*ticket));
        }
        for resource in self.textures.values_mut() {
            resource.last_uses.retain(|ticket| !is_terminal(*ticket));
        }
        for resource in self.texture_views.values_mut() {
            resource.last_uses.retain(|ticket| !is_terminal(*ticket));
        }
        for resource in self.samplers.values_mut() {
            resource.last_uses.retain(|ticket| !is_terminal(*ticket));
        }
        for resource in self.bind_group_layouts.values_mut() {
            resource.last_uses.retain(|ticket| !is_terminal(*ticket));
        }
        for resource in self.bind_groups.values_mut() {
            resource.last_uses.retain(|ticket| !is_terminal(*ticket));
        }
        for resource in self.shader_modules.values_mut() {
            resource.last_uses.retain(|ticket| !is_terminal(*ticket));
        }
        for resource in self.pipeline_layouts.values_mut() {
            resource.last_uses.retain(|ticket| !is_terminal(*ticket));
        }
        for resource in self.pipelines.values_mut() {
            resource
                .last_uses_mut()
                .retain(|ticket| !is_terminal(*ticket));
        }
    }

    pub(crate) fn memory_snapshot(&self) -> GpuMemorySnapshot {
        let active_buffer_bytes = self
            .buffers
            .values()
            .map(|resource| resource.desc.size_bytes)
            .fold(0_u64, u64::saturating_add);
        let active_texture_bytes = self
            .textures
            .values()
            .map(|resource| texture_storage_size(&resource.desc))
            .fold(0_u64, u64::saturating_add);
        let (retired_buffer_bytes, retired_texture_bytes, retired_allocations) =
            self.retired.iter().fold(
                (0_u64, 0_u64, 0_u32),
                |(buffer_bytes, texture_bytes, allocations), retirement| match &retirement.resource
                {
                    WgpuRetiredResource::Buffer(resource) => (
                        buffer_bytes.saturating_add(resource.desc.size_bytes),
                        texture_bytes,
                        allocations.saturating_add(1),
                    ),
                    WgpuRetiredResource::Texture(resource) => (
                        buffer_bytes,
                        texture_bytes.saturating_add(texture_storage_size(&resource.desc)),
                        allocations.saturating_add(1),
                    ),
                    WgpuRetiredResource::TextureView(_)
                    | WgpuRetiredResource::Sampler(_)
                    | WgpuRetiredResource::BindGroupLayout(_)
                    | WgpuRetiredResource::BindGroup(_)
                    | WgpuRetiredResource::ShaderModule(_)
                    | WgpuRetiredResource::PipelineLayout(_)
                    | WgpuRetiredResource::Pipeline(_) => {
                        (buffer_bytes, texture_bytes, allocations)
                    }
                },
            );
        GpuMemorySnapshot {
            active_buffer_bytes,
            active_texture_bytes,
            retired_buffer_bytes,
            retired_texture_bytes,
            active_allocations: saturating_u32(
                self.buffers.len().saturating_add(self.textures.len()),
            ),
            retired_allocations,
            ..GpuMemorySnapshot::default()
        }
    }

    pub(super) fn physical_allocator_stats(&self) -> TransientAllocatorStats {
        let snapshot = self.memory_snapshot();
        TransientAllocatorStats {
            bytes_reserved: snapshot.reserved_resource_bytes(),
            allocations: snapshot.reserved_resource_allocations(),
        }
    }
}

const fn saturating_u32(value: usize) -> u32 {
    if value > u32::MAX as usize {
        u32::MAX
    } else {
        value as u32
    }
}

fn ensure_memory_capacity(
    class: GpuMemoryClass,
    current_bytes: u64,
    requested_bytes: u64,
    limit_bytes: u64,
) -> Result<(), RhiError> {
    if requested_bytes > limit_bytes.saturating_sub(current_bytes) {
        return Err(RhiError::MemoryBudgetExceeded {
            class,
            current_bytes,
            requested_bytes,
            limit_bytes,
        });
    }
    Ok(())
}
