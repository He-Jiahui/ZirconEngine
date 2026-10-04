use super::{
    RenderDitherSettings, RenderPostProcessEffectStackSettings,
    RenderScreenSpaceReflectionSettings, RenderTonemapOperator, RenderTonemapSettings,
};

#[test]
fn extended_effect_stack_settings_enable_product_node_without_retired_fields() {
    let settings = RenderPostProcessEffectStackSettings {
        tonemap: RenderTonemapSettings {
            operator: RenderTonemapOperator::Aces,
            ..Default::default()
        },
        dither: RenderDitherSettings {
            intensity: 0.1,
            ..Default::default()
        },
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: 0.5,
            max_steps: 32,
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(settings.is_enabled());
}
