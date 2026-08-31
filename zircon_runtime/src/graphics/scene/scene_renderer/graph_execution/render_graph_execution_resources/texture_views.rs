use crate::render_graph::{RenderGraphTextureAspect, RenderGraphTextureSubresourceRange};
use crate::rhi::{TextureDesc, TextureDimension};

pub(super) fn texture_mip_view_descriptor(mip_level: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        ..Default::default()
    }
}

pub(super) fn texture_full_mip_view_descriptor(
    mip_level_count: u32,
) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        base_mip_level: 0,
        mip_level_count: Some(mip_level_count),
        ..Default::default()
    }
}

pub(super) fn texture_subresource_view_descriptor(
    range: RenderGraphTextureSubresourceRange,
) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        base_mip_level: range.base_mip_level,
        mip_level_count: range.mip_level_count,
        base_array_layer: range.base_array_layer,
        array_layer_count: range.array_layer_count,
        aspect: match range.aspect {
            RenderGraphTextureAspect::All => wgpu::TextureAspect::All,
            RenderGraphTextureAspect::Color => wgpu::TextureAspect::All,
            RenderGraphTextureAspect::Depth => wgpu::TextureAspect::DepthOnly,
            RenderGraphTextureAspect::Stencil => wgpu::TextureAspect::StencilOnly,
        },
        ..Default::default()
    }
}

pub(super) fn validate_texture_view_descriptor(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    validate_owned_texture_view_format(name, texture_desc, view_desc)?;
    validate_owned_texture_view_usage(name, texture_desc, view_desc)?;
    validate_owned_texture_view_dimension(name, texture_desc, view_desc)?;
    validate_owned_texture_view_mip_range(name, texture_desc, view_desc)?;
    validate_owned_texture_view_array_range(name, texture_desc, view_desc)?;
    Ok(())
}

fn validate_owned_texture_view_format(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let expected_format = super::super::materialization::wgpu_texture_format(texture_desc.format);
    if let Some(view_format) = view_desc.format.filter(|format| *format != expected_format) {
        return Err(format!(
            "render graph execution texture resource `{name}` view format {:?} does not match texture format {:?}",
            view_format, expected_format
        ));
    }
    Ok(())
}

fn validate_owned_texture_view_usage(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let Some(requested_usage) = view_desc.usage else {
        return Ok(());
    };
    let texture_usages = super::super::materialization::wgpu_texture_usages(
        texture_desc.format,
        texture_desc.usage,
    )?;
    if !texture_usages.contains(requested_usage) {
        return Err(format!(
            "render graph execution texture resource `{name}` view usage {:?} is not allowed by texture usages {:?}",
            requested_usage, texture_usages
        ));
    }
    Ok(())
}

fn validate_owned_texture_view_dimension(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let Some(view_dimension) = view_desc.dimension else {
        return Ok(());
    };
    if texture_view_dimension_allowed(texture_desc.dimension, view_dimension) {
        return Ok(());
    }
    Err(format!(
        "render graph execution texture resource `{name}` view dimension {:?} is not compatible with texture dimension {:?}",
        view_dimension, texture_desc.dimension
    ))
}

fn texture_view_dimension_allowed(
    texture_dimension: TextureDimension,
    view_dimension: wgpu::TextureViewDimension,
) -> bool {
    match texture_dimension {
        TextureDimension::D1 => matches!(view_dimension, wgpu::TextureViewDimension::D1),
        TextureDimension::D2 => matches!(view_dimension, wgpu::TextureViewDimension::D2),
        TextureDimension::D2Array => matches!(
            view_dimension,
            wgpu::TextureViewDimension::D2 | wgpu::TextureViewDimension::D2Array
        ),
        TextureDimension::D3 => matches!(view_dimension, wgpu::TextureViewDimension::D3),
        TextureDimension::Cube => matches!(
            view_dimension,
            wgpu::TextureViewDimension::D2
                | wgpu::TextureViewDimension::D2Array
                | wgpu::TextureViewDimension::Cube
                | wgpu::TextureViewDimension::CubeArray
        ),
    }
}

fn validate_owned_texture_view_mip_range(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let base = view_desc.base_mip_level;
    let count = view_desc
        .mip_level_count
        .unwrap_or_else(|| texture_desc.mip_levels.saturating_sub(base));
    if count == 0 || base.saturating_add(count) > texture_desc.mip_levels {
        return Err(format!(
            "render graph execution texture resource `{name}` view mip range [{base}..{}) is outside mip_levels {}",
            base.saturating_add(count),
            texture_desc.mip_levels
        ));
    }
    Ok(())
}

fn validate_owned_texture_view_array_range(
    name: &str,
    texture_desc: &TextureDesc,
    view_desc: &wgpu::TextureViewDescriptor<'_>,
) -> Result<(), String> {
    let base = view_desc.base_array_layer;
    let count = view_desc
        .array_layer_count
        .unwrap_or_else(|| texture_desc.depth.saturating_sub(base));
    if count == 0 || base.saturating_add(count) > texture_desc.depth {
        return Err(format!(
            "render graph execution texture resource `{name}` view array range [{base}..{}) is outside depth/array_layers {}",
            base.saturating_add(count),
            texture_desc.depth
        ));
    }
    Ok(())
}
