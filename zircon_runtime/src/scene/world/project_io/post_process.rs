use crate::asset::assets::{
    SceneBloomSettingsAsset, SceneChromaticAberrationSettingsAsset, SceneColorGradingSettingsAsset,
    SceneDitherSettingsAsset, SceneFilmGrainSettingsAsset, SceneFogSettingsAsset,
    ScenePostProcessEffectStackAsset, ScenePostProcessSettingsAsset, ScenePostProcessVolumeAsset,
    ScenePostProcessVolumeProfileAsset, SceneTonemapOperatorAsset, SceneTonemapSettingsAsset,
    SceneVignetteSettingsAsset,
};
use crate::core::framework::render::{
    RenderBloomSettings, RenderChromaticAberrationSettings, RenderColorGradingSettings,
    RenderDitherSettings, RenderFilmGrainSettings, RenderFogSettings,
    RenderPostProcessEffectStackSettings, RenderPostProcessVolumeProfile, RenderTonemapOperator,
    RenderTonemapSettings, RenderVignetteSettings,
};
use crate::scene::components::{PostProcessSettingsComponent, PostProcessVolumeComponent};
pub(super) fn post_process_settings_from_asset(
    settings: ScenePostProcessSettingsAsset,
) -> PostProcessSettingsComponent {
    PostProcessSettingsComponent::from_parts(
        bloom_from_asset(settings.bloom),
        color_grading_from_asset(settings.color_grading),
        effect_stack_from_asset(settings.effect_stack),
    )
}

pub(super) fn post_process_settings_to_asset(
    settings: PostProcessSettingsComponent,
) -> ScenePostProcessSettingsAsset {
    ScenePostProcessSettingsAsset {
        bloom: bloom_to_asset(settings.bloom),
        color_grading: color_grading_to_asset(settings.color_grading),
        effect_stack: effect_stack_to_asset(settings.effect_stack),
    }
}

pub(super) fn post_process_volume_from_asset(
    volume: ScenePostProcessVolumeAsset,
) -> PostProcessVolumeComponent {
    PostProcessVolumeComponent {
        active: volume.active,
        is_global: volume.is_global,
        priority: volume.priority,
        weight: volume.weight,
        blend_distance: volume.blend_distance,
        profile: volume_profile_from_asset(volume.profile),
    }
}

pub(super) fn post_process_volume_to_asset(
    volume: PostProcessVolumeComponent,
) -> ScenePostProcessVolumeAsset {
    ScenePostProcessVolumeAsset {
        active: volume.active,
        is_global: volume.is_global,
        priority: volume.priority,
        weight: volume.weight,
        blend_distance: volume.blend_distance,
        profile: volume_profile_to_asset(volume.profile),
    }
}

fn volume_profile_from_asset(
    profile: ScenePostProcessVolumeProfileAsset,
) -> RenderPostProcessVolumeProfile {
    RenderPostProcessVolumeProfile {
        bloom: profile.bloom.map(bloom_from_asset),
        color_grading: profile.color_grading.map(color_grading_from_asset),
        effect_stack: profile.effect_stack.map(effect_stack_from_asset),
    }
}

fn volume_profile_to_asset(
    profile: RenderPostProcessVolumeProfile,
) -> ScenePostProcessVolumeProfileAsset {
    ScenePostProcessVolumeProfileAsset {
        bloom: profile.bloom.map(bloom_to_asset),
        color_grading: profile.color_grading.map(color_grading_to_asset),
        effect_stack: profile.effect_stack.map(effect_stack_to_asset),
    }
}

fn bloom_from_asset(settings: SceneBloomSettingsAsset) -> RenderBloomSettings {
    RenderBloomSettings {
        threshold: settings.threshold,
        intensity: settings.intensity,
        radius: settings.radius,
    }
}

fn bloom_to_asset(settings: RenderBloomSettings) -> SceneBloomSettingsAsset {
    SceneBloomSettingsAsset {
        threshold: settings.threshold,
        intensity: settings.intensity,
        radius: settings.radius,
    }
}

fn color_grading_from_asset(
    settings: SceneColorGradingSettingsAsset,
) -> RenderColorGradingSettings {
    RenderColorGradingSettings {
        exposure: settings.exposure,
        contrast: settings.contrast,
        saturation: settings.saturation,
        gamma: settings.gamma,
        tint: crate::core::math::Vec3::from_array(settings.tint),
    }
}

fn color_grading_to_asset(settings: RenderColorGradingSettings) -> SceneColorGradingSettingsAsset {
    SceneColorGradingSettingsAsset {
        exposure: settings.exposure,
        contrast: settings.contrast,
        saturation: settings.saturation,
        gamma: settings.gamma,
        tint: settings.tint.to_array(),
    }
}

