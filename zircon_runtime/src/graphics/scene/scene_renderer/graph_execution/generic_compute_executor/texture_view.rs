use crate::render_graph::{
    BindingSchemaEntry, RenderGraphResourceKind, RenderGraphVersionedAccessKey,
};
use crate::rhi::TextureDesc;

use super::RenderPassGpuExecutionContext;

pub(super) fn resolve_compute_texture_view(
    gpu: &RenderPassGpuExecutionContext<'_>,
    binding: &BindingSchemaEntry,
    binding_access: RenderGraphVersionedAccessKey,
) -> Result<wgpu::TextureView, String> {
    match binding_access.resource.kind() {
        RenderGraphResourceKind::TransientTexture => {
            return gpu
                .resources
                .transient_texture_view_for_access(binding_access.access_id)
                .cloned();
        }
        // External physical views are resolved from the compiler's immutable access-ID packet.
        // Descriptor-less report-only imports remain valid for view-only compatibility; any
        // schema that needs a physical descriptor fails closed in `resolve_compute_texture_desc`.
        RenderGraphResourceKind::External => {
            return gpu
                .resources
                .external_texture_view_for_access(binding_access.access_id)
                .cloned();
        }
        RenderGraphResourceKind::TransientBuffer => {
            return Err(format!(
                "compute binding `{}` resource `{}` resolves to a transient buffer access, not a texture",
                binding.binding, binding.resource
            ));
        }
    }
    if binding.texture_full_mip_chain {
        return gpu.texture_view_with_full_mip_fallback(&binding.resource, binding_access.access);
    }
    match binding.texture_mip_level {
        Some(mip_level) => {
            gpu.require_owned_texture_mip_view(&binding.resource, binding_access.access, mip_level)
        }
        None => gpu
            .require_texture_view(&binding.resource, binding_access.access)
            .cloned(),
    }
}

pub(super) fn resolve_compute_texture_desc(
    gpu: &RenderPassGpuExecutionContext<'_>,
    binding: &BindingSchemaEntry,
    binding_access: RenderGraphVersionedAccessKey,
) -> Result<TextureDesc, String> {
    match binding_access.resource.kind() {
        RenderGraphResourceKind::TransientTexture => {
            let resolver = gpu.resource_resolver().ok_or_else(|| {
                format!(
                    "compute binding `{}` resource `{}` has no compiled resolver for exact transient access {:?}",
                    binding.binding, binding.resource, binding_access.access_id
                )
            })?;
            let declaration = resolver
                .resource_declaration(binding_access.resource)
                .ok_or_else(|| {
                    format!(
                        "compute binding `{}` resource `{}` exact access {:?} has no resource declaration",
                        binding.binding, binding.resource, binding_access.access_id
                    )
                })?;
            gpu.resources
                .require_texture_desc_for_declaration(declaration)
        }
        RenderGraphResourceKind::External => gpu
            .resources
            .external_texture_desc_for_access(binding_access.access_id),
        RenderGraphResourceKind::TransientBuffer => Err(format!(
            "compute binding `{}` resource `{}` resolves to a transient buffer access, not a texture descriptor",
            binding.binding, binding.resource
        )),
    }
}
