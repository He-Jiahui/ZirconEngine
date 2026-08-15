use crate::asset::assets::{
    TextureUploadPlan, TextureUploadSupport, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT,
};
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDescriptor, RenderImageUsage, TextureMipPolicy,
};

pub(super) fn wgpu_texture_usages(
    descriptor: &RenderImageDescriptor,
    format: wgpu::TextureFormat,
    requires_upload_dst: bool,
) -> wgpu::TextureUsages {
    let mut usages = wgpu::TextureUsages::empty();
    for usage in &descriptor.usage {
        match usage {
            RenderImageUsage::Sampled => usages |= wgpu::TextureUsages::TEXTURE_BINDING,
            RenderImageUsage::Storage if supports_storage_binding_usage(format) => {
                usages |= wgpu::TextureUsages::STORAGE_BINDING;
            }
            RenderImageUsage::Storage => {}
            RenderImageUsage::RenderTarget if supports_render_attachment_usage(format) => {
                usages |= wgpu::TextureUsages::RENDER_ATTACHMENT;
            }
            RenderImageUsage::RenderTarget => {}
            RenderImageUsage::CopySrc => usages |= wgpu::TextureUsages::COPY_SRC,
            RenderImageUsage::CopyDst => usages |= wgpu::TextureUsages::COPY_DST,
        }
    }
    if descriptor.metadata.mip_policy == TextureMipPolicy::GenerateRuntime
        && descriptor.mip_count > 1
        && format == wgpu::TextureFormat::Rgba8Unorm
    {
        usages |= wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING;
    }
    if requires_upload_dst {
        usages |= wgpu::TextureUsages::COPY_DST;
    }
    usages
}

fn supports_render_attachment_usage(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb
    )
}

fn supports_storage_binding_usage(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::R8Unorm
            | wgpu::TextureFormat::R16Float
            | wgpu::TextureFormat::R32Float
            | wgpu::TextureFormat::Rg16Float
            | wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba16Float
            | wgpu::TextureFormat::Rgba32Float
    )
}

pub(crate) fn texture_upload_support_from_device(device: &wgpu::Device) -> TextureUploadSupport {
    let features = device.features();
    TextureUploadSupport {
        bc: features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC),
        bc_sliced_3d: features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC_SLICED_3D),
        etc2: features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2),
        astc_ldr: features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC),
        astc_sliced_3d: features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC_SLICED_3D),
    }
}

pub(super) fn rgba8_wgpu_format(format: &str) -> wgpu::TextureFormat {
    if format.trim().eq_ignore_ascii_case(RGBA8_UNORM_FORMAT) {
        wgpu::TextureFormat::Rgba8Unorm
    } else if format.trim().eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT) {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
}

pub(super) fn compressed_wgpu_format(
    plan: &TextureUploadPlan,
    color_space: RenderImageColorSpace,
) -> Option<wgpu::TextureFormat> {
    let srgb = color_space == RenderImageColorSpace::Srgb;
    match plan.format.as_str() {
        "dds/dxt1" => Some(if srgb {
            wgpu::TextureFormat::Bc1RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Bc1RgbaUnorm
        }),
        "dds/dxt3" => Some(if srgb {
            wgpu::TextureFormat::Bc2RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Bc2RgbaUnorm
        }),
        "dds/dxt5" => Some(if srgb {
            wgpu::TextureFormat::Bc3RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Bc3RgbaUnorm
        }),
        "dds/ati1" | "dds/bc4u" => Some(wgpu::TextureFormat::Bc4RUnorm),
        "dds/bc4s" => Some(wgpu::TextureFormat::Bc4RSnorm),
        "dds/ati2" | "dds/bc5u" => Some(wgpu::TextureFormat::Bc5RgUnorm),
        "dds/bc5s" => Some(wgpu::TextureFormat::Bc5RgSnorm),
        "dds/dxgi-71" => Some(wgpu::TextureFormat::Bc1RgbaUnorm),
        "dds/dxgi-72" => Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
        "dds/dxgi-74" => Some(wgpu::TextureFormat::Bc2RgbaUnorm),
        "dds/dxgi-75" => Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
        "dds/dxgi-77" => Some(wgpu::TextureFormat::Bc3RgbaUnorm),
        "dds/dxgi-78" => Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb),
        "dds/dxgi-80" => Some(wgpu::TextureFormat::Bc4RUnorm),
        "dds/dxgi-81" => Some(wgpu::TextureFormat::Bc4RSnorm),
        "dds/dxgi-83" => Some(wgpu::TextureFormat::Bc5RgUnorm),
        "dds/dxgi-84" => Some(wgpu::TextureFormat::Bc5RgSnorm),
        "dds/dxgi-95" => Some(wgpu::TextureFormat::Bc6hRgbUfloat),
        "dds/dxgi-96" => Some(wgpu::TextureFormat::Bc6hRgbFloat),
        "dds/dxgi-98" => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        "dds/dxgi-99" => Some(wgpu::TextureFormat::Bc7RgbaUnormSrgb),
        format if format.starts_with("ktx/gl-internal-0x") => ktx_gl_wgpu_format(format),
        format if format.starts_with("ktx2/vk-") => ktx2_vk_wgpu_format(format),
        format if format.starts_with("astc/") && plan.block_depth == 1 => {
            Some(wgpu::TextureFormat::Astc {
                block: astc_block(plan.block_width, plan.block_height)?,
                channel: if srgb {
                    wgpu::AstcChannel::UnormSrgb
                } else {
                    wgpu::AstcChannel::Unorm
                },
            })
        }
        _ => None,
    }
}

