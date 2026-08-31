use std::num::NonZeroU64;

use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, RenderGraphResourceKind, RenderGraphVersionedAccessKey,
};

use super::RenderPassGpuExecutionContext;

pub(super) struct ResolvedComputeBuffer {
    pub(super) buffer: wgpu::Buffer,
    pub(super) offset: u64,
    pub(super) size: Option<NonZeroU64>,
}

struct ResolvedComputeBufferRange {
    offset: u64,
    size: Option<NonZeroU64>,
}

pub(super) fn resolve_compute_buffer(
    gpu: &RenderPassGpuExecutionContext<'_>,
    binding: &BindingSchemaEntry,
    binding_access: RenderGraphVersionedAccessKey,
) -> Result<ResolvedComputeBuffer, String> {
    let (buffer, range) = match binding_access.resource.kind() {
        RenderGraphResourceKind::TransientBuffer => {
            let (buffer, range) = gpu
                .resources
                .transient_buffer_binding_for_access(binding_access.access_id)?;
            (buffer.clone(), range)
        }
        // External physical leases are resolved from the compiler's immutable access-ID packet.
        // An unresolved report-only import intentionally receives the full physical buffer range
        // as a compatibility boundary; typed access ranges remain exact below.
        RenderGraphResourceKind::External => {
            let (buffer, lease_range) = gpu
                .resources
                .external_buffer_binding_for_access(binding_access.access_id)?;
            let buffer = buffer.clone();
            let requested =
                resolve_buffer_binding_range(binding, buffer.size(), &gpu.device.limits())?;
            let requested_end = requested
                .size
                .map_or(buffer.size(), |size| requested.offset + size.get());
            if matches!(
                binding_access.key.range,
                crate::render_graph::RenderGraphResourceAccessRange::Buffer(_)
            ) && (requested.offset != lease_range.start || requested_end != lease_range.end)
            {
                return Err(format!(
                    "compute binding `{}` resource `{}` schema range [{}..{}) differs from external lease range [{}..{})",
                    binding.binding,
                    binding.resource,
                    requested.offset,
                    requested_end,
                    lease_range.start,
                    lease_range.end,
                ));
            }
            return Ok(ResolvedComputeBuffer {
                buffer,
                offset: lease_range.start,
                size: NonZeroU64::new(lease_range.end - lease_range.start),
            });
        }
        RenderGraphResourceKind::TransientTexture => {
            return Err(format!(
                "compute binding `{}` resource `{}` resolves to a transient texture access, not a buffer",
                binding.binding, binding.resource
            ));
        }
    };
    let requested = resolve_buffer_binding_range(binding, buffer.size(), &gpu.device.limits())?;
    let exact_size = range.end.checked_sub(range.start).ok_or_else(|| {
        format!(
            "compute binding `{}` resource `{}` exact access range is inverted",
            binding.binding, binding.resource
        )
    })?;
    let requested_end = requested
        .size
        .map_or(buffer.size(), |size| requested.offset + size.get());
    if requested.offset != range.start || requested_end != range.end {
        return Err(format!(
            "compute binding `{}` resource `{}` schema range [{}..{}) differs from compiler access {:?} range [{}..{})",
            binding.binding,
            binding.resource,
            requested.offset,
            requested_end,
            binding_access.access_id,
            range.start,
            range.end,
        ));
    }
    let size = NonZeroU64::new(exact_size).ok_or_else(|| {
        format!(
            "compute binding `{}` resource `{}` exact access range must not be empty",
            binding.binding, binding.resource
        )
    })?;
    Ok(ResolvedComputeBuffer {
        buffer,
        offset: range.start,
        size: Some(size),
    })
}

fn resolve_buffer_binding_range(
    binding: &BindingSchemaEntry,
    buffer_size: u64,
    limits: &wgpu::Limits,
) -> Result<ResolvedComputeBufferRange, String> {
    let (offset, requested_size) = binding
        .buffer_range
        .map(|range| (range.offset, range.size))
        .unwrap_or((0, None));
    validate_buffer_binding_offset(binding, offset, limits)?;
    let available = buffer_size.checked_sub(offset).ok_or_else(|| {
        format!(
            "compute buffer binding `{}` for resource `{}` starts at offset {offset}, outside its {buffer_size} byte buffer",
            binding.binding, binding.resource
        )
    })?;
    let size = match requested_size {
        Some(0) => {
            return Err(format!(
                "compute buffer binding `{}` for resource `{}` range offset {offset} must not be empty",
                binding.binding, binding.resource
            ));
        }
        Some(size) if size > available => {
            return Err(format!(
                "compute buffer binding `{}` for resource `{}` range offset {offset} size {size} exceeds its {buffer_size} byte buffer",
                binding.binding, binding.resource
            ));
        }
        Some(size) => NonZeroU64::new(size),
        None if available == 0 => {
            return Err(format!(
                "compute buffer binding `{}` for resource `{}` starts at offset {offset}, outside its {buffer_size} byte buffer",
                binding.binding, binding.resource
            ));
        }
        None => None,
    };
    Ok(ResolvedComputeBufferRange { offset, size })
}

