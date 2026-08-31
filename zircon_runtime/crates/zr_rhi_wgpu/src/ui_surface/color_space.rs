use std::sync::Arc;

pub(super) const UI_IMAGE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiTargetColorMode {
    LinearSrgb,
    ByteEncodedFallback,
}

pub(super) const fn target_color_mode(format: wgpu::TextureFormat) -> UiTargetColorMode {
    match format {
        wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb => {
            UiTargetColorMode::LinearSrgb
        }
        _ => UiTargetColorMode::ByteEncodedFallback,
    }
}

/// Converts straight-alpha sRGB bytes into sRGB-encoded, linear-premultiplied texels.
///
/// Hardware sRGB sampling decodes these texels before filtering, so transparent image edges retain
/// both correct premultiplied interpolation and linear-light composition.
pub(super) fn encode_linear_premultiplied_srgba8(mut rgba: Arc<[u8]>) -> Arc<[u8]> {
    for pixel in Arc::make_mut(&mut rgba).chunks_exact_mut(4) {
        let alpha = f32::from(pixel[3]) / 255.0;
        for channel in &mut pixel[..3] {
            let linear = srgb_to_linear(f32::from(*channel) / 255.0) * alpha;
            *channel = (linear_to_srgb(linear) * 255.0).round() as u8;
        }
    }
    rgba
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}
