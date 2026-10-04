use super::{
    color_lookup_layout_binding_mode, color_lut_bake_dispatch_groups,
    color_lut_bake_workgroup_size, color_transform_requires_lut_bake,
};
use crate::core::framework::render::{
    RenderColorGradingSettings, RenderColorLookupSettings, RenderColorLookupTextureLayout,
    RenderPostProcessEffectStackSettings, RenderTonemapOperator, RenderTonemapSettings,
    COLOR_LUT_SIZE_DEFAULT,
};

#[test]
fn color_lut_bake_dispatch_covers_default_lut_volume() {
    assert_eq!(color_lut_bake_workgroup_size(), [4, 4, 4]);
    assert_eq!(color_lut_bake_dispatch_groups(), [8, 8, 8]);
    assert_eq!(COLOR_LUT_SIZE_DEFAULT, 32);
}

#[test]
fn color_lut_params_are_returned_as_pre_submit_uploads() {
    let source = include_str!("../mod.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("color LUT production source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(production.contains("WgpuBufferUploadBatch"));
}

#[test]
fn color_lut_bake_is_required_for_all_color_transform_sources() {
    assert!(!color_transform_requires_lut_bake(
        RenderColorGradingSettings::default(),
        RenderPostProcessEffectStackSettings::default(),
    ));
    assert!(color_transform_requires_lut_bake(
        RenderColorGradingSettings {
            exposure: 1.1,
            ..Default::default()
        },
        RenderPostProcessEffectStackSettings::default(),
    ));
    assert!(color_transform_requires_lut_bake(
        RenderColorGradingSettings::default(),
        RenderPostProcessEffectStackSettings {
            tonemap: RenderTonemapSettings {
                operator: RenderTonemapOperator::Aces,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    assert!(color_transform_requires_lut_bake(
        RenderColorGradingSettings::default(),
        RenderPostProcessEffectStackSettings {
            color_lookup: RenderColorLookupSettings {
                intensity: 0.5,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

#[test]
fn color_lookup_layout_binding_modes_match_shader_contract() {
    assert_eq!(
        color_lookup_layout_binding_mode(RenderColorLookupTextureLayout::Auto),
        1
    );
    assert_eq!(
        color_lookup_layout_binding_mode(RenderColorLookupTextureLayout::Texture2dStrip {
            size: 32
        }),
        2
    );
    assert_eq!(
        color_lookup_layout_binding_mode(RenderColorLookupTextureLayout::Texture3d { size: 32 }),
        3
    );
}