fn ktx_gl_wgpu_format(format: &str) -> Option<wgpu::TextureFormat> {
    let value = format.strip_prefix("ktx/gl-internal-0x")?;
    let gl_internal_format = u32::from_str_radix(value, 16).ok()?;
    match gl_internal_format {
        0x83f0 | 0x83f1 => Some(wgpu::TextureFormat::Bc1RgbaUnorm),
        0x8c4c | 0x8c4d => Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
        0x83f2 => Some(wgpu::TextureFormat::Bc2RgbaUnorm),
        0x8c4e => Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
        0x83f3 => Some(wgpu::TextureFormat::Bc3RgbaUnorm),
        0x8c4f => Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb),
        0x8dbb => Some(wgpu::TextureFormat::Bc4RUnorm),
        0x8dbc => Some(wgpu::TextureFormat::Bc4RSnorm),
        0x8dbd => Some(wgpu::TextureFormat::Bc5RgUnorm),
        0x8dbe => Some(wgpu::TextureFormat::Bc5RgSnorm),
        0x8e8f => Some(wgpu::TextureFormat::Bc6hRgbUfloat),
        0x8e8e => Some(wgpu::TextureFormat::Bc6hRgbFloat),
        0x8e8c => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        0x8e8d => Some(wgpu::TextureFormat::Bc7RgbaUnormSrgb),
        0x9274 => Some(wgpu::TextureFormat::Etc2Rgb8Unorm),
        0x9275 => Some(wgpu::TextureFormat::Etc2Rgb8UnormSrgb),
        0x9276 => Some(wgpu::TextureFormat::Etc2Rgb8A1Unorm),
        0x9277 => Some(wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb),
        0x9278 => Some(wgpu::TextureFormat::Etc2Rgba8Unorm),
        0x9279 => Some(wgpu::TextureFormat::Etc2Rgba8UnormSrgb),
        0x9270 => Some(wgpu::TextureFormat::EacR11Unorm),
        0x9271 => Some(wgpu::TextureFormat::EacR11Snorm),
        0x9272 => Some(wgpu::TextureFormat::EacRg11Unorm),
        0x9273 => Some(wgpu::TextureFormat::EacRg11Snorm),
        0x93b0..=0x93bd => Some(wgpu::TextureFormat::Astc {
            block: ktx_gl_astc_block(gl_internal_format)?,
            channel: wgpu::AstcChannel::Unorm,
        }),
        0x93d0..=0x93dd => Some(wgpu::TextureFormat::Astc {
            block: ktx_gl_astc_block(gl_internal_format)?,
            channel: wgpu::AstcChannel::UnormSrgb,
        }),
        _ => None,
    }
}

fn ktx_gl_astc_block(gl_internal_format: u32) -> Option<wgpu::AstcBlock> {
    let index = if (0x93b0..=0x93bd).contains(&gl_internal_format) {
        gl_internal_format - 0x93b0
    } else if (0x93d0..=0x93dd).contains(&gl_internal_format) {
        gl_internal_format - 0x93d0
    } else {
        return None;
    };
    astc_block_by_index(index)
}

