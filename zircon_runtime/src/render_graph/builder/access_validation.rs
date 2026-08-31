use std::collections::HashMap;

use crate::rhi::{BufferUsage, TextureDesc, TextureDimension, TextureUsage};

use super::super::access::{
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessRange, RenderGraphTextureAspect,
    RenderGraphTextureSubresourceRange,
};
use super::super::error::RenderGraphError;
use super::super::types::{
    RenderGraphResource, RenderGraphResourceAccessKind, RenderGraphResourceDesc,
};
use super::{RenderGraphBuilder, ResourceAccessKind, ResourceNode};

pub(super) fn validate_resource_access_ranges(
    builder: &RenderGraphBuilder,
    resource_names: &HashMap<RenderGraphResource, &str>,
) -> Result<(), RenderGraphError> {
    let resources = builder
        .resources
        .iter()
        .map(|resource| (resource.resource, resource))
        .collect::<HashMap<_, _>>();

    for pass in &builder.passes {
        for access in &pass.resources {
            let resource_name = resource_names
                .get(&access.resource)
                .copied()
                .unwrap_or("<missing resource>");
            let node = resources.get(&access.resource).copied().ok_or_else(|| {
                RenderGraphError::ResourceDeclarationMissing {
                    resource: resource_name.to_owned(),
                }
            })?;

            match access.metadata.range {
                RenderGraphResourceAccessRange::Texture(range) => {
                    let Some(desc) = texture_desc(node) else {
                        return Err(RenderGraphError::TextureAccessRangeRequiresTexture {
                            pass: pass.name.clone(),
                            resource: resource_name.to_owned(),
                        });
                    };
                    validate_texture_range(&pass.name, resource_name, range, desc)?;
                }
                RenderGraphResourceAccessRange::Buffer(range) => {
                    let Some(size_bytes) = buffer_size(node) else {
                        return Err(RenderGraphError::BufferAccessRangeRequiresBuffer {
                            pass: pass.name.clone(),
                            resource: resource_name.to_owned(),
                        });
                    };
                    validate_buffer_range(
                        &pass.name,
                        resource_name,
                        range.offset,
                        range.size,
                        size_bytes,
                    )?;
                }
                RenderGraphResourceAccessRange::UnresolvedExternal
                    if !matches!(
                        access.metadata.intent,
                        RenderGraphResourceAccessIntent::Legacy
                    ) =>
                {
                    return Err(RenderGraphError::UnresolvedExternalAccessMetadata {
                        pass: pass.name.clone(),
                        resource: resource_name.to_owned(),
                    });
                }
                RenderGraphResourceAccessRange::UnresolvedExternal => {}
            }
            validate_access_intent(
                &pass.name,
                resource_name,
                access.kind,
                access.metadata.intent,
                node,
            )?;
        }
    }

    Ok(())
}

fn texture_desc(node: &ResourceNode) -> Option<&TextureDesc> {
    match &node.desc {
        RenderGraphResourceDesc::Texture(desc) => Some(desc),
        RenderGraphResourceDesc::External => node.external_texture_desc.as_ref(),
        RenderGraphResourceDesc::Buffer(_) => None,
    }
}

fn buffer_size(node: &ResourceNode) -> Option<u64> {
    match &node.desc {
        RenderGraphResourceDesc::Buffer(desc) => Some(desc.size_bytes),
        RenderGraphResourceDesc::External => node
            .external_buffer_desc
            .as_ref()
            .map(|desc| desc.size_bytes),
        RenderGraphResourceDesc::Texture(_) => None,
    }
}

fn validate_texture_range(
    pass: &str,
    resource: &str,
    range: RenderGraphTextureSubresourceRange,
    desc: &TextureDesc,
) -> Result<(), RenderGraphError> {
    if range_end_exceeds_u32(range.base_mip_level, range.mip_level_count, desc.mip_levels) {
        return Err(RenderGraphError::TextureAccessMipRangeOutOfBounds {
            pass: pass.to_owned(),
            resource: resource.to_owned(),
            base_mip_level: range.base_mip_level,
            mip_level_count: range.mip_level_count,
            mip_levels: desc.mip_levels,
        });
    }

    let array_layers = match desc.dimension {
        TextureDimension::D2Array | TextureDimension::Cube => desc.depth,
        TextureDimension::D1 | TextureDimension::D2 | TextureDimension::D3 => 1,
    };
    if range_end_exceeds_u32(
        range.base_array_layer,
        range.array_layer_count,
        array_layers,
    ) {
        return Err(RenderGraphError::TextureAccessArrayLayerRangeOutOfBounds {
            pass: pass.to_owned(),
            resource: resource.to_owned(),
            base_array_layer: range.base_array_layer,
            array_layer_count: range.array_layer_count,
            array_layers,
        });
    }

    let aspect_supported = match range.aspect {
        RenderGraphTextureAspect::All => true,
        RenderGraphTextureAspect::Color => !desc.format.is_depth(),
        RenderGraphTextureAspect::Depth => desc.format.is_depth(),
        RenderGraphTextureAspect::Stencil => desc.format.has_stencil(),
    };
    if !aspect_supported {
        return Err(RenderGraphError::TextureAccessAspectUnsupported {
            pass: pass.to_owned(),
            resource: resource.to_owned(),
            aspect: range.aspect,
            format: desc.format,
        });
    }

    Ok(())
}

