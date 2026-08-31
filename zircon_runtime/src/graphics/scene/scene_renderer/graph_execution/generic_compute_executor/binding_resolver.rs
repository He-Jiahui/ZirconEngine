use crate::render_graph::{
    BindingSchemaEntry, CompiledRenderGraphComputeBindingAccessPacket, ComputeBindingKind,
    RenderGraphComputePassMetadata, RenderGraphVersionedAccessKey,
};

use super::super::RenderPassGpuExecutionContext;
use super::super::compute_pipeline_cache::ComputePipelineBindingLayout;
use super::buffer_binding::{ResolvedComputeBuffer, resolve_compute_buffer};
use super::texture_view::{resolve_compute_texture_desc, resolve_compute_texture_view};

pub(super) struct ResolvedComputeBinding {
    pub(super) layout: ComputePipelineBindingLayout,
    resource: ResolvedComputeBindingResource,
}

impl ResolvedComputeBinding {
    pub(super) fn bind_group_entry(&self) -> wgpu::BindGroupEntry<'_> {
        let resource = match &self.resource {
            ResolvedComputeBindingResource::Buffer(buffer) => {
                wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer.buffer,
                    offset: buffer.offset,
                    size: buffer.size,
                })
            }
            ResolvedComputeBindingResource::TextureView(view) => {
                wgpu::BindingResource::TextureView(view)
            }
        };
        wgpu::BindGroupEntry {
            binding: self.layout.binding(),
            resource,
        }
    }
}

enum ResolvedComputeBindingResource {
    Buffer(ResolvedComputeBuffer),
    TextureView(wgpu::TextureView),
}

pub(super) fn resolve_bindings(
    gpu: &RenderPassGpuExecutionContext<'_>,
    metadata: &RenderGraphComputePassMetadata,
    access_packet: &CompiledRenderGraphComputeBindingAccessPacket,
) -> Result<Vec<ResolvedComputeBinding>, String> {
    let mut bindings = Vec::with_capacity(metadata.bindings.len());
    for binding in &metadata.bindings {
        let binding_access = binding_access(access_packet, binding)?;
        let resolved = match binding.kind {
            ComputeBindingKind::UniformBuffer => ResolvedComputeBinding {
                layout: ComputePipelineBindingLayout::uniform_buffer(binding.binding),
                resource: ResolvedComputeBindingResource::Buffer(resolve_compute_buffer(
                    gpu,
                    binding,
                    binding_access,
                )?),
            },
            ComputeBindingKind::StorageBufferRead => ResolvedComputeBinding {
                layout: ComputePipelineBindingLayout::storage_buffer_read(binding.binding),
                resource: ResolvedComputeBindingResource::Buffer(resolve_compute_buffer(
                    gpu,
                    binding,
                    binding_access,
                )?),
            },
            ComputeBindingKind::StorageBufferReadWrite => ResolvedComputeBinding {
                layout: ComputePipelineBindingLayout::storage_buffer_read_write(binding.binding),
                resource: ResolvedComputeBindingResource::Buffer(resolve_compute_buffer(
                    gpu,
                    binding,
                    binding_access,
                )?),
            },
            ComputeBindingKind::SampledTexture => {
                let desc = resolve_compute_texture_desc(gpu, binding, binding_access)?;
                ResolvedComputeBinding {
                    layout: ComputePipelineBindingLayout::sampled_texture(binding.binding, &desc)?,
                    resource: ResolvedComputeBindingResource::TextureView(
                        resolve_compute_texture_view(gpu, binding, binding_access)?,
                    ),
                }
            }
            ComputeBindingKind::StorageTextureWrite => {
                let desc = resolve_compute_texture_desc(gpu, binding, binding_access)?;
                ResolvedComputeBinding {
                    layout: ComputePipelineBindingLayout::storage_texture_write(
                        binding.binding,
                        &desc,
                    )?,
                    resource: ResolvedComputeBindingResource::TextureView(
                        resolve_compute_texture_view(gpu, binding, binding_access)?,
                    ),
                }
            }
        };
        bindings.push(resolved);
    }
    bindings.sort_by_key(|binding| binding.layout.binding());
    Ok(bindings)
}

fn binding_access(
    packet: &CompiledRenderGraphComputeBindingAccessPacket,
    binding: &BindingSchemaEntry,
) -> Result<RenderGraphVersionedAccessKey, String> {
    let compiled = packet.binding(binding.binding).ok_or_else(|| {
        format!(
            "compute binding packet for pass {:?} has no entry for binding {} resource `{}`",
            packet.pass, binding.binding, binding.resource
        )
    })?;
    if compiled.kind != binding.kind {
        return Err(format!(
            "compute binding packet for pass {:?} binding {} has kind {:?}, expected {:?}",
            packet.pass, binding.binding, compiled.kind, binding.kind
        ));
    }
    let access = match binding.kind {
        ComputeBindingKind::UniformBuffer
        | ComputeBindingKind::StorageBufferRead
        | ComputeBindingKind::SampledTexture => compiled.read_access,
        ComputeBindingKind::StorageBufferReadWrite | ComputeBindingKind::StorageTextureWrite => {
            compiled.write_access
        }
    };
    access.ok_or_else(|| {
        format!(
            "compute binding packet for pass {:?} binding {} resource `{}` has no required exact access",
            packet.pass, binding.binding, binding.resource
        )
    })
}
