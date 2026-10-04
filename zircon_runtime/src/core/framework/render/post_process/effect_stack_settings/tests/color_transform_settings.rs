use super::{
    RenderColorLookupSettings, RenderColorLookupTextureLayout, RenderTonemapOperator,
    RenderTonemapSettings,
};
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension, RenderImageFallbackKind,
    RenderImageUsage, RenderSamplerDescriptor, TextureMetadata,
};

#[test]
fn tonemap_settings_encode_renderer_upload_values() {
    let settings = RenderTonemapSettings {
        operator: RenderTonemapOperator::Aces,
        exposure_bias: -0.25,
        white_point: -1.0,
    };

    assert!(settings.is_enabled());
    assert_eq!(settings.render_operator_id(), 2);
    assert_eq!(settings.render_exposure_bias(), -0.25);
    assert_eq!(settings.render_white_point(), 0.001);
}

#[test]
fn color_lookup_intensity_requests_lut_even_without_texture_handle() {
    let settings = RenderColorLookupSettings {
        texture: None,
        texture_layout: RenderColorLookupTextureLayout::Auto,
        intensity: 0.25,
    };

    assert!(settings.is_enabled());
    assert_eq!(settings.render_intensity(), 0.25);
}

#[test]
fn color_lookup_settings_clamp_renderer_upload_intensity() {
    let settings = RenderColorLookupSettings {
        intensity: -0.5,
        ..Default::default()
    };

    assert!(!settings.is_enabled());
    assert_eq!(settings.render_intensity(), 0.0);
}

#[test]
fn color_lookup_texture_layout_accepts_current_2d_strip_contract() {
    let descriptor = texture_descriptor(33 * 33, 33, 1, RenderImageDimension::D2);
    let layout = RenderColorLookupTextureLayout::Texture2dStrip { size: 33 };

    assert_eq!(layout.label(), "texture-2d-strip");
    assert!(layout.matches_texture_2d_strip(&descriptor));
    assert!(layout.accepts_current_post_process_binding(&descriptor));
}

#[test]
fn color_lookup_texture_layout_rejects_zero_extent_metadata() {
    let descriptor = texture_descriptor(33 * 33, 33, 0, RenderImageDimension::D2);
    let layout = RenderColorLookupTextureLayout::Texture2dStrip { size: 33 };

    assert!(!layout.matches_texture_2d_strip(&descriptor));
    assert!(!layout.accepts_current_post_process_binding(&descriptor));
}

#[test]
fn color_lookup_texture_3d_layout_is_recognized_but_not_2d_bindable() {
    let descriptor = texture_descriptor(33, 33, 33, RenderImageDimension::D3);
    let layout = RenderColorLookupTextureLayout::Texture3d { size: 33 };

    assert_eq!(layout.label(), "texture-3d");
    assert!(layout.matches_texture_3d(&descriptor));
    assert!(!layout.accepts_current_post_process_binding(&descriptor));
}

fn texture_descriptor(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    dimension: RenderImageDimension,
) -> RenderImageDescriptor {
    RenderImageDescriptor {
        width,
        height,
        depth_or_array_layers,
        dimension,
        format: "rgba8unorm".to_string(),
        color_space: RenderImageColorSpace::Linear,
        metadata: TextureMetadata {
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        },
        sampler: RenderSamplerDescriptor::default(),
        usage: vec![RenderImageUsage::Sampled],
        asset_usage: Vec::new(),
        mip_count: 1,
        fallback: RenderImageFallbackKind::MissingImage,
    }
}
