use super::{
    resource_status::RenderPostProcessEffectStackResourceStatus,
    RenderPostProcessEffectStackSettings,
};
use crate::core::framework::render::MotionVectorCameraStatus;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderPostProcessEffectStackReport {
    pub enabled: bool,
    pub active_family_count: usize,
    pub active_families: Vec<String>,
    pub approximated_family_count: usize,
    pub approximated_families: Vec<String>,
    pub missing_resource_count: usize,
    pub missing_resources: Vec<String>,
}

impl RenderPostProcessEffectStackReport {
    pub fn from_settings(settings: RenderPostProcessEffectStackSettings) -> Self {
        Self::from_settings_with_resources(
            settings,
            RenderPostProcessEffectStackResourceStatus::default(),
        )
    }

    pub fn from_settings_with_resources(
        settings: RenderPostProcessEffectStackSettings,
        resources: RenderPostProcessEffectStackResourceStatus,
    ) -> Self {
        let mut report = Self::default();

        push_label(
            &mut report.active_families,
            settings.tonemap.is_enabled(),
            "tonemap",
        );
        push_label(
            &mut report.active_families,
            settings.color_lookup.is_enabled(),
            "lut",
        );
        push_label(
            &mut report.active_families,
            settings.blur.is_enabled(),
            "blur",
        );
        push_label(
            &mut report.active_families,
            settings.depth_of_field.is_enabled(),
            "depth-of-field",
        );
        push_label(
            &mut report.active_families,
            settings.motion_blur.is_enabled(),
            "motion-blur",
        );
        push_label(
            &mut report.active_families,
            settings.screen_space_reflection.is_enabled(),
            "screen-space-reflection",
        );
        push_label(
            &mut report.active_families,
            settings.vignette.is_enabled(),
            "vignette",
        );
        push_label(
            &mut report.active_families,
            settings.grain.is_enabled(),
            "film-grain",
        );
        push_label(
            &mut report.active_families,
            settings.dither.is_enabled(),
            "dither",
        );
        push_label(
            &mut report.active_families,
            settings.chromatic_aberration.is_enabled(),
            "chromatic-aberration",
        );
        push_label(
            &mut report.active_families,
            settings.fog.is_enabled(),
            "fog",
        );

        push_label(
            &mut report.approximated_families,
            settings.depth_of_field.is_enabled(),
            "depth-of-field",
        );
        push_label(
            &mut report.approximated_families,
            settings.motion_blur.is_enabled(),
            "motion-blur",
        );
        push_label(
            &mut report.approximated_families,
            settings.screen_space_reflection.is_enabled(),
            "screen-space-reflection",
        );

        if settings.color_lookup.intensity > 0.0 && settings.color_lookup.texture.is_none() {
            report
                .missing_resources
                .push("effect-stack.lut.texture".to_string());
        }
        if settings.color_lookup.intensity > 0.0
            && !settings
                .color_lookup
                .texture_layout
                .has_valid_requested_size()
        {
            report
                .missing_resources
                .push("effect-stack.lut.texture-layout".to_string());
        }
        if settings.screen_space_reflection.is_enabled() && !resources.ssr_normal_available {
            report
                .missing_resources
                .push("effect-stack.ssr.normal".to_string());
        }
        let ssr_temporal_history_missing = settings.screen_space_reflection.is_enabled()
            && !resources.ssr_temporal_history_available;
        if ssr_temporal_history_missing {
            report
                .missing_resources
                .push("effect-stack.ssr.temporal-history".to_string());
        }
        let ssr_temporal_vector_missing = settings.screen_space_reflection.is_enabled()
            && resources.ssr_temporal_history_available
            && !resources.motion_vector_available;
        if ssr_temporal_vector_missing {
            report
                .missing_resources
                .push("effect-stack.ssr.temporal-motion-vector".to_string());
        }
        let ssr_temporal_prepass_missing = settings.screen_space_reflection.is_enabled()
            && resources.ssr_temporal_history_available
            && !resources.motion_vector_prepass_available;
        if ssr_temporal_prepass_missing {
            push_velocity_prepass_missing_resources(
                &mut report,
                resources,
                "effect-stack.ssr.temporal-velocity-prepass",
            );
        }
        if settings.motion_blur.is_enabled() && !resources.motion_vector_available {
            report
                .missing_resources
                .push("effect-stack.motion-blur.motion-vector".to_string());
        }
        let motion_vector_prepass_missing =
            settings.motion_blur.is_enabled() && !resources.motion_vector_prepass_available;
        if motion_vector_prepass_missing {
            push_velocity_prepass_missing_resources(
                &mut report,
                resources,
                "effect-stack.motion-blur.velocity-prepass",
            );
        }

        report.enabled = !report.active_families.is_empty();
        report.active_family_count = report.active_families.len();
        report.approximated_family_count = report.approximated_families.len();
        report.missing_resource_count = report.missing_resources.len();
        report
    }
}

