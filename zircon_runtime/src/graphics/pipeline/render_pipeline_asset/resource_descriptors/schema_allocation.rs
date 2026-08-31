use crate::core::framework::render::{RenderFrameExtract, RenderViewFamilyPipeline};
use crate::graphics::{
    RenderResourceSchema, RenderTextureExtentPolicy, RenderTextureExtentReference,
    RenderTextureExtentRounding,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureUsage};

pub(super) fn texture_desc_from_schema(
    name: &str,
    schema: RenderResourceSchema,
    extract: &RenderFrameExtract,
) -> Result<TextureDesc, String> {
    let schema = match schema {
        RenderResourceSchema::Texture(schema) => schema,
        RenderResourceSchema::Buffer(_) => {
            return Err(format!(
                "resource `{name}` texture allocation requires RenderResourceSchema::Texture"
            ));
        }
    };
    let (width, height, depth) = match schema.extent {
        RenderTextureExtentPolicy::Render => {
            let extent = extract
                .view
                .view_family_pipeline()
                .resolution()
                .primary_allocation_extent();
            (extent.x.max(1), extent.y.max(1), 1)
        }
        RenderTextureExtentPolicy::View => {
            let extent = extract
                .view
                .view_family_pipeline()
                .resolution()
                .display_extent();
            (extent.x.max(1), extent.y.max(1), 1)
        }
        RenderTextureExtentPolicy::Relative {
            reference,
            numerator,
            denominator,
            rounding,
        } => {
            let extent = match reference {
                RenderTextureExtentReference::Render => extract
                    .view
                    .view_family_pipeline()
                    .resolution()
                    .primary_allocation_extent(),
                RenderTextureExtentReference::View => extract
                    .view
                    .view_family_pipeline()
                    .resolution()
                    .display_extent(),
            };
            let width =
                resolve_relative_extent_axis(extent.x.max(1), numerator, denominator, rounding)
                    .map_err(|reason| format!("resource `{name}` relative width {reason}"))?;
            let height =
                resolve_relative_extent_axis(extent.y.max(1), numerator, denominator, rounding)
                    .map_err(|reason| format!("resource `{name}` relative height {reason}"))?;
            (width, height, 1)
        }
        RenderTextureExtentPolicy::Fixed {
            width,
            height,
            depth_or_array_layers,
        } => (width, height, depth_or_array_layers),
    };
    if width == 0 || height == 0 || depth == 0 {
        return Err(format!(
            "resource `{name}` schema declares a zero texture extent {width}x{height}x{depth}"
        ));
    }
    if schema.mip_levels == 0 || schema.sample_count == 0 {
        return Err(format!(
            "resource `{name}` schema must declare non-zero mip levels and sample count"
        ));
    }
    if schema.usage == TextureUsage::NONE {
        return Err(format!(
            "resource `{name}` texture schema usage must not be empty"
        ));
    }
    let desc = TextureDesc::new(name, width, height, schema.format, schema.usage)
        .with_dimension(schema.dimension)
        .with_depth(depth)
        .with_mip_levels(schema.mip_levels)
        .with_sample_count(schema.sample_count);
    if !desc.mip_levels_fit_shape() {
        return Err(format!(
            "resource `{name}` schema mip level count {} exceeds its {}x{}x{} extent",
            desc.mip_levels, desc.width, desc.height, desc.depth
        ));
    }
    Ok(desc)
}

pub(super) fn resolve_relative_extent_axis(
    reference_extent: u32,
    numerator: u32,
    denominator: u32,
    rounding: RenderTextureExtentRounding,
) -> Result<u32, String> {
    if numerator == 0 || denominator == 0 {
        return Err("requires a non-zero numerator and denominator".to_string());
    }
    let product = u64::from(reference_extent) * u64::from(numerator);
    let denominator = u64::from(denominator);
    let quotient = product / denominator;
    let scaled = match rounding {
        RenderTextureExtentRounding::Floor => quotient,
        RenderTextureExtentRounding::Ceil => {
            quotient + if product % denominator == 0 { 0 } else { 1 }
        }
    }
    .max(1);
    u32::try_from(scaled).map_err(|_| "exceeds the supported u32 texture extent".to_string())
}

pub(super) fn buffer_desc_from_schema(
    name: &str,
    schema: RenderResourceSchema,
    minimum_size_bytes: Option<u64>,
) -> Result<BufferDesc, String> {
    let schema = match schema {
        RenderResourceSchema::Texture(_) => {
            return Err(format!(
                "resource `{name}` buffer allocation requires RenderResourceSchema::Buffer"
            ));
        }
        RenderResourceSchema::Buffer(schema) => schema,
    };
    if schema.size_bytes == 0 {
        return Err(format!(
            "resource `{name}` buffer schema must declare a non-zero byte size"
        ));
    }
    if schema.usage == BufferUsage::NONE {
        return Err(format!(
            "resource `{name}` buffer schema usage must not be empty"
        ));
    }
    let size_bytes = schema.size_bytes.max(minimum_size_bytes.unwrap_or(0));
    Ok(BufferDesc::new(name, size_bytes, schema.usage))
}
