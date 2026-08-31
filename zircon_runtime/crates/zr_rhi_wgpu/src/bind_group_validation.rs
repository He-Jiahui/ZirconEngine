use std::collections::BTreeSet;

use zr_rhi::{
    BindGroupBufferBinding, BindGroupDesc, BindGroupEntryResource, BindGroupLayoutDesc,
    BindGroupLayoutHandle, BindingResourceType, BufferDesc, BufferHandle, BufferUsage,
    RenderDeviceLimits, RhiError, SamplerBindingType, SamplerDesc, SamplerHandle,
    StorageTextureBindingDesc, TextureDesc, TextureHandle, TextureSampleType, TextureUsage,
    TextureViewDesc, TextureViewHandle,
};

use super::resource_validation::{ensure_buffer_usage, ensure_texture_usage};
use super::texture_view::texture_sample_type;

pub(crate) trait BindGroupResourceLookup {
    fn layout_desc(&self, handle: BindGroupLayoutHandle) -> Result<&BindGroupLayoutDesc, RhiError>;
    fn buffer_desc(&self, handle: BufferHandle) -> Result<&BufferDesc, RhiError>;
    fn texture_desc(&self, handle: TextureHandle) -> Result<&TextureDesc, RhiError>;
    fn texture_view_desc(&self, handle: TextureViewHandle) -> Result<&TextureViewDesc, RhiError>;
    fn sampler_desc(&self, handle: SamplerHandle) -> Result<&SamplerDesc, RhiError>;
}

pub(crate) fn validate_bind_group_desc(
    resources: &impl BindGroupResourceLookup,
    desc: &BindGroupDesc,
) -> Result<(), RhiError> {
    let layout = resources.layout_desc(desc.layout)?;

    if desc.entries.len() != layout.entries.len() {
        return Err(RhiError::InvalidBindGroupDescriptor {
            label: desc.label.clone(),
            reason: format!(
                "entry count {} does not match layout entry count {}",
                desc.entries.len(),
                layout.entries.len()
            ),
        });
    }

    let mut seen_bindings = BTreeSet::new();
    for entry in &desc.entries {
        if !seen_bindings.insert(entry.binding) {
            return Err(RhiError::InvalidBindGroupDescriptor {
                label: desc.label.clone(),
                reason: format!("binding {} is duplicated", entry.binding),
            });
        }
    }

    for layout_entry in &layout.entries {
        let Some(entry) = desc
            .entries
            .iter()
            .find(|entry| entry.binding == layout_entry.binding)
        else {
            return Err(RhiError::InvalidBindGroupDescriptor {
                label: desc.label.clone(),
                reason: format!("binding {} is missing", layout_entry.binding),
            });
        };

        match (layout_entry.resource_type, entry.resource) {
            (BindingResourceType::UniformBuffer, BindGroupEntryResource::Buffer(binding)) => {
                validate_buffer_binding(
                    resources,
                    desc,
                    layout_entry,
                    binding,
                    BufferUsage::UNIFORM,
                )?;
            }
            (BindingResourceType::StorageBuffer, BindGroupEntryResource::Buffer(binding)) => {
                validate_buffer_binding(
                    resources,
                    desc,
                    layout_entry,
                    binding,
                    BufferUsage::STORAGE,
                )?;
            }
            (
                BindingResourceType::SampledTexture {
                    sample_type,
                    view_dimension,
                    multisampled,
                },
                BindGroupEntryResource::TextureView(handle),
            ) => {
                let view = resources.texture_view_desc(handle)?;
                let texture = resources.texture_desc(view.texture)?;
                ensure_texture_usage(view.texture.diagnostic_id(), texture, TextureUsage::SAMPLED)?;
                validate_sampled_texture_binding(
                    desc,
                    layout_entry.binding,
                    texture,
                    view,
                    sample_type,
                    view_dimension,
                    multisampled,
                )?;
            }
            (
                BindingResourceType::StorageTexture(storage),
                BindGroupEntryResource::TextureView(handle),
            ) => {
                let view = resources.texture_view_desc(handle)?;
                let texture = resources.texture_desc(view.texture)?;
                validate_storage_texture_binding(
                    desc,
                    layout_entry.binding,
                    texture,
                    view,
                    storage,
                )?;
            }
            (
                BindingResourceType::Sampler(binding_type),
                BindGroupEntryResource::Sampler(handle),
            ) => {
                validate_sampler_binding(
                    desc,
                    layout_entry.binding,
                    resources.sampler_desc(handle)?,
                    binding_type,
                )?;
            }
            (expected, actual) => {
                return Err(RhiError::InvalidBindGroupDescriptor {
                    label: desc.label.clone(),
                    reason: format!(
                        "binding {} expects {:?}, got {:?}",
                        layout_entry.binding, expected, actual
                    ),
                });
            }
        }
    }

    Ok(())
}

