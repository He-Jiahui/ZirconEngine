use zr_rhi::{
    RhiError, TextureDesc, TextureDimension, TextureFormat, TextureSampleType, TextureViewAspect,
    TextureViewDesc, TextureViewDimension,
};

/// Validates neutral view ranges before either backend creates a native view.
/// It intentionally has no WGPU objects so deterministic contract tests and
/// the production registry share exactly the same shape rules.
pub(crate) fn validate_texture_view_desc(
    texture: &TextureDesc,
    view: &TextureViewDesc,
) -> Result<(), RhiError> {
    validate_texture_view_format(texture, view)?;
    validate_texture_view_aspect(texture, view)?;
    let mip_level_count = selected_count(
        texture.mip_levels,
        view.base_mip_level,
        view.mip_level_count,
        view,
        "mip level",
    )?;
    let array_layer_count = selected_count(
        texture_view_layer_count(texture),
        view.base_array_layer,
        view.array_layer_count,
        view,
        "array layer",
    )?;

    match view.dimension {
        TextureViewDimension::D1 => {
            if texture.dimension != TextureDimension::D1 {
                return Err(view_error(view, "D1 views require a D1 texture"));
            }
            require_single_array_layer(array_layer_count, view)?;
        }
        TextureViewDimension::D2 => {
            if !matches!(
                texture.dimension,
                TextureDimension::D2 | TextureDimension::D2Array | TextureDimension::Cube
            ) {
                return Err(view_error(view, "D2 views require a D2-compatible texture"));
            }
            require_single_array_layer(array_layer_count, view)?;
        }
        TextureViewDimension::D2Array => {
            if !matches!(
                texture.dimension,
                TextureDimension::D2Array | TextureDimension::Cube
            ) {
                return Err(view_error(
                    view,
                    "D2Array views require a D2Array or cube-face texture",
                ));
            }
        }
        TextureViewDimension::D3 => {
            if texture.dimension != TextureDimension::D3 {
                return Err(view_error(view, "D3 views require a D3 texture"));
            }
            if view.base_array_layer != 0 {
                return Err(view_error(
                    view,
                    "D3 views must select base array layer zero",
                ));
            }
            require_single_array_layer(array_layer_count, view)?;
        }
        TextureViewDimension::Cube => {
            if texture.dimension != TextureDimension::Cube {
                return Err(view_error(view, "cube views require a cube texture"));
            }
            if view.base_array_layer % 6 != 0 {
                return Err(view_error(
                    view,
                    "cube views must start on a six-face boundary",
                ));
            }
            if array_layer_count != 6 {
                return Err(view_error(view, "cube views must select exactly six faces"));
            }
        }
        TextureViewDimension::CubeArray => {
            if texture.dimension != TextureDimension::Cube {
                return Err(view_error(view, "cube-array views require a cube texture"));
            }
            if view.base_array_layer % 6 != 0 {
                return Err(view_error(
                    view,
                    "cube-array views must start on a six-face boundary",
                ));
            }
            if array_layer_count % 6 != 0 {
                return Err(view_error(
                    view,
                    "cube-array views must select a multiple of six faces",
                ));
            }
        }
    }

    if texture.sample_count > 1 && !matches!(view.dimension, TextureViewDimension::D2) {
        return Err(view_error(
            view,
            "multisampled textures support only D2 texture views",
        ));
    }
    let _ = mip_level_count;
    Ok(())
}

pub(crate) const fn texture_sample_type(
    format: TextureFormat,
    aspect: TextureViewAspect,
) -> Option<TextureSampleType> {
    match aspect {
        TextureViewAspect::DepthOnly => {
            if format.is_depth() {
                Some(TextureSampleType::Depth)
            } else {
                None
            }
        }
        TextureViewAspect::StencilOnly => {
            if format.has_stencil() {
                Some(TextureSampleType::Uint)
            } else {
                None
            }
        }
        TextureViewAspect::All => match format {
            TextureFormat::Depth24Plus
            | TextureFormat::Depth24PlusStencil8
            | TextureFormat::Depth32Float => None,
            TextureFormat::R32Float | TextureFormat::Rgba32Float => {
                Some(TextureSampleType::Float { filterable: false })
            }
            TextureFormat::R8Unorm
            | TextureFormat::R16Float
            | TextureFormat::Rg16Float
            | TextureFormat::Rg11b10Ufloat
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Bgra8Unorm
            | TextureFormat::Bgra8UnormSrgb
            | TextureFormat::Rgba16Float => Some(TextureSampleType::Float { filterable: true }),
        },
    }
}

fn validate_texture_view_format(
    texture: &TextureDesc,
    view: &TextureViewDesc,
) -> Result<(), RhiError> {
    let Some(format) = view.format else {
        return Ok(());
    };
    if format == texture.format || texture.view_formats.contains(&format) {
        Ok(())
    } else {
        Err(view_error(
            view,
            &format!("view format {format:?} was not declared by parent texture"),
        ))
    }
}

fn validate_texture_view_aspect(
    texture: &TextureDesc,
    view: &TextureViewDesc,
) -> Result<(), RhiError> {
    match view.aspect {
        TextureViewAspect::All => Ok(()),
        TextureViewAspect::DepthOnly if texture.format.is_depth() => Ok(()),
        TextureViewAspect::DepthOnly => Err(view_error(
            view,
            "depth-only aspect requires a depth texture",
        )),
        TextureViewAspect::StencilOnly if texture.format.has_stencil() => Ok(()),
        TextureViewAspect::StencilOnly => Err(view_error(
            view,
            "stencil-only aspect requires a depth-stencil texture",
        )),
    }
}

fn selected_count(
    total: u32,
    base: u32,
    requested: Option<u32>,
    view: &TextureViewDesc,
    role: &str,
) -> Result<u32, RhiError> {
    if base >= total {
        return Err(view_error(
            view,
            &format!("base {role} {base} is outside range 0..{total}"),
        ));
    }
    let count = requested.unwrap_or(total - base);
    if count == 0 || count > total - base {
        return Err(view_error(
            view,
            &format!(
                "{role} count {count} exceeds remaining range {}",
                total - base
            ),
        ));
    }
    Ok(count)
}

fn require_single_array_layer(
    array_layer_count: u32,
    view: &TextureViewDesc,
) -> Result<(), RhiError> {
    if array_layer_count != 1 {
        return Err(view_error(
            view,
            "view dimension requires exactly one array layer",
        ));
    }
    Ok(())
}

const fn texture_view_layer_count(texture: &TextureDesc) -> u32 {
    match texture.dimension {
        TextureDimension::D1 | TextureDimension::D2 | TextureDimension::D3 => 1,
        TextureDimension::D2Array | TextureDimension::Cube => texture.depth,
    }
}

fn view_error(view: &TextureViewDesc, reason: &str) -> RhiError {
    RhiError::InvalidTextureViewDescriptor {
        label: view.label.clone(),
        reason: reason.to_string(),
    }
}
