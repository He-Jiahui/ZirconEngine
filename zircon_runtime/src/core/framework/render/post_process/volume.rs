use std::cmp::Ordering;

use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderLayerSet,
};
use crate::core::math::{Real, Vec3};

use super::{
    RenderBlurSettings, RenderChromaticAberrationSettings, RenderColorLookupSettings,
    RenderDepthOfFieldSettings, RenderDitherSettings, RenderFilmGrainSettings, RenderFogSettings,
    RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
    RenderTonemapOperator, RenderTonemapSettings, RenderVignetteSettings,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderResolvedPostProcessSettings {
    pub bloom: RenderBloomSettings,
    pub color_grading: RenderColorGradingSettings,
    pub effect_stack: RenderPostProcessEffectStackSettings,
}

impl RenderResolvedPostProcessSettings {
    pub const fn new(
        bloom: RenderBloomSettings,
        color_grading: RenderColorGradingSettings,
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> Self {
        Self {
            bloom,
            color_grading,
            effect_stack,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderPostProcessVolumeProfile {
    pub bloom: Option<RenderBloomSettings>,
    pub color_grading: Option<RenderColorGradingSettings>,
    pub effect_stack: Option<RenderPostProcessEffectStackSettings>,
}

impl RenderPostProcessVolumeProfile {
    pub const fn with_bloom(mut self, bloom: RenderBloomSettings) -> Self {
        self.bloom = Some(bloom);
        self
    }

    pub const fn with_color_grading(mut self, color_grading: RenderColorGradingSettings) -> Self {
        self.color_grading = Some(color_grading);
        self
    }

    pub const fn with_effect_stack(
        mut self,
        effect_stack: RenderPostProcessEffectStackSettings,
    ) -> Self {
        self.effect_stack = Some(effect_stack);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderPostProcessVolume {
    pub active: bool,
    pub is_global: bool,
    pub priority: Real,
    pub weight: Real,
    pub layer_mask: RenderLayerSet,
    /// Local volumes receive this already-computed influence from scene extract.
    /// Global volumes ignore it and use only `weight`.
    pub local_blend: Real,
    pub profile: RenderPostProcessVolumeProfile,
}

impl Default for RenderPostProcessVolume {
    fn default() -> Self {
        Self {
            active: true,
            is_global: true,
            priority: 0.0,
            weight: 1.0,
            layer_mask: RenderLayerSet::default(),
            local_blend: 1.0,
            profile: RenderPostProcessVolumeProfile::default(),
        }
    }
}

impl RenderPostProcessVolume {
    pub fn global(priority: Real, profile: RenderPostProcessVolumeProfile) -> Self {
        Self {
            priority,
            profile,
            ..Self::default()
        }
    }

    pub fn local(
        priority: Real,
        weight: Real,
        local_blend: Real,
        profile: RenderPostProcessVolumeProfile,
    ) -> Self {
        Self {
            is_global: false,
            priority,
            weight,
            local_blend,
            profile,
            ..Self::default()
        }
    }

    pub fn with_weight(mut self, weight: Real) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_layer_mask(mut self, layer_mask: RenderLayerSet) -> Self {
        self.layer_mask = layer_mask;
        self
    }

    fn influence(&self) -> Real {
        let weight = saturate(self.weight);
        if self.is_global {
            weight
        } else {
            weight * saturate(self.local_blend)
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderPostProcessVolumeStack {
    pub volumes: Vec<RenderPostProcessVolume>,
}

impl RenderPostProcessVolumeStack {
    pub fn from_volumes(volumes: impl IntoIterator<Item = RenderPostProcessVolume>) -> Self {
        Self {
            volumes: volumes.into_iter().collect(),
        }
    }

    pub fn resolve(
        &self,
        camera_layers: &RenderLayerSet,
        base_bloom: RenderBloomSettings,
        base_color_grading: RenderColorGradingSettings,
        base_effect_stack: RenderPostProcessEffectStackSettings,
    ) -> RenderResolvedPostProcessSettings {
        let mut settings = RenderResolvedPostProcessSettings::new(
            base_bloom,
            base_color_grading,
            base_effect_stack,
        );
        let mut applicable = self
            .volumes
            .iter()
            .enumerate()
            .filter(|(_, volume)| volume.active)
            .filter(|(_, volume)| volume.layer_mask.intersects(camera_layers))
            .filter_map(|(index, volume)| {
                let influence = volume.influence();
                (influence > 0.0).then_some((index, volume, influence))
            })
            .collect::<Vec<_>>();
        applicable.sort_by(|left, right| compare_volume_priority(left, right));

        for (_, volume, influence) in applicable {
            settings = blend_profile(settings, &volume.profile, influence);
        }

        settings
    }
}

fn compare_volume_priority(
    left: &(usize, &RenderPostProcessVolume, Real),
    right: &(usize, &RenderPostProcessVolume, Real),
) -> Ordering {
    left.1
        .priority
        .partial_cmp(&right.1.priority)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.0.cmp(&right.0))
}

fn blend_profile(
    mut settings: RenderResolvedPostProcessSettings,
    profile: &RenderPostProcessVolumeProfile,
    weight: Real,
) -> RenderResolvedPostProcessSettings {
    let weight = saturate(weight);
    if let Some(bloom) = profile.bloom {
        settings.bloom = blend_bloom(settings.bloom, bloom, weight);
    }
    if let Some(color_grading) = profile.color_grading {
        settings.color_grading = blend_color_grading(settings.color_grading, color_grading, weight);
    }
    if let Some(effect_stack) = profile.effect_stack {
        settings.effect_stack = blend_effect_stack(settings.effect_stack, effect_stack, weight);
    }
    settings
}

fn blend_bloom(
    from: RenderBloomSettings,
    to: RenderBloomSettings,
    weight: Real,
) -> RenderBloomSettings {
    RenderBloomSettings {
        threshold: lerp(from.threshold, to.threshold, weight),
        intensity: lerp(from.intensity, to.intensity, weight),
        radius: lerp(from.radius, to.radius, weight),
    }
}

fn blend_color_grading(
    from: RenderColorGradingSettings,
    to: RenderColorGradingSettings,
    weight: Real,
) -> RenderColorGradingSettings {
    RenderColorGradingSettings {
        exposure: lerp(from.exposure, to.exposure, weight),
        contrast: lerp(from.contrast, to.contrast, weight),
        saturation: lerp(from.saturation, to.saturation, weight),
        gamma: lerp(from.gamma, to.gamma, weight),
        tint: lerp_vec3(from.tint, to.tint, weight),
    }
}

fn blend_effect_stack(
    from: RenderPostProcessEffectStackSettings,
    to: RenderPostProcessEffectStackSettings,
    weight: Real,
) -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        tonemap: RenderTonemapSettings {
            operator: blend_tonemap_operator(from.tonemap.operator, to.tonemap.operator, weight),
            exposure_bias: lerp(from.tonemap.exposure_bias, to.tonemap.exposure_bias, weight),
            white_point: lerp(from.tonemap.white_point, to.tonemap.white_point, weight),
        },
        color_lookup: RenderColorLookupSettings {
            texture: blend_discrete(from.color_lookup.texture, to.color_lookup.texture, weight),
            texture_layout: blend_discrete(
                from.color_lookup.texture_layout,
                to.color_lookup.texture_layout,
                weight,
            ),
            intensity: lerp(
                from.color_lookup.intensity,
                to.color_lookup.intensity,
                weight,
            ),
        },
        blur: RenderBlurSettings {
            radius: lerp(from.blur.radius, to.blur.radius, weight),
        },
        depth_of_field: RenderDepthOfFieldSettings {
            focus_distance: lerp(
                from.depth_of_field.focus_distance,
                to.depth_of_field.focus_distance,
                weight,
            ),
            focus_range: lerp(
                from.depth_of_field.focus_range,
                to.depth_of_field.focus_range,
                weight,
            ),
            aperture: lerp(
                from.depth_of_field.aperture,
                to.depth_of_field.aperture,
                weight,
            ),
            focal_length_mm: lerp(
                from.depth_of_field.focal_length_mm,
                to.depth_of_field.focal_length_mm,
                weight,
            ),
            max_blur_radius: lerp(
                from.depth_of_field.max_blur_radius,
                to.depth_of_field.max_blur_radius,
                weight,
            ),
            bokeh_blade_count: blend_discrete(
                from.depth_of_field.bokeh_blade_count,
                to.depth_of_field.bokeh_blade_count,
                weight,
            ),
            bokeh_rotation_radians: lerp(
                from.depth_of_field.bokeh_rotation_radians,
                to.depth_of_field.bokeh_rotation_radians,
                weight,
            ),
        },
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: lerp(
                from.screen_space_reflection.intensity,
                to.screen_space_reflection.intensity,
                weight,
            ),
            thickness: lerp(
                from.screen_space_reflection.thickness,
                to.screen_space_reflection.thickness,
                weight,
            ),
            max_ray_distance: lerp(
                from.screen_space_reflection.max_ray_distance,
                to.screen_space_reflection.max_ray_distance,
                weight,
            ),
            max_steps: blend_discrete(
                from.screen_space_reflection.max_steps,
                to.screen_space_reflection.max_steps,
                weight,
            ),
        },
        vignette: RenderVignetteSettings {
            intensity: lerp(from.vignette.intensity, to.vignette.intensity, weight),
            smoothness: lerp(from.vignette.smoothness, to.vignette.smoothness, weight),
            roundness: lerp(from.vignette.roundness, to.vignette.roundness, weight),
        },
        grain: RenderFilmGrainSettings {
            intensity: lerp(from.grain.intensity, to.grain.intensity, weight),
            response: lerp(from.grain.response, to.grain.response, weight),
        },
        dither: RenderDitherSettings {
            intensity: lerp(from.dither.intensity, to.dither.intensity, weight),
            scale: lerp(from.dither.scale, to.dither.scale, weight),
        },
        chromatic_aberration: RenderChromaticAberrationSettings {
            intensity: lerp(
                from.chromatic_aberration.intensity,
                to.chromatic_aberration.intensity,
                weight,
            ),
            sample_spread: lerp(
                from.chromatic_aberration.sample_spread,
                to.chromatic_aberration.sample_spread,
                weight,
            ),
        },
        fog: RenderFogSettings {
            density: lerp(from.fog.density, to.fog.density, weight),
            height_falloff: lerp(from.fog.height_falloff, to.fog.height_falloff, weight),
            color: lerp_vec3(from.fog.color, to.fog.color, weight),
        },
    }
}

fn saturate(value: Real) -> Real {
    value.clamp(0.0, 1.0)
}

fn blend_tonemap_operator(
    from: RenderTonemapOperator,
    to: RenderTonemapOperator,
    weight: Real,
) -> RenderTonemapOperator {
    blend_discrete(from, to, weight)
}

fn blend_discrete<T: Copy>(from: T, to: T, weight: Real) -> T {
    if weight >= 0.5 {
        to
    } else {
        from
    }
}

fn lerp(from: Real, to: Real, weight: Real) -> Real {
    from + (to - from) * weight
}

fn lerp_vec3(from: Vec3, to: Vec3, weight: Real) -> Vec3 {
    from + (to - from) * weight
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderBloomSettings, RenderColorGradingSettings, RenderDepthOfFieldSettings,
        RenderDitherSettings, RenderLayerSet, RenderPostProcessEffectStackSettings,
        RenderScreenSpaceReflectionSettings, RenderTonemapOperator, RenderTonemapSettings,
        RenderVignetteSettings,
    };
    use crate::core::math::{Real, Vec3};

    use super::{
        RenderPostProcessVolume, RenderPostProcessVolumeProfile, RenderPostProcessVolumeStack,
    };

    #[test]
    fn volume_stack_blends_global_volumes_by_priority_order() {
        let stack = RenderPostProcessVolumeStack::from_volumes([
            RenderPostProcessVolume::global(
                10.0,
                RenderPostProcessVolumeProfile::default().with_bloom(RenderBloomSettings {
                    threshold: 1.0,
                    intensity: 1.0,
                    radius: 0.4,
                }),
            )
            .with_weight(0.5),
            RenderPostProcessVolume::global(
                0.0,
                RenderPostProcessVolumeProfile::default()
                    .with_bloom(RenderBloomSettings {
                        threshold: 1.0,
                        intensity: 0.5,
                        radius: 0.2,
                    })
                    .with_color_grading(RenderColorGradingSettings {
                        exposure: 1.2,
                        tint: Vec3::new(0.8, 0.9, 1.0),
                        ..Default::default()
                    }),
            ),
        ]);

        let resolved = stack.resolve(
            &RenderLayerSet::default(),
            RenderBloomSettings::default(),
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings::default(),
        );

        assert_near(resolved.bloom.intensity, 0.75);
        assert_near(resolved.bloom.radius, 0.3);
        assert_near(resolved.color_grading.exposure, 1.2);
        assert_eq!(resolved.color_grading.tint, Vec3::new(0.8, 0.9, 1.0));
    }

    #[test]
    fn volume_stack_filters_layers_and_applies_local_blend() {
        let ignored = RenderPostProcessVolume::global(
            0.0,
            RenderPostProcessVolumeProfile::default().with_effect_stack(
                RenderPostProcessEffectStackSettings {
                    vignette: RenderVignetteSettings {
                        intensity: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
        )
        .with_layer_mask(RenderLayerSet::layer(9));
        let local = RenderPostProcessVolume::local(
            1.0,
            0.5,
            0.5,
            RenderPostProcessVolumeProfile::default().with_effect_stack(
                RenderPostProcessEffectStackSettings {
                    vignette: RenderVignetteSettings {
                        intensity: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
        );

        let resolved = RenderPostProcessVolumeStack::from_volumes([ignored, local]).resolve(
            &RenderLayerSet::default(),
            RenderBloomSettings::default(),
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings::default(),
        );

        assert_near(resolved.effect_stack.vignette.intensity, 0.25);
    }

    #[test]
    fn volume_stack_resolves_extended_effect_stack_settings() {
        let resolved =
            RenderPostProcessVolumeStack::from_volumes([RenderPostProcessVolume::global(
                0.0,
                RenderPostProcessVolumeProfile::default().with_effect_stack(
                    RenderPostProcessEffectStackSettings {
                        tonemap: RenderTonemapSettings {
                            operator: RenderTonemapOperator::Aces,
                            white_point: 1.2,
                            ..Default::default()
                        },
                        depth_of_field: RenderDepthOfFieldSettings {
                            focus_range: 2.0,
                            aperture: 0.8,
                            focal_length_mm: 85.0,
                            max_blur_radius: 4.0,
                            bokeh_blade_count: 7,
                            bokeh_rotation_radians: 0.25,
                            ..Default::default()
                        },
                        screen_space_reflection: RenderScreenSpaceReflectionSettings {
                            intensity: 0.6,
                            max_steps: 48,
                            ..Default::default()
                        },
                        dither: RenderDitherSettings {
                            intensity: 0.2,
                            scale: 2.0,
                        },
                        ..Default::default()
                    },
                ),
            )])
            .resolve(
                &RenderLayerSet::default(),
                RenderBloomSettings::default(),
                RenderColorGradingSettings::default(),
                RenderPostProcessEffectStackSettings::default(),
            );

        assert!(resolved.effect_stack.is_enabled());
        assert_eq!(
            resolved.effect_stack.tonemap.operator,
            RenderTonemapOperator::Aces
        );
        assert_near(resolved.effect_stack.depth_of_field.aperture, 0.8);
        assert_near(resolved.effect_stack.depth_of_field.focus_range, 2.0);
        assert_near(resolved.effect_stack.depth_of_field.focal_length_mm, 85.0);
        assert_eq!(resolved.effect_stack.depth_of_field.bokeh_blade_count, 7);
        assert_near(
            resolved.effect_stack.depth_of_field.bokeh_rotation_radians,
            0.25,
        );
        assert_eq!(resolved.effect_stack.screen_space_reflection.max_steps, 48);
        assert_near(resolved.effect_stack.dither.intensity, 0.2);
    }

    fn assert_near(actual: Real, expected: Real) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "expected {actual} to be near {expected}"
        );
    }
}
