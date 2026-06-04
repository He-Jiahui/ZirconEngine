use std::collections::BTreeSet;

use crate::rhi::{
    BindGroupDesc, BindGroupEntryResource, BindGroupLayoutDesc, BindGroupLayoutHandle,
    BindingResourceType, BufferDesc, BufferHandle, BufferUsage, RhiError, SamplerDesc,
    SamplerHandle, TextureDesc, TextureHandle, TextureUsage,
};

use super::resource_validation::{ensure_buffer_usage, ensure_texture_usage};

pub(super) trait BindGroupResourceLookup {
    fn layout_desc(&self, handle: BindGroupLayoutHandle) -> Result<&BindGroupLayoutDesc, RhiError>;
    fn buffer_desc(&self, handle: BufferHandle) -> Result<&BufferDesc, RhiError>;
    fn texture_desc(&self, handle: TextureHandle) -> Result<&TextureDesc, RhiError>;
    fn sampler_desc(&self, handle: SamplerHandle) -> Result<&SamplerDesc, RhiError>;
}

pub(super) fn validate_bind_group_desc(
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
            (BindingResourceType::UniformBuffer, BindGroupEntryResource::Buffer(handle)) => {
                let desc = resources.buffer_desc(handle)?;
                ensure_buffer_usage(handle.raw(), desc, BufferUsage::UNIFORM)?;
            }
            (BindingResourceType::StorageBuffer, BindGroupEntryResource::Buffer(handle)) => {
                let desc = resources.buffer_desc(handle)?;
                ensure_buffer_usage(handle.raw(), desc, BufferUsage::STORAGE)?;
            }
            (BindingResourceType::Texture, BindGroupEntryResource::Texture(handle)) => {
                let desc = resources.texture_desc(handle)?;
                ensure_texture_usage(handle.raw(), desc, TextureUsage::SAMPLED)?;
            }
            (BindingResourceType::StorageTexture, BindGroupEntryResource::Texture(handle)) => {
                let desc = resources.texture_desc(handle)?;
                ensure_texture_usage(handle.raw(), desc, TextureUsage::STORAGE)?;
            }
            (BindingResourceType::Sampler, BindGroupEntryResource::Sampler(handle)) => {
                resources.sampler_desc(handle)?;
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