fn push_label(labels: &mut Vec<String>, enabled: bool, label: &str) {
    if enabled {
        labels.push(label.to_string());
    }
}

fn push_velocity_prepass_missing_resources(
    report: &mut RenderPostProcessEffectStackReport,
    resources: RenderPostProcessEffectStackResourceStatus,
    prefix: &str,
) {
    report.missing_resources.push(prefix.to_string());
    if !resources.motion_vector_camera_available {
        report.missing_resources.push(format!("{prefix}.camera"));
    }
    if !resources.motion_vector_object_available {
        report.missing_resources.push(format!("{prefix}.object"));
    }
    if resources.motion_vector_camera_status == MotionVectorCameraStatus::MissingPreviousCamera {
        report
            .missing_resources
            .push(format!("{prefix}.camera-history"));
    }
    if resources.motion_vector_camera_status == MotionVectorCameraStatus::CameraCutOrInvalid {
        report
            .missing_resources
            .push(format!("{prefix}.camera-cut-or-invalid"));
    }
    if !resources.motion_vector_tile_max_available {
        report.missing_resources.push(format!("{prefix}.tile-max"));
    }
    if !resources.motion_vector_tile_max_coarse_available {
        report
            .missing_resources
            .push(format!("{prefix}.tile-max-coarse"));
    }
    if !resources.motion_vector_neighbor_max_available {
        report
            .missing_resources
            .push(format!("{prefix}.neighbor-max"));
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        RenderChromaticAberrationSettings, RenderColorLookupSettings,
        RenderColorLookupTextureLayout, RenderDepthOfFieldSettings, RenderDitherSettings,
        RenderFilmGrainSettings, RenderFogSettings, RenderMotionBlurSettings,
        RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
        RenderTonemapOperator, RenderTonemapSettings, RenderVignetteSettings,
    };
    use super::RenderPostProcessEffectStackResourceStatus;
    use crate::core::framework::render::MotionVectorCameraStatus;
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};

    #[test]
    fn effect_stack_report_records_active_approximated_and_missing_resources() {
        let settings = RenderPostProcessEffectStackSettings {
            tonemap: RenderTonemapSettings {
                operator: RenderTonemapOperator::Aces,
                ..Default::default()
            },
            color_lookup: RenderColorLookupSettings {
                texture: None,
                texture_layout: RenderColorLookupTextureLayout::Auto,
                intensity: 0.5,
            },
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 2.0,
                ..Default::default()
            },
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 2,
            },
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.4,
                max_steps: 16,
                ..Default::default()
            },
            vignette: RenderVignetteSettings {
                intensity: 0.25,
                ..Default::default()
            },
            grain: RenderFilmGrainSettings {
                intensity: 0.15,
                ..Default::default()
            },
            dither: RenderDitherSettings {
                intensity: 0.1,
                ..Default::default()
            },
            chromatic_aberration: RenderChromaticAberrationSettings {
                intensity: 0.2,
                ..Default::default()
            },
            fog: RenderFogSettings {
                density: 0.05,
                ..Default::default()
            },
            ..Default::default()
        };

        let report = settings.report();

        assert!(report.enabled);
        assert_eq!(report.active_family_count, 10);
        assert_eq!(
            report.active_families,
            labels([
                "tonemap",
                "lut",
                "depth-of-field",
                "motion-blur",
                "screen-space-reflection",
                "vignette",
                "film-grain",
                "dither",
                "chromatic-aberration",
                "fog",
            ])
        );
        assert_eq!(report.approximated_family_count, 3);
        assert_eq!(
            report.approximated_families,
            labels(["depth-of-field", "motion-blur", "screen-space-reflection"])
        );
        assert_eq!(report.missing_resource_count, 9);
        assert_eq!(
            report.missing_resources,
            labels([
                "effect-stack.lut.texture",
                "effect-stack.ssr.normal",
                "effect-stack.ssr.temporal-history",
                "effect-stack.motion-blur.motion-vector",
                "effect-stack.motion-blur.velocity-prepass",
                "effect-stack.motion-blur.velocity-prepass.camera",
                "effect-stack.motion-blur.velocity-prepass.tile-max",
                "effect-stack.motion-blur.velocity-prepass.tile-max-coarse",
                "effect-stack.motion-blur.velocity-prepass.neighbor-max",
            ])
        );
    }

    #[test]
    fn effect_stack_report_treats_authored_lut_as_renderer_bound_resource() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "postprocess/lut/filmic",
        ));
        let settings = RenderPostProcessEffectStackSettings {
            color_lookup: RenderColorLookupSettings {
                texture: Some(texture),
                texture_layout: RenderColorLookupTextureLayout::Texture2dStrip { size: 33 },
                intensity: 0.8,
            },
            ..Default::default()
        };

        let report = settings.report();

        assert!(report.enabled);
        assert_eq!(report.active_families, labels(["lut"]));
        assert_eq!(report.approximated_family_count, 0);
        assert!(report.approximated_families.is_empty());
        assert_eq!(report.missing_resource_count, 0);
        assert!(report.missing_resources.is_empty());
    }

    #[test]
    fn effect_stack_report_treats_bound_ssr_normal_as_available_but_keeps_temporal_history_gap() {
        let settings = RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 24,
                ..Default::default()
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            ssr_normal_available: true,
            ..Default::default()
        });

        assert!(report.enabled);
        assert_eq!(report.active_families, labels(["screen-space-reflection"]));
        assert_eq!(
            report.approximated_families,
            labels(["screen-space-reflection"])
        );
        assert_eq!(
            report.missing_resources,
            labels(["effect-stack.ssr.temporal-history"])
        );
    }

    #[test]
    fn effect_stack_report_records_ssr_temporal_velocity_stage_gaps() {
        let settings = RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 24,
                ..Default::default()
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            ssr_normal_available: true,
            ssr_temporal_history_available: true,
            motion_vector_available: true,
            motion_vector_prepass_available: false,
            ..Default::default()
        });

        assert_eq!(
            report.missing_resources,
            labels([
                "effect-stack.ssr.temporal-velocity-prepass",
                "effect-stack.ssr.temporal-velocity-prepass.camera",
                "effect-stack.ssr.temporal-velocity-prepass.object",
                "effect-stack.ssr.temporal-velocity-prepass.tile-max",
                "effect-stack.ssr.temporal-velocity-prepass.tile-max-coarse",
                "effect-stack.ssr.temporal-velocity-prepass.neighbor-max",
            ])
        );
    }

    #[test]
    fn effect_stack_report_clears_ssr_temporal_labels_when_history_and_velocity_chain_are_ready() {
        let settings = RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 24,
                ..Default::default()
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            ssr_normal_available: true,
            ssr_temporal_history_available: true,
            motion_vector_available: true,
            motion_vector_camera_available: true,
            motion_vector_object_available: true,
            motion_vector_tile_max_available: true,
            motion_vector_tile_max_coarse_available: true,
            motion_vector_neighbor_max_available: true,
            motion_vector_camera_status: MotionVectorCameraStatus::Ready,
            motion_vector_prepass_available: true,
            ..Default::default()
        });

        assert!(report.missing_resources.is_empty());
    }

    #[test]
    fn effect_stack_report_treats_bound_motion_vector_as_available_but_keeps_prepass_gap() {
        let settings = RenderPostProcessEffectStackSettings {
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 2,
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            motion_vector_available: true,
            motion_vector_prepass_available: false,
            ..Default::default()
        });

        assert!(report.enabled);
        assert_eq!(report.active_families, labels(["motion-blur"]));
        assert_eq!(report.approximated_families, labels(["motion-blur"]));
        assert_eq!(
            report.missing_resources,
            labels([
                "effect-stack.motion-blur.velocity-prepass",
                "effect-stack.motion-blur.velocity-prepass.camera",
                "effect-stack.motion-blur.velocity-prepass.object",
                "effect-stack.motion-blur.velocity-prepass.tile-max",
                "effect-stack.motion-blur.velocity-prepass.tile-max-coarse",
                "effect-stack.motion-blur.velocity-prepass.neighbor-max",
            ])
        );
    }

    #[test]
    fn effect_stack_report_clears_motion_vector_stage_labels_when_prepass_chain_is_ready() {
        let settings = RenderPostProcessEffectStackSettings {
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 2,
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            motion_vector_available: true,
            motion_vector_camera_available: true,
            motion_vector_object_available: true,
            motion_vector_tile_max_available: true,
            motion_vector_tile_max_coarse_available: true,
            motion_vector_neighbor_max_available: true,
            motion_vector_camera_status: MotionVectorCameraStatus::Ready,
            motion_vector_prepass_available: true,
            ..Default::default()
        });

        assert!(report.enabled);
        assert_eq!(report.active_families, labels(["motion-blur"]));
        assert_eq!(report.approximated_families, labels(["motion-blur"]));
        assert!(
            report.missing_resources.is_empty(),
            "complete motion-vector prepass should not report stage-level gaps"
        );
    }

    #[test]
    fn effect_stack_report_keeps_prepass_gap_when_object_velocity_is_missing() {
        let settings = RenderPostProcessEffectStackSettings {
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 2,
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            motion_vector_available: true,
            motion_vector_camera_available: true,
            motion_vector_object_available: false,
            motion_vector_tile_max_available: true,
            motion_vector_tile_max_coarse_available: true,
            motion_vector_neighbor_max_available: true,
            motion_vector_camera_status: MotionVectorCameraStatus::Ready,
            motion_vector_prepass_available: false,
            ..Default::default()
        });

        assert_eq!(
            report.missing_resources,
            labels([
                "effect-stack.motion-blur.velocity-prepass",
                "effect-stack.motion-blur.velocity-prepass.object",
            ])
        );
    }

    #[test]
    fn effect_stack_report_keeps_prepass_gap_when_camera_history_is_missing() {
        let settings = RenderPostProcessEffectStackSettings {
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 2,
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            motion_vector_available: true,
            motion_vector_camera_available: true,
            motion_vector_object_available: true,
            motion_vector_tile_max_available: true,
            motion_vector_tile_max_coarse_available: true,
            motion_vector_neighbor_max_available: true,
            motion_vector_camera_status: MotionVectorCameraStatus::MissingPreviousCamera,
            motion_vector_prepass_available: false,
            ..Default::default()
        });

        assert_eq!(
            report.missing_resources,
            labels([
                "effect-stack.motion-blur.velocity-prepass",
                "effect-stack.motion-blur.velocity-prepass.camera-history",
            ])
        );
    }

    #[test]
    fn effect_stack_report_keeps_prepass_gap_when_camera_vectors_are_cut_or_invalid() {
        let settings = RenderPostProcessEffectStackSettings {
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 2,
            },
            ..Default::default()
        };

        let report = settings.report_with_resources(RenderPostProcessEffectStackResourceStatus {
            motion_vector_available: true,
            motion_vector_camera_available: true,
            motion_vector_object_available: true,
            motion_vector_tile_max_available: true,
            motion_vector_tile_max_coarse_available: true,
            motion_vector_neighbor_max_available: true,
            motion_vector_camera_status: MotionVectorCameraStatus::CameraCutOrInvalid,
            motion_vector_prepass_available: false,
            ..Default::default()
        });

        assert_eq!(
            report.missing_resources,
            labels([
                "effect-stack.motion-blur.velocity-prepass",
                "effect-stack.motion-blur.velocity-prepass.camera-cut-or-invalid",
            ])
        );
    }

    #[test]
    fn effect_stack_report_records_invalid_lut_layout_size() {
        let settings = RenderPostProcessEffectStackSettings {
            color_lookup: RenderColorLookupSettings {
                texture: None,
                texture_layout: RenderColorLookupTextureLayout::Texture3d { size: 0 },
                intensity: 0.5,
            },
            ..Default::default()
        };

        let report = settings.report();

        assert_eq!(report.missing_resource_count, 2);
        assert_eq!(
            report.missing_resources,
            labels([
                "effect-stack.lut.texture",
                "effect-stack.lut.texture-layout"
            ])
        );
    }

    fn labels<const N: usize>(items: [&str; N]) -> Vec<String> {
        items.into_iter().map(str::to_string).collect()
    }
}
