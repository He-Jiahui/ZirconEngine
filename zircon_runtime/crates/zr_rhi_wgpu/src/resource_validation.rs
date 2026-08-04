use std::collections::BTreeSet;

use zr_rhi::{
    BindGroupLayoutDesc, BufferDesc, BufferUsage, RhiError, SamplerDesc, TextureDesc,
    TextureDimension, TextureUsage,
};

pub(super) fn ensure_buffer_usage(
    handle: u64,
    desc: &BufferDesc,
    required: BufferUsage,
) -> Result<(), RhiError> {
    if desc.usage.contains(required) {
        Ok(())
    } else {
        Err(RhiError::InvalidBufferUsage {
            buffer: handle,
            required,
            actual: desc.usage,
        })
    }
}

pub(super) fn ensure_texture_usage(
    handle: u64,
    desc: &TextureDesc,
    required: TextureUsage,
) -> Result<(), RhiError> {
    if desc.usage.contains(required) {
        Ok(())
    } else {
        Err(RhiError::InvalidTextureUsage {
            texture: handle,
            required,
            actual: desc.usage,
        })
    }
}

pub(super) fn texture_storage_size(desc: &TextureDesc) -> u64 {
    if desc.is_sparse_reserved() {
        return 0;
    }
    desc.checked_storage_size_bytes().unwrap_or(u64::MAX)
}

pub(super) fn validate_buffer_desc(desc: &BufferDesc) -> Result<(), RhiError> {
    if desc.size_bytes == 0 {
        return Err(RhiError::InvalidBufferDescriptor {
            label: desc.label.clone(),
            reason: "size_bytes must be greater than zero".to_string(),
        });
    }
    if desc.usage == BufferUsage::NONE {
        return Err(RhiError::InvalidBufferDescriptor {
            label: desc.label.clone(),
            reason: "usage must not be empty".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_texture_desc(
    desc: &TextureDesc,
    supports_sparse_texture: bool,
) -> Result<(), RhiError> {
    if desc.width == 0 || desc.height == 0 || desc.depth == 0 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "width, height, and depth must be greater than zero".to_string(),
        });
    }
    if desc.mip_levels == 0 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "mip_levels must be greater than zero".to_string(),
        });
    }
    if desc.sample_count == 0 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "sample_count must be greater than zero".to_string(),
        });
    }
    if desc.usage == TextureUsage::NONE {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "usage must not be empty".to_string(),
        });
    }
    match desc.dimension {
        TextureDimension::D1 => {
            if desc.height != 1 || desc.depth != 1 {
                return Err(RhiError::InvalidTextureDescriptor {
                    label: desc.label.clone(),
                    reason: "1D textures must declare height and depth as 1".to_string(),
                });
            }
        }
        TextureDimension::D2 => {
            if desc.depth != 1 {
                return Err(RhiError::InvalidTextureDescriptor {
                    label: desc.label.clone(),
                    reason: "2D textures must declare depth as 1".to_string(),
                });
            }
        }
        TextureDimension::D2Array | TextureDimension::D3 => {}
        TextureDimension::Cube => {
            if desc.width != desc.height {
                return Err(RhiError::InvalidTextureDescriptor {
                    label: desc.label.clone(),
                    reason: "cube textures must be square".to_string(),
                });
            }
            if desc.depth % 6 != 0 {
                return Err(RhiError::InvalidTextureDescriptor {
                    label: desc.label.clone(),
                    reason: "cube textures must declare depth as a multiple of six faces"
                        .to_string(),
                });
            }
        }
    }
    if desc.sample_count > 1 && desc.dimension != TextureDimension::D2 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "multisampling is only valid for 2D textures".to_string(),
        });
    }
    if desc.sample_count > 1 && desc.mip_levels > 1 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "multisampled textures cannot declare mip levels".to_string(),
        });
    }
    if !desc.mip_levels_fit_shape() {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "mip_levels exceeds the texture extent chain".to_string(),
        });
    }
    if desc.is_sparse_reserved() && !supports_sparse_texture {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "sparse texture residency requires backend sparse texture support".to_string(),
        });
    }
    let Some(storage_size) = desc.checked_storage_size_bytes() else {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "storage size overflows u64".to_string(),
        });
    };
    if !desc.is_sparse_reserved() && storage_size > usize::MAX as u64 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "storage size exceeds addressable memory".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_sampler_desc(desc: &SamplerDesc) -> Result<(), RhiError> {
    if !desc.lod_min_clamp.is_finite() || !desc.lod_max_clamp.is_finite() {
        return Err(RhiError::InvalidSamplerDescriptor {
            label: desc.label.clone(),
            reason: "lod clamps must be finite".to_string(),
        });
    }
    if desc.lod_min_clamp > desc.lod_max_clamp {
        return Err(RhiError::InvalidSamplerDescriptor {
            label: desc.label.clone(),
            reason: "lod_min_clamp must be less than or equal to lod_max_clamp".to_string(),
        });
    }
    if !(1..=16).contains(&desc.anisotropy_clamp) {
        return Err(RhiError::InvalidSamplerDescriptor {
            label: desc.label.clone(),
            reason: "anisotropy_clamp must be in the range 1..=16".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_bind_group_layout_desc(desc: &BindGroupLayoutDesc) -> Result<(), RhiError> {
    if desc.entries.is_empty() {
        return Err(RhiError::InvalidBindGroupLayoutDescriptor {
            label: desc.label.clone(),
            reason: "entries must not be empty".to_string(),
        });
    }

    let mut seen_bindings = BTreeSet::new();
    for entry in &desc.entries {
        if !seen_bindings.insert(entry.binding) {
            return Err(RhiError::InvalidBindGroupLayoutDescriptor {
                label: desc.label.clone(),
                reason: format!("binding {} is duplicated", entry.binding),
            });
        }
        if entry.visibility.is_empty() {
            return Err(RhiError::InvalidBindGroupLayoutDescriptor {
                label: desc.label.clone(),
                reason: format!("binding {} has no shader-stage visibility", entry.binding),
            });
        }
        let mut seen_stages = BTreeSet::new();
        for stage in &entry.visibility {
            if !seen_stages.insert(*stage as u8) {
                return Err(RhiError::InvalidBindGroupLayoutDescriptor {
                    label: desc.label.clone(),
                    reason: format!("binding {} repeats shader-stage visibility", entry.binding),
                });
            }
        }
    }

    Ok(())
}
