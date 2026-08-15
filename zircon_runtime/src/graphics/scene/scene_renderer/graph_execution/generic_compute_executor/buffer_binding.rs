use crate::render_graph::{BindingSchemaEntry, ComputeBindingKind, RenderGraphResourceAccessKind};

use super::RenderPassGpuExecutionContext;

pub(super) struct ResolvedComputeBuffer {
    pub(super) buffer: wgpu::Buffer,
    pub(super) offset: u64,
}

pub(super) fn resolve_compute_buffer(
    gpu: &RenderPassGpuExecutionContext<'_>,
    binding: &BindingSchemaEntry,
    access: RenderGraphResourceAccessKind,
) -> Result<ResolvedComputeBuffer, String> {
    let offset = binding.buffer_offset.unwrap_or_default();
    validate_buffer_binding_offset(binding, offset, &gpu.device.limits())?;
    let buffer = gpu.require_buffer(&binding.resource, access)?.clone();
    if offset >= buffer.size() {
        return Err(format!(
            "compute buffer binding `{}` for resource `{}` starts at offset {offset}, outside its {} byte buffer",
            binding.binding,
            binding.resource,
            buffer.size()
        ));
    }
    Ok(ResolvedComputeBuffer { buffer, offset })
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
    use crate::render_graph::{BindingSchemaEntry, ComputeBindingKind};

    use super::validate_buffer_binding_offset;

    #[test]
    fn buffer_offsets_follow_the_device_binding_alignment() {
        let limits = wgpu::Limits {
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 16,
            ..wgpu::Limits::default()
        };
        let uniform = BindingSchemaEntry::new(0, "params", ComputeBindingKind::UniformBuffer)
            .with_buffer_offset(256);
        let storage = BindingSchemaEntry::new(1, "weights", ComputeBindingKind::StorageBufferRead)
            .with_buffer_offset(16);

        assert!(validate_buffer_binding_offset(&uniform, 256, &limits).is_ok());
        assert!(validate_buffer_binding_offset(&storage, 16, &limits).is_ok());
        assert!(validate_buffer_binding_offset(&uniform, 16, &limits)
            .expect_err("uniform offsets must honor the device alignment")
            .contains("align to 256 bytes"));
    }
}