pub(crate) fn validate_bind_group_dynamic_offsets(
    resources: &impl BindGroupResourceLookup,
    bind_group: zr_rhi::BindGroupHandle,
    desc: &BindGroupDesc,
    dynamic_offsets: &[u32],
    limits: &RenderDeviceLimits,
) -> Result<(), RhiError> {
    let layout = resources.layout_desc(desc.layout)?;
    let mut dynamic_entries = layout
        .entries
        .iter()
        .filter(|entry| entry.has_dynamic_offset)
        .collect::<Vec<_>>();
    dynamic_entries.sort_by_key(|entry| entry.binding);
    if dynamic_offsets.len() != dynamic_entries.len() {
        return Err(RhiError::InvalidBindGroupUsage {
            reason: format!(
                "bind group `{}` requires {} dynamic offsets, got {}",
                bind_group.diagnostic_id(),
                dynamic_entries.len(),
                dynamic_offsets.len(),
            ),
        });
    }

    for (layout_entry, dynamic_offset) in dynamic_entries.into_iter().zip(dynamic_offsets) {
        let binding = desc
            .entries
            .iter()
            .find(|entry| entry.binding == layout_entry.binding)
            .and_then(|entry| match entry.resource {
                BindGroupEntryResource::Buffer(binding) => Some(binding),
                BindGroupEntryResource::TextureView(_) | BindGroupEntryResource::Sampler(_) => None,
            })
            .ok_or_else(|| {
                bind_group_error(
                    desc,
                    format!(
                        "binding {} must be a buffer for dynamic offsets",
                        layout_entry.binding
                    ),
                )
            })?;
        let alignment = dynamic_offset_alignment(limits, layout_entry.resource_type)?;
        if *dynamic_offset % alignment != 0 {
            return Err(RhiError::InvalidBindGroupUsage {
                reason: format!(
                    "bind group `{}` dynamic offset {} for layout binding {} must be aligned to {}",
                    bind_group.diagnostic_id(),
                    dynamic_offset,
                    layout_entry.binding,
                    alignment,
                ),
            });
        }
        let buffer = resources.buffer_desc(binding.buffer)?;
        let size = resolved_buffer_binding_size(desc, layout_entry.binding, binding, buffer)?;
        let start = binding
            .offset
            .checked_add(u64::from(*dynamic_offset))
            .ok_or_else(|| dynamic_offset_range_error(bind_group, layout_entry.binding))?;
        let end = start
            .checked_add(size)
            .ok_or_else(|| dynamic_offset_range_error(bind_group, layout_entry.binding))?;
        if end > buffer.size_bytes {
            return Err(dynamic_offset_range_error(bind_group, layout_entry.binding));
        }
    }

    Ok(())
}

fn validate_buffer_binding(
    resources: &impl BindGroupResourceLookup,
    bind_group: &BindGroupDesc,
    layout_entry: &zr_rhi::BindGroupLayoutEntryDesc,
    binding: BindGroupBufferBinding,
    required_usage: BufferUsage,
) -> Result<(), RhiError> {
    let buffer = resources.buffer_desc(binding.buffer)?;
    ensure_buffer_usage(binding.buffer.diagnostic_id(), buffer, required_usage)?;
    let size = resolved_buffer_binding_size(bind_group, layout_entry.binding, binding, buffer)?;
    if layout_entry.has_dynamic_offset && binding.size.is_none() {
        return Err(bind_group_error(
            bind_group,
            format!(
                "binding {} with a dynamic offset requires an explicit buffer size",
                layout_entry.binding
            ),
        ));
    }
    if let Some(minimum) = layout_entry.min_binding_size {
        if size < minimum {
            return Err(bind_group_error(
                bind_group,
                format!(
                    "binding {} binds {} bytes, below layout minimum {}",
                    layout_entry.binding, size, minimum
                ),
            ));
        }
    }
    Ok(())
}

fn resolved_buffer_binding_size(
    bind_group: &BindGroupDesc,
    binding_index: u32,
    binding: BindGroupBufferBinding,
    buffer: &BufferDesc,
) -> Result<u64, RhiError> {
    let Some(available) = buffer.size_bytes.checked_sub(binding.offset) else {
        return Err(RhiError::BufferBindingOutOfRange {
            buffer: binding.buffer.diagnostic_id(),
            offset: binding.offset,
            size: binding.size.unwrap_or(0),
        });
    };
    let size = binding.size.unwrap_or(available);
    if size == 0 || size > available {
        return Err(RhiError::BufferBindingOutOfRange {
            buffer: binding.buffer.diagnostic_id(),
            offset: binding.offset,
            size,
        });
    }
    if binding.size.is_none() && available == 0 {
        return Err(bind_group_error(
            bind_group,
            format!("binding {binding_index} resolves to an empty buffer range"),
        ));
    }
    Ok(size)
}