fn validate_buffer_binding_offset(
    binding: &BindingSchemaEntry,
    offset: u64,
    limits: &wgpu::Limits,
) -> Result<(), String> {
    let (alignment, kind) = match binding.kind {
        ComputeBindingKind::UniformBuffer => {
            (limits.min_uniform_buffer_offset_alignment, "uniform")
        }
        ComputeBindingKind::StorageBufferRead | ComputeBindingKind::StorageBufferReadWrite => {
            (limits.min_storage_buffer_offset_alignment, "storage")
        }
        ComputeBindingKind::SampledTexture | ComputeBindingKind::StorageTextureWrite => {
            return Err(format!(
                "compute binding `{}` selects buffer offset {offset}, but `{}` is not a buffer binding",
                binding.binding, binding.resource
            ));
        }
    };
    if offset % u64::from(alignment) != 0 {
        return Err(format!(
            "compute {kind} buffer binding `{}` for resource `{}` offset {offset} must align to {alignment} bytes",
            binding.binding, binding.resource
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use crate::render_graph::{BindingSchemaEntry, ComputeBindingKind};

    use super::{resolve_buffer_binding_range, validate_buffer_binding_offset};

    #[test]
    fn buffer_offsets_follow_the_device_binding_alignment() {
        let limits = wgpu::Limits {
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 16,
            ..wgpu::Limits::default()
        };
        let uniform = BindingSchemaEntry::new(0, "params", ComputeBindingKind::UniformBuffer)
            .with_buffer_range(256, Some(512));
        let storage = BindingSchemaEntry::new(1, "weights", ComputeBindingKind::StorageBufferRead)
            .with_buffer_range(16, Some(128));

        assert!(validate_buffer_binding_offset(&uniform, 256, &limits).is_ok());
        assert!(validate_buffer_binding_offset(&storage, 16, &limits).is_ok());
        assert!(
            validate_buffer_binding_offset(&uniform, 16, &limits)
                .expect_err("uniform offsets must honor the device alignment")
                .contains("align to 256 bytes")
        );
    }

    #[test]
    fn buffer_ranges_preserve_explicit_nonzero_binding_windows() {
        let limits = wgpu::Limits {
            min_uniform_buffer_offset_alignment: 256,
            ..wgpu::Limits::default()
        };
        let binding = BindingSchemaEntry::new(0, "params", ComputeBindingKind::UniformBuffer)
            .with_buffer_range(256, Some(512));

        let resolved = resolve_buffer_binding_range(&binding, 1_024, &limits)
            .expect("a contained nonzero range is valid");

        assert_eq!(resolved.offset, 256);
        assert_eq!(resolved.size, Some(NonZeroU64::new(512).unwrap()));
    }

    #[test]
    fn buffer_ranges_reject_empty_or_out_of_bounds_windows() {
        let limits = wgpu::Limits {
            min_storage_buffer_offset_alignment: 16,
            ..wgpu::Limits::default()
        };
        let empty = BindingSchemaEntry::new(0, "params", ComputeBindingKind::UniformBuffer)
            .with_buffer_range(0, Some(0));
        let overrun = BindingSchemaEntry::new(1, "params", ComputeBindingKind::StorageBufferRead)
            .with_buffer_range(16, Some(1_009));

        assert!(
            resolve_buffer_binding_range(&empty, 1_024, &limits)
                .expect_err("empty windows cannot produce a WGPU buffer binding")
                .contains("must not be empty")
        );
        assert!(
            resolve_buffer_binding_range(&overrun, 1_024, &limits)
                .expect_err("ranges must fit their resolved buffer")
                .contains("exceeds its 1024 byte buffer")
        );
    }
}