fn validate_buffer_range(
    pass: &str,
    resource: &str,
    offset: u64,
    size: Option<u64>,
    buffer_size: u64,
) -> Result<(), RenderGraphError> {
    if matches!(size, Some(0)) {
        return Err(RenderGraphError::BufferAccessRangeEmpty {
            pass: pass.to_owned(),
            resource: resource.to_owned(),
        });
    }
    if range_end_exceeds_u64(offset, size, buffer_size) {
        return Err(RenderGraphError::BufferAccessRangeOutOfBounds {
            pass: pass.to_owned(),
            resource: resource.to_owned(),
            offset,
            size,
            buffer_size,
        });
    }

    Ok(())
}

fn validate_access_intent(
    pass: &str,
    resource: &str,
    access: ResourceAccessKind,
    intent: RenderGraphResourceAccessIntent,
    node: &ResourceNode,
) -> Result<(), RenderGraphError> {
    if matches!(intent, RenderGraphResourceAccessIntent::Legacy) {
        return Ok(());
    }
    if let Some(stages) = shader_stages(intent) {
        if stages.is_empty() {
            return Err(RenderGraphError::ResourceAccessIntentShaderStagesEmpty {
                pass: pass.to_owned(),
                resource: resource.to_owned(),
                intent,
            });
        }
    }
    if intent_requires_write(intent) != matches!(access, ResourceAccessKind::Write) {
        return Err(RenderGraphError::ResourceAccessIntentKindMismatch {
            pass: pass.to_owned(),
            resource: resource.to_owned(),
            declared_access: render_graph_access_kind(access),
            intent,
        });
    }

    if let Some(required) = texture_usage_for_intent(intent) {
        let Some(desc) = texture_desc(node) else {
            return Err(RenderGraphError::ResourceAccessIntentRequiresTexture {
                pass: pass.to_owned(),
                resource: resource.to_owned(),
                intent,
            });
        };
        if !desc.usage.contains(required) {
            return Err(RenderGraphError::TextureAccessIntentUsageMissing {
                pass: pass.to_owned(),
                resource: resource.to_owned(),
                intent,
                required,
                actual: desc.usage,
            });
        }
        return Ok(());
    }

    if let Some(required) = buffer_usage_for_intent(intent) {
        let Some(usage) = buffer_usage(node) else {
            return Err(RenderGraphError::ResourceAccessIntentRequiresBuffer {
                pass: pass.to_owned(),
                resource: resource.to_owned(),
                intent,
            });
        };
        if !usage.contains(required) {
            return Err(RenderGraphError::BufferAccessIntentUsageMissing {
                pass: pass.to_owned(),
                resource: resource.to_owned(),
                intent,
                required,
                actual: usage,
            });
        }
        return Ok(());
    }

    if let Some((texture_usage, required_buffer_usage)) = copy_usage_for_intent(intent) {
        if let Some(desc) = texture_desc(node) {
            if !desc.usage.contains(texture_usage) {
                return Err(RenderGraphError::TextureAccessIntentUsageMissing {
                    pass: pass.to_owned(),
                    resource: resource.to_owned(),
                    intent,
                    required: texture_usage,
                    actual: desc.usage,
                });
            }
            return Ok(());
        }
        if let Some(actual) = buffer_usage(node) {
            if !actual.contains(required_buffer_usage) {
                return Err(RenderGraphError::BufferAccessIntentUsageMissing {
                    pass: pass.to_owned(),
                    resource: resource.to_owned(),
                    intent,
                    required: required_buffer_usage,
                    actual,
                });
            }
            return Ok(());
        }
    }

    Ok(())
}