fn dynamic_offset_alignment(
    limits: &RenderDeviceLimits,
    resource_type: BindingResourceType,
) -> Result<u32, RhiError> {
    let alignment = match resource_type {
        BindingResourceType::UniformBuffer => limits.min_uniform_buffer_offset_alignment,
        BindingResourceType::StorageBuffer => limits.min_storage_buffer_offset_alignment,
        BindingResourceType::SampledTexture { .. }
        | BindingResourceType::StorageTexture(_)
        | BindingResourceType::Sampler(_) => {
            return Err(RhiError::InvalidBindGroupUsage {
                reason: "dynamic offset layout entry is not a buffer binding".to_string(),
            });
        }
    };
    if alignment == 0 {
        Err(RhiError::InvalidBindGroupUsage {
            reason: "device reports zero dynamic-buffer offset alignment".to_string(),
        })
    } else {
        Ok(alignment)
    }
}

fn dynamic_offset_range_error(bind_group: zr_rhi::BindGroupHandle, binding: u32) -> RhiError {
    RhiError::InvalidBindGroupUsage {
        reason: format!(
            "bind group `{}` dynamic offset range for layout binding {} exceeds its buffer",
            bind_group.diagnostic_id(),
            binding
        ),
    }
}

fn validate_storage_texture_binding(
    bind_group: &BindGroupDesc,
    binding: u32,
    texture: &TextureDesc,
    view: &TextureViewDesc,
    storage: StorageTextureBindingDesc,
) -> Result<(), RhiError> {
    ensure_texture_usage(view.texture.diagnostic_id(), texture, TextureUsage::STORAGE)?;
    if view.dimension != storage.view_dimension {
        return Err(bind_group_error(
            bind_group,
            format!(
                "binding {binding} requires {:?} storage texture view, got {:?}",
                storage.view_dimension, view.dimension
            ),
        ));
    }
    if texture.sample_count != 1 {
        return Err(bind_group_error(
            bind_group,
            format!(
                "binding {binding} requires a single-sampled storage texture, got {} samples",
                texture.sample_count
            ),
        ));
    }
    let view_format = view.resolved_format(texture.format);
    if view_format != storage.format {
        return Err(bind_group_error(
            bind_group,
            format!(
                "binding {binding} requires {:?} storage texture format, got {:?}",
                storage.format, view_format
            ),
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_sampled_texture_binding(
    bind_group: &BindGroupDesc,
    binding: u32,
    texture: &TextureDesc,
    view: &TextureViewDesc,
    sample_type: TextureSampleType,
    view_dimension: zr_rhi::TextureViewDimension,
    multisampled: bool,
) -> Result<(), RhiError> {
    if view.dimension != view_dimension {
        return Err(bind_group_error(
            bind_group,
            format!(
                "binding {binding} requires {:?} texture view, got {:?}",
                view_dimension, view.dimension
            ),
        ));
    }
    if (texture.sample_count > 1) != multisampled {
        return Err(bind_group_error(
            bind_group,
            format!(
                "binding {binding} multisampled flag does not match texture sample count {}",
                texture.sample_count
            ),
        ));
    }

    let Some(actual_sample_type) =
        texture_sample_type(view.resolved_format(texture.format), view.aspect)
    else {
        return Err(bind_group_error(
            bind_group,
            format!("binding {binding} requires a shader-sampleable texture view aspect"),
        ));
    };
    match (sample_type, actual_sample_type) {
        (
            TextureSampleType::Float { filterable: true },
            TextureSampleType::Float { filterable: false },
        ) => Err(bind_group_error(
            bind_group,
            format!("binding {binding} requires a filterable float texture view"),
        )),
        (TextureSampleType::Float { filterable: false }, TextureSampleType::Float { .. }) => Ok(()),
        (expected, actual) if expected != actual => Err(bind_group_error(
            bind_group,
            format!(
                "binding {binding} requires {:?} texture samples, got {:?}",
                expected, actual
            ),
        )),
        _ => Ok(()),
    }
}

fn validate_sampler_binding(
    bind_group: &BindGroupDesc,
    binding: u32,
    sampler: &SamplerDesc,
    binding_type: SamplerBindingType,
) -> Result<(), RhiError> {
    let compatible = match binding_type {
        SamplerBindingType::Filtering => sampler.compare.is_none(),
        SamplerBindingType::NonFiltering => {
            sampler.compare.is_none()
                && matches!(sampler.mag_filter, zr_rhi::FilterMode::Nearest)
                && matches!(sampler.min_filter, zr_rhi::FilterMode::Nearest)
                && matches!(sampler.mipmap_filter, zr_rhi::MipmapFilterMode::Nearest)
                && sampler.anisotropy_clamp == 1
        }
        SamplerBindingType::Comparison => sampler.compare.is_some(),
    };
    if compatible {
        Ok(())
    } else {
        Err(bind_group_error(
            bind_group,
            format!("binding {binding} is incompatible with {binding_type:?} sampler layout"),
        ))
    }
}

fn bind_group_error(bind_group: &BindGroupDesc, reason: String) -> RhiError {
    RhiError::InvalidBindGroupDescriptor {
        label: bind_group.label.clone(),
        reason,
    }
}
