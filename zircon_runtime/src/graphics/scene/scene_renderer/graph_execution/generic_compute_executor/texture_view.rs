use crate::render_graph::{BindingSchemaEntry, RenderGraphResourceAccessKind};

use super::RenderPassGpuExecutionContext;

pub(super) fn resolve_compute_texture_view(
    gpu: &RenderPassGpuExecutionContext<'_>,
    binding: &BindingSchemaEntry,
    access: RenderGraphResourceAccessKind,
) -> Result<wgpu::TextureView, String> {
    if binding.texture_full_mip_chain {
        return gpu.texture_view_with_full_mip_fallback(&binding.resource, access);
    }
    match binding.texture_mip_level {
        Some(mip_level) => gpu.require_owned_texture_mip_view(&binding.resource, access, mip_level),
        None => gpu.require_texture_view(&binding.resource, access).cloned(),
    }
}
