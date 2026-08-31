use crate::asset::assets::{
    SceneAmbientOcclusionSettingsAsset, SceneAoQualityTierAsset, SceneBloomSettingsAsset,
    SceneChromaticAberrationSettingsAsset, SceneColorGradingSettingsAsset,
    SceneDitherSettingsAsset, SceneFilmGrainSettingsAsset, SceneFogSettingsAsset,
    ScenePostProcessEffectStackAsset, ScenePostProcessSettingsAsset, ScenePostProcessVolumeAsset,
    ScenePostProcessVolumeProfileAsset, SceneTonemapOperatorAsset, SceneTonemapSettingsAsset,
    SceneVignetteSettingsAsset, SceneVolumetricFogSettingsAsset,
};
use crate::core::framework::render::{
    AoQualityTier, AoSourceSettings, RenderBloomSettings, RenderChromaticAberrationSettings,
    RenderColorGradingSettings, RenderDitherSettings, RenderFilmGrainSettings, RenderFogSettings,
    RenderPostProcessEffectStackSettings, RenderPostProcessVolumeProfile, RenderTonemapOperator,
    RenderTonemapSettings, RenderVignetteSettings, VolumetricFogSettings,
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
    .with_ambient_occlusion(ambient_occlusion_from_asset(settings.ambient_occlusion))
}

pub(super) fn post_process_settings_to_asset(
    settings: PostProcessSettingsComponent,
) -> ScenePostProcessSettingsAsset {
    ScenePostProcessSettingsAsset {
        ambient_occlusion: ambient_occlusion_to_asset(settings.ambient_occlusion),
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
        ambient_occlusion: profile.ambient_occlusion.map(ambient_occlusion_from_asset),
        volumetric_fog: profile.volumetric_fog.map(volumetric_fog_from_asset),
        bloom: profile.bloom.map(bloom_from_asset),
        color_grading: profile.color_grading.map(color_grading_from_asset),
        effect_stack: profile.effect_stack.map(effect_stack_from_asset),
    }
}

fn volume_profile_to_asset(
    profile: RenderPostProcessVolumeProfile,
) -> ScenePostProcessVolumeProfileAsset {
    ScenePostProcessVolumeProfileAsset {
        ambient_occlusion: profile.ambient_occlusion.map(ambient_occlusion_to_asset),
        volumetric_fog: profile.volumetric_fog.map(volumetric_fog_to_asset),
        bloom: profile.bloom.map(bloom_to_asset),
        color_grading: profile.color_grading.map(color_grading_to_asset),
        effect_stack: profile.effect_stack.map(effect_stack_to_asset),
    }
}

fn ambient_occlusion_from_asset(settings: SceneAmbientOcclusionSettingsAsset) -> AoSourceSettings {
    AoSourceSettings {
        intensity: settings.intensity,
        radius_meters: settings.radius_meters,
        thickness_meters: settings.thickness_meters,
        depth_bias_meters: settings.depth_bias_meters,
        falloff_start_meters: settings.falloff_start_meters,
        quality: match settings.quality {
            SceneAoQualityTierAsset::Low => AoQualityTier::Low,
            SceneAoQualityTierAsset::Medium => AoQualityTier::Medium,
            SceneAoQualityTierAsset::High => AoQualityTier::High,
            SceneAoQualityTierAsset::Ultra => AoQualityTier::Ultra,
        },
        half_resolution: settings.half_resolution,
        temporal: settings.temporal,
    }
}

fn ambient_occlusion_to_asset(settings: AoSourceSettings) -> SceneAmbientOcclusionSettingsAsset {
    SceneAmbientOcclusionSettingsAsset {
        intensity: settings.intensity,
        radius_meters: settings.radius_meters,
        thickness_meters: settings.thickness_meters,
        depth_bias_meters: settings.depth_bias_meters,
        falloff_start_meters: settings.falloff_start_meters,
        quality: match settings.quality {
            AoQualityTier::Low => SceneAoQualityTierAsset::Low,
            AoQualityTier::Medium => SceneAoQualityTierAsset::Medium,
            AoQualityTier::High => SceneAoQualityTierAsset::High,
            AoQualityTier::Ultra => SceneAoQualityTierAsset::Ultra,
        },
        half_resolution: settings.half_resolution,
        temporal: settings.temporal,
    }
}

fn volumetric_fog_from_asset(settings: SceneVolumetricFogSettingsAsset) -> VolumetricFogSettings {
    VolumetricFogSettings {
        density: settings.density,
        albedo: crate::core::math::Vec3::from_array(settings.albedo),
        phase_g: settings.phase_g,
        height_falloff: settings.height_falloff,
        scattering_intensity: settings.scattering_intensity,
        depth_distribution_exp: settings.depth_distribution_exp,
        temporal: settings.temporal,
    }
    .sanitized()
}

fn volumetric_fog_to_asset(settings: VolumetricFogSettings) -> SceneVolumetricFogSettingsAsset {
    SceneVolumetricFogSettingsAsset {
        density: settings.density,
        albedo: settings.albedo.to_array(),
        phase_g: settings.phase_g,
        height_falloff: settings.height_falloff,
        scattering_intensity: settings.scattering_intensity,
        depth_distribution_exp: settings.depth_distribution_exp,
        temporal: settings.temporal,
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
