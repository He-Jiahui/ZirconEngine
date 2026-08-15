use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, RenderGraphComputePassMetadata,
    RenderGraphResourceAccessKind,
};

use super::super::compute_pipeline_cache::ComputePipelineBindingLayout;
use super::super::RenderPassGpuExecutionContext;
use super::buffer_binding::{resolve_compute_buffer, ResolvedComputeBuffer};
use super::texture_view::resolve_compute_texture_view;

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
                    size: None,
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
) -> Result<Vec<ResolvedComputeBinding>, String> {
    let mut bindings = Vec::with_capacity(metadata.bindings.len());
    for binding in &metadata.bindings {
        let resolved = match binding.kind {
            ComputeBindingKind::UniformBuffer => ResolvedComputeBinding {
                layout: ComputePipelineBindingLayout::uniform_buffer(binding.binding),
                resource: ResolvedComputeBindingResource::Buffer(resolve_compute_buffer(
                    gpu,
                    binding,
                    RenderGraphResourceAccessKind::Read,
                )?),
            },
            ComputeBindingKind::StorageBufferRead => ResolvedComputeBinding {
                layout: ComputePipelineBindingLayout::storage_buffer_read(binding.binding),
                resource: ResolvedComputeBindingResource::Buffer(resolve_compute_buffer(
                    gpu,
                    binding,
                    RenderGraphResourceAccessKind::Read,
                )?),
            },
            ComputeBindingKind::StorageBufferReadWrite => ResolvedComputeBinding {
                layout: ComputePipelineBindingLayout::storage_buffer_read_write(binding.binding),
                resource: ResolvedComputeBindingResource::Buffer(resolve_compute_buffer(
                    gpu,
                    binding,
                    RenderGraphResourceAccessKind::Write,
                )?),
            },
            ComputeBindingKind::SampledTexture => {
                let desc = gpu
                    .require_texture_desc(&binding.resource, RenderGraphResourceAccessKind::Read)?;
                ResolvedComputeBinding {
                    layout: ComputePipelineBindingLayout::sampled_texture(binding.binding, &desc)?,
                    resource: ResolvedComputeBindingResource::TextureView(
                        resolve_compute_texture_view(
                            gpu,
                            binding,
                            RenderGraphResourceAccessKind::Read,
                        )?,
                    ),
                }
            }
            ComputeBindingKind::StorageTextureWrite => {
                let desc = gpu.require_texture_desc(
                    &binding.resource,
                    RenderGraphResourceAccessKind::Write,
                )?;
                ResolvedComputeBinding {
                    layout: ComputePipelineBindingLayout::storage_texture_write(
                        binding.binding,
                        &desc,
                    )?,
                    resource: ResolvedComputeBindingResource::TextureView(
                        resolve_compute_texture_view(
                            gpu,
                            binding,
                            RenderGraphResourceAccessKind::Write,
                        )?,
                    ),
                }
            }
        };
        bindings.push(resolved);
    }
    bindings.sort_by_key(|binding| binding.layout.binding());
    Ok(bindings)
}