fn effect_stack_from_asset(
    settings: ScenePostProcessEffectStackAsset,
) -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        tonemap: tonemap_from_asset(settings.tonemap),
        vignette: vignette_from_asset(settings.vignette),
        grain: grain_from_asset(settings.grain),
        dither: dither_from_asset(settings.dither),
        chromatic_aberration: chromatic_aberration_from_asset(settings.chromatic_aberration),
        fog: fog_from_asset(settings.fog),
        ..RenderPostProcessEffectStackSettings::default()
    }
}

fn effect_stack_to_asset(
    settings: RenderPostProcessEffectStackSettings,
) -> ScenePostProcessEffectStackAsset {
    ScenePostProcessEffectStackAsset {
        tonemap: tonemap_to_asset(settings.tonemap),
        vignette: vignette_to_asset(settings.vignette),
        grain: grain_to_asset(settings.grain),
        dither: dither_to_asset(settings.dither),
        chromatic_aberration: chromatic_aberration_to_asset(settings.chromatic_aberration),
        fog: fog_to_asset(settings.fog),
    }
}

fn tonemap_from_asset(settings: SceneTonemapSettingsAsset) -> RenderTonemapSettings {
    RenderTonemapSettings {
        operator: tonemap_operator_from_asset(settings.operator),
        exposure_bias: settings.exposure_bias,
        white_point: settings.white_point,
    }
}

fn tonemap_to_asset(settings: RenderTonemapSettings) -> SceneTonemapSettingsAsset {
    SceneTonemapSettingsAsset {
        operator: tonemap_operator_to_asset(settings.operator),
        exposure_bias: settings.exposure_bias,
        white_point: settings.white_point,
    }
}

fn tonemap_operator_from_asset(operator: SceneTonemapOperatorAsset) -> RenderTonemapOperator {
    match operator {
        SceneTonemapOperatorAsset::None => RenderTonemapOperator::None,
        SceneTonemapOperatorAsset::Reinhard => RenderTonemapOperator::Reinhard,
        SceneTonemapOperatorAsset::Aces => RenderTonemapOperator::Aces,
        SceneTonemapOperatorAsset::Filmic => RenderTonemapOperator::Filmic,
    }
}

fn tonemap_operator_to_asset(operator: RenderTonemapOperator) -> SceneTonemapOperatorAsset {
    match operator {
        RenderTonemapOperator::None => SceneTonemapOperatorAsset::None,
        RenderTonemapOperator::Reinhard => SceneTonemapOperatorAsset::Reinhard,
        RenderTonemapOperator::Aces => SceneTonemapOperatorAsset::Aces,
        RenderTonemapOperator::Filmic => SceneTonemapOperatorAsset::Filmic,
    }
}

fn vignette_from_asset(settings: SceneVignetteSettingsAsset) -> RenderVignetteSettings {
    RenderVignetteSettings {
        intensity: settings.intensity,
        smoothness: settings.smoothness,
        roundness: settings.roundness,
    }
}

fn vignette_to_asset(settings: RenderVignetteSettings) -> SceneVignetteSettingsAsset {
    SceneVignetteSettingsAsset {
        intensity: settings.intensity,
        smoothness: settings.smoothness,
        roundness: settings.roundness,
    }
}

fn grain_from_asset(settings: SceneFilmGrainSettingsAsset) -> RenderFilmGrainSettings {
    RenderFilmGrainSettings {
        intensity: settings.intensity,
        response: settings.response,
    }
}

fn grain_to_asset(settings: RenderFilmGrainSettings) -> SceneFilmGrainSettingsAsset {
    SceneFilmGrainSettingsAsset {
        intensity: settings.intensity,
        response: settings.response,
    }
}

fn dither_from_asset(settings: SceneDitherSettingsAsset) -> RenderDitherSettings {
    RenderDitherSettings {
        intensity: settings.intensity,
        scale: settings.scale,
    }
}

fn dither_to_asset(settings: RenderDitherSettings) -> SceneDitherSettingsAsset {
    SceneDitherSettingsAsset {
        intensity: settings.intensity,
        scale: settings.scale,
    }
}

fn chromatic_aberration_from_asset(
    settings: SceneChromaticAberrationSettingsAsset,
) -> RenderChromaticAberrationSettings {
    RenderChromaticAberrationSettings {
        intensity: settings.intensity,
        sample_spread: settings.sample_spread,
    }
}

fn chromatic_aberration_to_asset(
    settings: RenderChromaticAberrationSettings,
) -> SceneChromaticAberrationSettingsAsset {
    SceneChromaticAberrationSettingsAsset {
        intensity: settings.intensity,
        sample_spread: settings.sample_spread,
    }
}

fn fog_from_asset(settings: SceneFogSettingsAsset) -> RenderFogSettings {
    RenderFogSettings {
        density: settings.density,
        height_falloff: settings.height_falloff,
        color: crate::core::math::Vec3::from_array(settings.color),
    }
}

fn fog_to_asset(settings: RenderFogSettings) -> SceneFogSettingsAsset {
    SceneFogSettingsAsset {
        density: settings.density,
        height_falloff: settings.height_falloff,
        color: settings.color.to_array(),
    }
}