fn shader_stages(
    intent: RenderGraphResourceAccessIntent,
) -> Option<super::super::access::RenderGraphShaderStages> {
    match intent {
        RenderGraphResourceAccessIntent::SampledTexture { stages }
        | RenderGraphResourceAccessIntent::StorageTextureRead { stages }
        | RenderGraphResourceAccessIntent::StorageTextureWrite { stages }
        | RenderGraphResourceAccessIntent::UniformBuffer { stages }
        | RenderGraphResourceAccessIntent::StorageBufferRead { stages }
        | RenderGraphResourceAccessIntent::StorageBufferReadWrite { stages } => Some(stages),
        RenderGraphResourceAccessIntent::Legacy
        | RenderGraphResourceAccessIntent::ColorAttachment
        | RenderGraphResourceAccessIntent::DepthStencilAttachment
        | RenderGraphResourceAccessIntent::CopySource
        | RenderGraphResourceAccessIntent::CopyDestination
        | RenderGraphResourceAccessIntent::Indirect
        | RenderGraphResourceAccessIntent::Present
        | RenderGraphResourceAccessIntent::Readback => None,
    }
}

fn intent_requires_write(intent: RenderGraphResourceAccessIntent) -> bool {
    matches!(
        intent,
        RenderGraphResourceAccessIntent::StorageTextureWrite { .. }
            | RenderGraphResourceAccessIntent::ColorAttachment
            | RenderGraphResourceAccessIntent::DepthStencilAttachment
            | RenderGraphResourceAccessIntent::StorageBufferReadWrite { .. }
            | RenderGraphResourceAccessIntent::CopyDestination
    )
}

fn texture_usage_for_intent(intent: RenderGraphResourceAccessIntent) -> Option<TextureUsage> {
    match intent {
        RenderGraphResourceAccessIntent::SampledTexture { .. } => Some(TextureUsage::SAMPLED),
        RenderGraphResourceAccessIntent::StorageTextureRead { .. }
        | RenderGraphResourceAccessIntent::StorageTextureWrite { .. } => {
            Some(TextureUsage::STORAGE)
        }
        RenderGraphResourceAccessIntent::ColorAttachment
        | RenderGraphResourceAccessIntent::DepthStencilAttachment => {
            Some(TextureUsage::RENDER_ATTACHMENT)
        }
        RenderGraphResourceAccessIntent::Present => Some(TextureUsage::PRESENT),
        _ => None,
    }
}

fn buffer_usage_for_intent(intent: RenderGraphResourceAccessIntent) -> Option<BufferUsage> {
    match intent {
        RenderGraphResourceAccessIntent::UniformBuffer { .. } => Some(BufferUsage::UNIFORM),
        RenderGraphResourceAccessIntent::StorageBufferRead { .. }
        | RenderGraphResourceAccessIntent::StorageBufferReadWrite { .. } => {
            Some(BufferUsage::STORAGE)
        }
        RenderGraphResourceAccessIntent::Indirect => Some(BufferUsage::INDIRECT),
        RenderGraphResourceAccessIntent::Readback => Some(BufferUsage::STAGING_READ),
        _ => None,
    }
}

fn copy_usage_for_intent(
    intent: RenderGraphResourceAccessIntent,
) -> Option<(TextureUsage, BufferUsage)> {
    match intent {
        RenderGraphResourceAccessIntent::CopySource => {
            Some((TextureUsage::COPY_SRC, BufferUsage::COPY_SRC))
        }
        RenderGraphResourceAccessIntent::CopyDestination => {
            Some((TextureUsage::COPY_DST, BufferUsage::COPY_DST))
        }
        _ => None,
    }
}

fn buffer_usage(node: &ResourceNode) -> Option<BufferUsage> {
    match &node.desc {
        RenderGraphResourceDesc::Buffer(desc) => Some(desc.usage),
        RenderGraphResourceDesc::External => {
            node.external_buffer_desc.as_ref().map(|desc| desc.usage)
        }
        RenderGraphResourceDesc::Texture(_) => None,
    }
}

fn render_graph_access_kind(access: ResourceAccessKind) -> RenderGraphResourceAccessKind {
    match access {
        ResourceAccessKind::Read => RenderGraphResourceAccessKind::Read,
        ResourceAccessKind::Write => RenderGraphResourceAccessKind::Write,
    }
}

fn range_end_exceeds_u32(base: u32, count: Option<u32>, limit: u32) -> bool {
    if base >= limit {
        return true;
    }
    match count {
        Some(0) => true,
        Some(count) => base.checked_add(count).map_or(true, |end| end > limit),
        None => false,
    }
}

fn range_end_exceeds_u64(base: u64, count: Option<u64>, limit: u64) -> bool {
    if base >= limit {
        return true;
    }
    match count {
        Some(0) => true,
        Some(count) => base.checked_add(count).map_or(true, |end| end > limit),
        None => false,
    }
}