fn ktx2_vk_wgpu_format(format: &str) -> Option<wgpu::TextureFormat> {
    let vk_format = format
        .split('/')
        .find_map(|part| part.strip_prefix("vk-"))
        .and_then(|value| value.parse::<u32>().ok())?;
    match vk_format {
        131 | 133 => Some(wgpu::TextureFormat::Bc1RgbaUnorm),
        132 | 134 => Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
        135 => Some(wgpu::TextureFormat::Bc2RgbaUnorm),
        136 => Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
        137 => Some(wgpu::TextureFormat::Bc3RgbaUnorm),
        138 => Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb),
        139 => Some(wgpu::TextureFormat::Bc4RUnorm),
        140 => Some(wgpu::TextureFormat::Bc4RSnorm),
        141 => Some(wgpu::TextureFormat::Bc5RgUnorm),
        142 => Some(wgpu::TextureFormat::Bc5RgSnorm),
        143 => Some(wgpu::TextureFormat::Bc6hRgbUfloat),
        144 => Some(wgpu::TextureFormat::Bc6hRgbFloat),
        145 => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        146 => Some(wgpu::TextureFormat::Bc7RgbaUnormSrgb),
        147 => Some(wgpu::TextureFormat::Etc2Rgb8Unorm),
        148 => Some(wgpu::TextureFormat::Etc2Rgb8UnormSrgb),
        149 => Some(wgpu::TextureFormat::Etc2Rgb8A1Unorm),
        150 => Some(wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb),
        151 => Some(wgpu::TextureFormat::Etc2Rgba8Unorm),
        152 => Some(wgpu::TextureFormat::Etc2Rgba8UnormSrgb),
        153 => Some(wgpu::TextureFormat::EacR11Unorm),
        154 => Some(wgpu::TextureFormat::EacR11Snorm),
        155 => Some(wgpu::TextureFormat::EacRg11Unorm),
        156 => Some(wgpu::TextureFormat::EacRg11Snorm),
        157..=184 => {
            let (block, channel) = ktx2_astc_format(vk_format)?;
            Some(wgpu::TextureFormat::Astc { block, channel })
        }
        _ => None,
    }
}

fn ktx2_astc_format(vk_format: u32) -> Option<(wgpu::AstcBlock, wgpu::AstcChannel)> {
    if !(157..=184).contains(&vk_format) {
        return None;
    }
    let block = astc_block_by_index((vk_format - 157) / 2)?;
    let channel = if vk_format % 2 == 0 {
        wgpu::AstcChannel::UnormSrgb
    } else {
        wgpu::AstcChannel::Unorm
    };
    Some((block, channel))
}

fn astc_block(width: u32, height: u32) -> Option<wgpu::AstcBlock> {
    match (width, height) {
        (4, 4) => Some(wgpu::AstcBlock::B4x4),
        (5, 4) => Some(wgpu::AstcBlock::B5x4),
        (5, 5) => Some(wgpu::AstcBlock::B5x5),
        (6, 5) => Some(wgpu::AstcBlock::B6x5),
        (6, 6) => Some(wgpu::AstcBlock::B6x6),
        (8, 5) => Some(wgpu::AstcBlock::B8x5),
        (8, 6) => Some(wgpu::AstcBlock::B8x6),
        (8, 8) => Some(wgpu::AstcBlock::B8x8),
        (10, 5) => Some(wgpu::AstcBlock::B10x5),
        (10, 6) => Some(wgpu::AstcBlock::B10x6),
        (10, 8) => Some(wgpu::AstcBlock::B10x8),
        (10, 10) => Some(wgpu::AstcBlock::B10x10),
        (12, 10) => Some(wgpu::AstcBlock::B12x10),
        (12, 12) => Some(wgpu::AstcBlock::B12x12),
        _ => None,
    }
}

fn astc_block_by_index(index: u32) -> Option<wgpu::AstcBlock> {
    Some(match index {
        0 => wgpu::AstcBlock::B4x4,
        1 => wgpu::AstcBlock::B5x4,
        2 => wgpu::AstcBlock::B5x5,
        3 => wgpu::AstcBlock::B6x5,
        4 => wgpu::AstcBlock::B6x6,
        5 => wgpu::AstcBlock::B8x5,
        6 => wgpu::AstcBlock::B8x6,
        7 => wgpu::AstcBlock::B8x8,
        8 => wgpu::AstcBlock::B10x5,
        9 => wgpu::AstcBlock::B10x6,
        10 => wgpu::AstcBlock::B10x8,
        11 => wgpu::AstcBlock::B10x10,
        12 => wgpu::AstcBlock::B12x10,
        13 => wgpu::AstcBlock::B12x12,
        _ => return None,
    })
}
