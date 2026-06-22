use crate::core::framework::render::{
    AntiAliasMode, AntiAliasSettings, RenderBloomSettings, RenderColorGradingSettings,
    RenderExposureMode, RenderExposureSettings,
};

use super::{
    PostProcessEffectKind, PostProcessEffectSettings, PostProcessPassGraph,
    RenderPostProcessEffectStackSettings,
};

pub struct PostProcessGraphResourceNames;

impl PostProcessGraphResourceNames {
    pub const SCENE_COLOR: &'static str = "scene-color";
    pub const SCENE_DEPTH: &'static str = "scene-depth";
    pub const SHADOW_ATLAS: &'static str = "shadow-atlas";
    pub const SCENE_VELOCITY: &'static str = "scene-velocity";
    pub const MOTION_VECTOR_TILE_MAX: &'static str = "postprocess.motion-vector.tile-max";
    pub const MOTION_VECTOR_TILE_MAX_COARSE: &'static str =
        "postprocess.motion-vector.tile-max.coarse";
    pub const MOTION_VECTOR_NEIGHBOR_MAX: &'static str = "postprocess.motion-vector.neighbor-max";
    pub const DEPTH_OF_FIELDED: &'static str = "postprocess.depth-of-fielded";
    pub const MOTION_BLURRED: &'static str = "postprocess.motion-blurred";
    pub const GBUFFER_ALBEDO: &'static str = "gbuffer-albedo";
    pub const GBUFFER_NORMAL: &'static str = "gbuffer-normal";
    pub const GBUFFER_MATERIAL: &'static str = "gbuffer-material";
    pub const AMBIENT_OCCLUSION: &'static str = "ambient-occlusion";
    pub const CONTACT_SHADOW_OCCLUSION: &'static str = "contact-shadow-occlusion";
    pub const GLOBAL_ILLUMINATION: &'static str = "global-illumination";
    pub const LIGHT_LIST: &'static str = "light-list";
    pub const LIGHT_GRID_PARAMS: &'static str = "light-grid-params";
    pub const LIGHT_ZBINS: &'static str = "light-zbins";
    pub const LIGHT_TILE_MASKS: &'static str = "light-tile-masks";
    pub const HZB_FURTHEST: &'static str = "hzb-furthest";
    // Temporal resources use distinct names so a pass cannot silently read and overwrite the same history slot.
    pub const HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION: &'static str =
        "history.previous.screen-space-reflection";
    pub const HISTORY_PREVIOUS_HZB_FURTHEST: &'static str = "history.previous.hzb-furthest";
    pub const TAA_HISTORY_PREVIOUS: &'static str = "taa.history.previous.scene-color";
    pub const TAA_HISTORY_CURRENT: &'static str = "taa.history.current.scene-color";
    pub const TAA_OUTPUT: &'static str = "taa.output.scene-color";
    pub const TAA_REACTIVE_MASK: &'static str = "taa.reactive-mask";
    pub const BLOOM: &'static str = "bloom-texture";
    pub const EXPOSURE_HISTOGRAM: &'static str = "postprocess.exposure.histogram";
    pub const EXPOSURE_PREVIOUS: &'static str = "history.previous.exposure";
    pub const EXPOSURE_CURRENT: &'static str = "history.current.exposure";
    pub const COLOR_LUT: &'static str = "postprocess.color-lut";
    pub const COLOR_GRADED: &'static str = "postprocess.color-graded";
    pub const SCENE_COMPOSITED: &'static str = "postprocess.scene-composited";
    pub const BLURRED: &'static str = "postprocess.blurred";
    pub const EFFECT_STACKED: &'static str = "postprocess.effect-stacked";
    pub const TONEMAPPED: &'static str = "postprocess.tonemapped";
    pub const DEPTH_OF_FIELD_COC: &'static str = "postprocess.depth-of-field.coc";
    pub const DEPTH_OF_FIELD_BOKEH: &'static str = "postprocess.depth-of-field.bokeh";
    pub const SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID: &'static str =
        "postprocess.screen-space-reflection.reflection-pyramid";
    pub const SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE: &'static str =
        "postprocess.screen-space-reflection.reflection-pyramid.coarse";
    pub const SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION: &'static str =
        "postprocess.screen-space-reflection.specular-occlusion";
    pub const SCREEN_SPACE_REFLECTION_HISTORY: &'static str =
        "postprocess.screen-space-reflection.history";
    pub const UPSCALED: &'static str = "postprocess.upscaled";
    pub const FINAL_COMPOSITED: &'static str = "postprocess.terminal-aa-input";
    pub const FINAL_COLOR: &'static str = "final-color";
    pub const VIEWPORT_OUTPUT: &'static str = "viewport-output";
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostProcessStackDescriptor {
    pub initial_resources: Vec<String>,
    pub effects: Vec<PostProcessEffectSettings>,
}

impl Default for PostProcessStackDescriptor {
    fn default() -> Self {
        Self::from_extract_settings(
            &RenderBloomSettings::default(),
            &RenderColorGradingSettings::default(),
            false,
            false,
        )
    }
}

impl PostProcessStackDescriptor {
    pub fn from_extract_settings(
        bloom: &RenderBloomSettings,
        color_grading: &RenderColorGradingSettings,
        temporal_history_enabled: bool,
        history_available: bool,
    ) -> Self {
        Self::from_extract_settings_with_anti_alias(
            bloom,
            color_grading,
            temporal_history_enabled,
            history_available,
            &AntiAliasSettings::off(),
        )
    }

    pub fn from_extract_settings_with_anti_alias(
        bloom: &RenderBloomSettings,
        color_grading: &RenderColorGradingSettings,
        temporal_history_enabled: bool,
        history_available: bool,
        anti_alias: &AntiAliasSettings,
    ) -> Self {
        Self::from_extract_settings_with_effect_stack_and_anti_alias(
            bloom,
            color_grading,
            &RenderPostProcessEffectStackSettings::default(),
            temporal_history_enabled,
            history_available,
            anti_alias,
        )
    }

    pub fn from_extract_settings_with_effect_stack_and_anti_alias(
        bloom: &RenderBloomSettings,
        color_grading: &RenderColorGradingSettings,
        effect_stack: &RenderPostProcessEffectStackSettings,
        temporal_history_enabled: bool,
        history_available: bool,
        anti_alias: &AntiAliasSettings,
    ) -> Self {
        Self::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
            bloom,
            color_grading,
            RenderExposureSettings::default(),
            effect_stack,
            temporal_history_enabled,
            history_available,
            anti_alias,
        )
    }

    pub fn from_extract_settings_with_effect_stack_exposure_and_anti_alias(
        bloom: &RenderBloomSettings,
        color_grading: &RenderColorGradingSettings,
        exposure: RenderExposureSettings,
        effect_stack: &RenderPostProcessEffectStackSettings,
        temporal_history_enabled: bool,
        history_available: bool,
        anti_alias: &AntiAliasSettings,
    ) -> Self {
        Self::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale(
            bloom,
            color_grading,
            exposure,
            effect_stack,
            temporal_history_enabled,
            history_available,
            anti_alias,
            false,
        )
    }

    pub fn from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale(
        bloom: &RenderBloomSettings,
        color_grading: &RenderColorGradingSettings,
        exposure: RenderExposureSettings,
        effect_stack: &RenderPostProcessEffectStackSettings,
        temporal_history_enabled: bool,
        history_available: bool,
        anti_alias: &AntiAliasSettings,
        upscale_required: bool,
    ) -> Self {
        let bloom_enabled = bloom.intensity > 0.0;
        let color_grading_enabled = *color_grading != RenderColorGradingSettings::default();
        let effect_stack_enabled = effect_stack.is_enabled();
        let color_lut_bake_enabled = color_grading_enabled
            || effect_stack.tonemap.is_enabled()
            || effect_stack.color_lookup.is_enabled();
        let taa_enabled = anti_alias.mode == AntiAliasMode::Taa && history_available;
        let terminal_anti_alias_effect = terminal_anti_alias_effect_kind(anti_alias.mode);
        let terminal_anti_alias_enabled = terminal_anti_alias_effect.is_some();
        let exposure_histogram_enabled = exposure.mode == RenderExposureMode::Histogram;
        let ssr_enabled = effect_stack.screen_space_reflection.is_enabled();
        let ssr_temporal_enabled = ssr_enabled && temporal_history_enabled && history_available;
        let depth_of_field_enabled = effect_stack.depth_of_field.is_enabled();
        let motion_blur_enabled = effect_stack.motion_blur.is_enabled();
        let scene_composite_enabled = ssr_enabled || effect_stack.fog.is_enabled();
        let blur_enabled = effect_stack.blur.is_enabled();
        let scene_velocity_enabled = taa_enabled || motion_blur_enabled || ssr_enabled;
        let motion_vector_effects_enabled = effect_stack.motion_blur.is_enabled() || ssr_enabled;
        let mut initial_resources = vec![
            PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            PostProcessGraphResourceNames::SCENE_DEPTH.to_string(),
            PostProcessGraphResourceNames::LIGHT_LIST.to_string(),
        ];
        if depth_of_field_enabled {
            initial_resources.push(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC.to_string());
            initial_resources.push(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH.to_string());
        }
        if taa_enabled {
            initial_resources.push(PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string());
            initial_resources.push(PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string());
        }
        initial_resources.push(PostProcessGraphResourceNames::EXPOSURE_PREVIOUS.to_string());
        if ssr_temporal_enabled {
            initial_resources.push(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string(),
            );
        }
        if ssr_enabled {
            initial_resources.push(PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string());
            initial_resources.push(PostProcessGraphResourceNames::GBUFFER_MATERIAL.to_string());
            initial_resources.push(PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string());
            initial_resources.push(PostProcessGraphResourceNames::HZB_FURTHEST.to_string());
        }
        if scene_velocity_enabled {
            initial_resources.push(PostProcessGraphResourceNames::SCENE_VELOCITY.to_string());
        }
        if motion_vector_effects_enabled {
            initial_resources
                .push(PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX.to_string());
            initial_resources
                .push(PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE.to_string());
            initial_resources
                .push(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string());
        }

        let final_scene_color = if taa_enabled {
            PostProcessGraphResourceNames::TAA_OUTPUT
        } else {
            PostProcessGraphResourceNames::SCENE_COLOR
        };
        let post_dof_scene_color = if depth_of_field_enabled {
            PostProcessGraphResourceNames::DEPTH_OF_FIELDED
        } else {
            final_scene_color
        };
        let post_motion_scene_color = if motion_blur_enabled {
            PostProcessGraphResourceNames::MOTION_BLURRED
        } else {
            post_dof_scene_color
        };
        let post_composite_scene_color = if scene_composite_enabled {
            PostProcessGraphResourceNames::SCENE_COMPOSITED
        } else {
            post_motion_scene_color
        };
        let post_blur_scene_color = if blur_enabled {
            PostProcessGraphResourceNames::BLURRED
        } else {
            post_composite_scene_color
        };
        let mut final_inputs = vec![post_blur_scene_color.to_string()];
        let mut scene_color_after = Vec::new();
        if taa_enabled {
            scene_color_after.push(PostProcessEffectKind::TaaResolve);
        }
        let mut post_dof_after = scene_color_after.clone();
        if depth_of_field_enabled {
            post_dof_after.push(PostProcessEffectKind::DepthOfField);
        }
        let mut post_motion_after = post_dof_after.clone();
        if motion_blur_enabled {
            post_motion_after.push(PostProcessEffectKind::MotionBlur);
        }
        let mut scene_composite_after_base = post_motion_after.clone();
        if bloom_enabled {
            scene_composite_after_base.push(PostProcessEffectKind::Bloom);
        }
        scene_composite_after_base.push(PostProcessEffectKind::ExposureResolve);
        let mut post_composite_after = post_motion_after.clone();
        if scene_composite_enabled {
            post_composite_after.push(PostProcessEffectKind::SceneComposite);
        }
        let mut post_blur_after = post_composite_after.clone();
        if blur_enabled {
            post_blur_after.push(PostProcessEffectKind::Blur);
        }
        let mut final_after = post_blur_after.clone();
        if bloom_enabled {
            final_inputs.push(PostProcessGraphResourceNames::BLOOM.to_string());
            final_after.push(PostProcessEffectKind::Bloom);
        }
        if color_lut_bake_enabled {
            final_inputs.push(PostProcessGraphResourceNames::COLOR_LUT.to_string());
            final_after.push(PostProcessEffectKind::ColorLutBake);
        }
        final_after.push(PostProcessEffectKind::ExposureResolve);
        let mut effect_stack_inputs = final_inputs.clone();
        effect_stack_inputs.push(PostProcessGraphResourceNames::EXPOSURE_CURRENT.to_string());
        if effect_stack_requires_scene_depth(
            effect_stack,
            depth_of_field_enabled,
            motion_blur_enabled,
            scene_composite_enabled,
        ) && !effect_stack_inputs
            .iter()
            .any(|resource| resource.as_str() == PostProcessGraphResourceNames::SCENE_DEPTH)
        {
            effect_stack_inputs.push(PostProcessGraphResourceNames::SCENE_DEPTH.to_string());
        }
        if effect_stack.motion_blur.is_enabled()
            && !motion_blur_enabled
            && !effect_stack_inputs.iter().any(|resource| {
                resource.as_str() == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
            })
        {
            effect_stack_inputs
                .push(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string());
        }
        let pre_scene_composite_after = scene_composite_after_base;
        let effect_stack_after = final_after.clone();
        if effect_stack_enabled {
            final_inputs = vec![PostProcessGraphResourceNames::EFFECT_STACKED.to_string()];
            final_after = vec![PostProcessEffectKind::Uber];
        }
        if ssr_enabled && !scene_composite_enabled {
            final_inputs
                .push(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string());
            final_after.push(PostProcessEffectKind::ScreenSpaceReflectionResolve);
        }
        let output_transfer_input = if upscale_required {
            vec![PostProcessGraphResourceNames::UPSCALED.to_string()]
        } else {
            final_inputs.clone()
        };
        let mut output_transfer_after = final_after.clone();
        if upscale_required {
            output_transfer_after.push(PostProcessEffectKind::Upscale);
        }
        let output_transfer_output = if terminal_anti_alias_enabled {
            PostProcessGraphResourceNames::FINAL_COMPOSITED
        } else {
            PostProcessGraphResourceNames::FINAL_COLOR
        };
        let color_grading_after = if bloom_enabled {
            vec![PostProcessEffectKind::Bloom]
        } else {
            Vec::new()
        };
        let mut color_lut_after = color_grading_after;
        color_lut_after.push(PostProcessEffectKind::ExposureResolve);
        let mut effects = Vec::new();
        if taa_enabled {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::TaaResolve)
                    .with_required_inputs([
                        PostProcessGraphResourceNames::SCENE_COLOR,
                        PostProcessGraphResourceNames::SCENE_DEPTH,
                        PostProcessGraphResourceNames::SCENE_VELOCITY,
                        PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
                        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
                    ])
                    .with_produced_outputs([
                        PostProcessGraphResourceNames::TAA_OUTPUT,
                        PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
                    ]),
            );
        }
        if depth_of_field_enabled {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::DepthOfField)
                    .with_required_inputs([
                        final_scene_color,
                        PostProcessGraphResourceNames::SCENE_DEPTH,
                        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
                        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
                    ])
                    .with_produced_outputs([PostProcessGraphResourceNames::DEPTH_OF_FIELDED])
                    .with_after(scene_color_after.clone()),
            );
        }
        if motion_blur_enabled {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::MotionBlur)
                    .with_required_inputs([
                        post_dof_scene_color,
                        PostProcessGraphResourceNames::SCENE_DEPTH,
                        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
                    ])
                    .with_produced_outputs([PostProcessGraphResourceNames::MOTION_BLURRED])
                    .with_after(post_dof_after.clone()),
            );
        }
        if exposure_histogram_enabled {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::ExposureHistogram)
                    .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                    .with_produced_outputs([PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM]),
            );
        }
        effects.push(
            PostProcessEffectSettings::new(PostProcessEffectKind::ExposureResolve)
                .with_required_inputs(exposure_resolve_inputs(exposure_histogram_enabled))
                .with_produced_outputs([PostProcessGraphResourceNames::EXPOSURE_CURRENT])
                .with_after(exposure_resolve_after(exposure_histogram_enabled)),
        );
        effects.extend([
            PostProcessEffectSettings::new(PostProcessEffectKind::Bloom)
                .with_enabled(bloom_enabled)
                .with_required_inputs([post_motion_scene_color])
                .with_produced_outputs([PostProcessGraphResourceNames::BLOOM])
                .with_after(post_motion_after.clone()),
            PostProcessEffectSettings::new(PostProcessEffectKind::ColorLutBake)
                .with_enabled(color_lut_bake_enabled)
                .with_required_inputs([PostProcessGraphResourceNames::EXPOSURE_CURRENT])
                .with_produced_outputs([PostProcessGraphResourceNames::COLOR_LUT])
                .with_after(color_lut_after),
        ]);
        if effect_stack_enabled {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::Uber)
                    .with_required_inputs(effect_stack_inputs)
                    .with_produced_outputs(effect_stack_outputs(effect_stack, upscale_required))
                    .with_after(effect_stack_after.clone()),
            );
        } else if upscale_required {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::Uber)
                    .with_required_inputs(final_inputs.clone())
                    .with_produced_outputs([PostProcessGraphResourceNames::TONEMAPPED])
                    .with_after(final_after.clone()),
            );
        }
        if ssr_enabled {
            effects.push(
                PostProcessEffectSettings::new(
                    PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid,
                )
                .with_required_inputs(screen_space_reflection_reflection_pyramid_inputs())
                .with_produced_outputs([
                    PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
                ])
                .with_after(pre_scene_composite_after.clone()),
            );
            effects.push(
                PostProcessEffectSettings::new(
                    PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse,
                )
                .with_required_inputs(screen_space_reflection_reflection_pyramid_coarse_inputs())
                .with_produced_outputs([
                    PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
                ])
                .with_after([PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid]),
            );
            effects.push(
                PostProcessEffectSettings::new(
                    PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion,
                )
                .with_required_inputs(screen_space_reflection_specular_occlusion_inputs())
                .with_produced_outputs([
                    PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
                ])
                .with_after(pre_scene_composite_after.clone()),
            );
            let mut resolve_after = pre_scene_composite_after.clone();
            resolve_after.push(PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid);
            resolve_after.push(PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse);
            resolve_after.push(PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion);
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::ScreenSpaceReflectionResolve)
                    .with_required_inputs(screen_space_reflection_resolve_inputs(
                        ssr_temporal_enabled,
                    ))
                    .with_produced_outputs([
                        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
                    ])
                    .with_after(resolve_after),
            );
        }
        if scene_composite_enabled {
            let mut scene_composite_inputs = vec![
                post_motion_scene_color,
                PostProcessGraphResourceNames::SCENE_DEPTH,
                PostProcessGraphResourceNames::EXPOSURE_CURRENT,
            ];
            if ssr_enabled {
                scene_composite_inputs
                    .push(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY);
            }
            let mut scene_composite_after = pre_scene_composite_after;
            if ssr_enabled {
                scene_composite_after.push(PostProcessEffectKind::ScreenSpaceReflectionResolve);
            }
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::SceneComposite)
                    .with_required_inputs(scene_composite_inputs)
                    .with_produced_outputs([PostProcessGraphResourceNames::SCENE_COMPOSITED])
                    .with_after(scene_composite_after),
            );
        }
        if blur_enabled {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::Blur)
                    .with_required_inputs([post_composite_scene_color])
                    .with_produced_outputs([PostProcessGraphResourceNames::BLURRED])
                    .with_after(post_composite_after),
            );
        }
        if upscale_required {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::Upscale)
                    .with_required_inputs([PostProcessGraphResourceNames::TONEMAPPED])
                    .with_produced_outputs([PostProcessGraphResourceNames::UPSCALED])
                    .with_after([PostProcessEffectKind::Uber]),
            );
        }
        effects.push(
            PostProcessEffectSettings::new(PostProcessEffectKind::OutputTransfer)
                .with_required_inputs(output_transfer_input)
                .with_produced_outputs([output_transfer_output])
                .with_after(output_transfer_after),
        );
        if let Some(terminal_anti_alias_effect) = terminal_anti_alias_effect {
            effects.push(
                PostProcessEffectSettings::new(terminal_anti_alias_effect)
                    .with_required_inputs([PostProcessGraphResourceNames::FINAL_COMPOSITED])
                    .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COLOR])
                    .with_after([PostProcessEffectKind::OutputTransfer]),
            );
        }

        Self {
            initial_resources,
            effects,
        }
    }

    pub fn validated_graph(&self) -> PostProcessPassGraph {
        PostProcessPassGraph::validate_stack(self)
            .expect("default post-process stack descriptor must validate")
    }

    pub fn without_history_resources(&self) -> Self {
        let mut stack = self.clone();
        stack.initial_resources.retain(|resource| {
            resource != PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
                && resource != PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS
                && resource != PostProcessGraphResourceNames::TAA_HISTORY_CURRENT
                && resource != PostProcessGraphResourceNames::TAA_REACTIVE_MASK
        });
        for effect in &mut stack.effects {
            if effect.kind == PostProcessEffectKind::TaaResolve {
                effect.enabled = false;
            }
            let had_taa_output_input = effect
                .required_inputs
                .iter()
                .any(|resource| resource == PostProcessGraphResourceNames::TAA_OUTPUT);
            effect.required_inputs.retain(|resource| {
                resource != PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
                    && resource != PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS
                    && resource != PostProcessGraphResourceNames::TAA_HISTORY_CURRENT
                    && resource != PostProcessGraphResourceNames::TAA_OUTPUT
                    && resource != PostProcessGraphResourceNames::TAA_REACTIVE_MASK
            });
            if had_taa_output_input
                && !effect
                    .required_inputs
                    .iter()
                    .any(|resource| resource == PostProcessGraphResourceNames::SCENE_COLOR)
            {
                effect
                    .required_inputs
                    .insert(0, PostProcessGraphResourceNames::SCENE_COLOR.to_string());
            }
            effect.produced_outputs.retain(|resource| {
                resource != PostProcessGraphResourceNames::TAA_HISTORY_CURRENT
                    && resource != PostProcessGraphResourceNames::TAA_OUTPUT
                    && resource != PostProcessGraphResourceNames::TAA_REACTIVE_MASK
            });
            effect
                .after
                .retain(|dependency| *dependency != PostProcessEffectKind::TaaResolve);
        }
        let needs_reconstructed_motion_vectors = stack.initial_resources.iter().any(|resource| {
            resource == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
                || resource == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
                || resource == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
        });
        if !needs_reconstructed_motion_vectors {
            stack
                .initial_resources
                .retain(|resource| resource != PostProcessGraphResourceNames::SCENE_VELOCITY);
        }
        stack
    }
}

fn effect_stack_requires_scene_depth(
    effect_stack: &RenderPostProcessEffectStackSettings,
    depth_of_field_split: bool,
    motion_blur_split: bool,
    scene_composite_split: bool,
) -> bool {
    (effect_stack.depth_of_field.is_enabled() && !depth_of_field_split)
        || (effect_stack.motion_blur.is_enabled() && !motion_blur_split)
        || (effect_stack.fog.density > 0.0 && !scene_composite_split)
}

fn effect_stack_outputs(
    _effect_stack: &RenderPostProcessEffectStackSettings,
    _upscale_required: bool,
) -> Vec<&'static str> {
    vec![
        PostProcessGraphResourceNames::EFFECT_STACKED,
        PostProcessGraphResourceNames::TONEMAPPED,
    ]
}

fn exposure_resolve_inputs(histogram_enabled: bool) -> Vec<&'static str> {
    let mut inputs = vec![PostProcessGraphResourceNames::EXPOSURE_PREVIOUS];
    if histogram_enabled {
        inputs.push(PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM);
    }
    inputs
}

fn exposure_resolve_after(histogram_enabled: bool) -> Vec<PostProcessEffectKind> {
    if histogram_enabled {
        vec![PostProcessEffectKind::ExposureHistogram]
    } else {
        Vec::new()
    }
}

fn terminal_anti_alias_effect_kind(mode: AntiAliasMode) -> Option<PostProcessEffectKind> {
    match mode {
        AntiAliasMode::Fxaa => Some(PostProcessEffectKind::Fxaa),
        AntiAliasMode::Smaa => Some(PostProcessEffectKind::Smaa),
        _ => None,
    }
}

fn screen_space_reflection_specular_occlusion_inputs() -> Vec<&'static str> {
    vec![
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
    ]
}

fn screen_space_reflection_reflection_pyramid_inputs() -> Vec<&'static str> {
    vec![PostProcessGraphResourceNames::SCENE_COLOR]
}

fn screen_space_reflection_reflection_pyramid_coarse_inputs() -> Vec<&'static str> {
    vec![PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID]
}

fn screen_space_reflection_resolve_inputs(ssr_temporal_enabled: bool) -> Vec<&'static str> {
    let mut inputs = vec![
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::HZB_FURTHEST,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
    ];
    if ssr_temporal_enabled {
        inputs.push(PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION);
    }
    inputs
}

#[cfg(test)]
mod tests {
    use super::{PostProcessGraphResourceNames, PostProcessStackDescriptor};
    use crate::core::framework::render::{
        AntiAliasSettings, PostProcessEffectKind, RenderBlurSettings, RenderDepthOfFieldSettings,
        RenderExposureMode, RenderExposureSettings, RenderMotionBlurSettings,
        RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
        RenderVignetteSettings,
    };

    fn expected_uber_effect_stack_outputs() -> Vec<String> {
        [
            PostProcessGraphResourceNames::EFFECT_STACKED,
            PostProcessGraphResourceNames::TONEMAPPED,
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    #[test]
    fn manual_exposure_declares_resolve_without_histogram() {
        let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
            &Default::default(),
            &Default::default(),
            RenderExposureSettings {
                mode: RenderExposureMode::Manual,
                ..Default::default()
            },
            &RenderPostProcessEffectStackSettings::default(),
            false,
            false,
            &AntiAliasSettings::off(),
        );

        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::EXPOSURE_PREVIOUS.to_string()));
        assert!(!stack
            .effects
            .iter()
            .any(|effect| effect.kind == PostProcessEffectKind::ExposureHistogram));
        let resolve = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::ExposureResolve)
            .expect("manual exposure still writes the unified exposure buffer");
        assert_eq!(
            resolve.required_inputs,
            vec![PostProcessGraphResourceNames::EXPOSURE_PREVIOUS.to_string()]
        );
        assert_eq!(
            resolve.produced_outputs,
            vec![PostProcessGraphResourceNames::EXPOSURE_CURRENT.to_string()]
        );

        let graph = stack.validated_graph();
        let resolve_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::ExposureResolve)
            .expect("validated graph should keep exposure resolve");
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .expect("validated graph should keep final transfer");
        assert!(resolve_index < output_index);
    }

    #[test]
    fn default_stack_declares_light_list_for_uber_cluster_bind_group() {
        let stack = PostProcessStackDescriptor::default();

        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::LIGHT_LIST.to_string()));
    }

    #[test]
    fn enabled_effect_stack_declares_tonemapped_for_uber_descriptor() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    vignette: RenderVignetteSettings {
                        intensity: 0.25,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                false,
                false,
                &AntiAliasSettings::off(),
            );
        let uber = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Uber)
            .expect("enabled effect stack should keep the uber pass");

        assert!(
            uber.produced_outputs
                .contains(&PostProcessGraphResourceNames::TONEMAPPED.to_string()),
            "uber writes TONEMAPPED in the built-in pass descriptor, so the stack must declare it"
        );
    }

    #[test]
    fn histogram_exposure_declares_histogram_before_resolve() {
        let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
            &Default::default(),
            &Default::default(),
            RenderExposureSettings {
                mode: RenderExposureMode::Histogram,
                ..Default::default()
            },
            &RenderPostProcessEffectStackSettings::default(),
            false,
            false,
            &AntiAliasSettings::off(),
        );

        let histogram = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::ExposureHistogram)
            .expect("histogram mode should build the histogram node");
        assert_eq!(
            histogram.required_inputs,
            vec![PostProcessGraphResourceNames::SCENE_COLOR.to_string()]
        );
        assert_eq!(
            histogram.produced_outputs,
            vec![PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM.to_string()]
        );

        let resolve = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::ExposureResolve)
            .expect("histogram mode should resolve exposure");
        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM.to_string()));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ExposureHistogram));

        let graph = stack.validated_graph();
        let histogram_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::ExposureHistogram)
            .expect("validated graph should keep exposure histogram");
        let resolve_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::ExposureResolve)
            .expect("validated graph should keep exposure resolve");
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .expect("validated graph should keep final transfer");
        assert!(histogram_index < resolve_index);
        assert!(resolve_index < output_index);
    }

    #[test]
    fn fxaa_terminal_anti_alias_routes_output_transfer_through_terminal_input() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings::default(),
                false,
                false,
                &AntiAliasSettings::fxaa(),
            );

        let output_transfer = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
            .expect("FXAA stack should still transfer postprocess output");
        assert_eq!(
            output_transfer.produced_outputs,
            vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
        );

        let fxaa = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Fxaa)
            .expect("FXAA settings should declare the terminal anti-alias node");
        assert!(fxaa.enabled);
        assert_eq!(
            fxaa.required_inputs,
            vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
        );
        assert_eq!(
            fxaa.produced_outputs,
            vec![PostProcessGraphResourceNames::FINAL_COLOR.to_string()]
        );
        assert_eq!(fxaa.after, vec![PostProcessEffectKind::OutputTransfer]);

        let graph = stack.validated_graph();
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .expect("validated graph should keep final transfer");
        let fxaa_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::Fxaa)
            .expect("validated graph should keep enabled FXAA");
        assert!(output_index < fxaa_index);
    }

    #[test]
    fn smaa_terminal_anti_alias_routes_output_transfer_through_terminal_input() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings::default(),
                false,
                false,
                &AntiAliasSettings::smaa(),
            );

        let output_transfer = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
            .expect("SMAA stack should still transfer postprocess output");
        assert_eq!(
            output_transfer.produced_outputs,
            vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
        );

        let smaa = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Smaa)
            .expect("SMAA settings should declare the terminal anti-alias node");
        assert!(smaa.enabled);
        assert_eq!(
            smaa.required_inputs,
            vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
        );
        assert_eq!(
            smaa.produced_outputs,
            vec![PostProcessGraphResourceNames::FINAL_COLOR.to_string()]
        );
        assert_eq!(smaa.after, vec![PostProcessEffectKind::OutputTransfer]);
        assert!(!stack
            .effects
            .iter()
            .any(|effect| effect.kind == PostProcessEffectKind::Fxaa));

        let graph = stack.validated_graph();
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .expect("validated graph should keep final transfer");
        let smaa_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::Smaa)
            .expect("validated graph should keep enabled SMAA terminal pass");
        assert!(output_index < smaa_index);
    }

    #[test]
    fn dynamic_resolution_declares_upscale_before_output_transfer() {
        let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale(
            &Default::default(),
            &Default::default(),
            RenderExposureSettings::default(),
            &RenderPostProcessEffectStackSettings::default(),
            false,
            false,
            &AntiAliasSettings::off(),
            true,
        );

        let upscale = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Upscale)
            .expect("dynamic resolution should declare an explicit upscale node");
        assert_eq!(
            upscale.required_inputs,
            vec![PostProcessGraphResourceNames::TONEMAPPED.to_string()]
        );
        assert_eq!(
            upscale.produced_outputs,
            vec![PostProcessGraphResourceNames::UPSCALED.to_string()]
        );

        let output_transfer = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
            .expect("dynamic resolution stack should keep final transfer");
        assert_eq!(
            output_transfer.required_inputs,
            vec![PostProcessGraphResourceNames::UPSCALED.to_string()]
        );
        assert!(output_transfer
            .after
            .contains(&PostProcessEffectKind::Upscale));

        let graph = stack.validated_graph();
        let uber_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::Uber)
            .expect("validated graph should include the tonemap source");
        let upscale_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::Upscale)
            .expect("validated graph should keep upscale");
        let output_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .expect("validated graph should keep final transfer");
        assert!(uber_index < upscale_index);
        assert!(upscale_index < output_index);
    }

    #[test]
    fn screen_space_reflection_declares_specular_occlusion_and_resolve_inputs() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    screen_space_reflection: RenderScreenSpaceReflectionSettings {
                        intensity: 0.5,
                        max_steps: 32,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                false,
                false,
                &AntiAliasSettings::off(),
            );

        let specular_occlusion = stack
            .effects
            .iter()
            .find(|effect| {
                effect.kind == PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion
            })
            .expect("SSR should enable the screen-space reflection specular occlusion node");

        assert!(specular_occlusion
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(specular_occlusion
            .required_inputs
            .contains(&PostProcessGraphResourceNames::GBUFFER_MATERIAL.to_string()));
        assert!(specular_occlusion
            .required_inputs
            .contains(&PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string()));
        assert!(specular_occlusion.produced_outputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION.to_string()
        ));

        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::HZB_FURTHEST.to_string()));

        let reflection_pyramid = stack
            .effects
            .iter()
            .find(|effect| {
                effect.kind == PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid
            })
            .expect("SSR should enable the screen-space reflection reflection pyramid node");

        assert!(reflection_pyramid
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert!(reflection_pyramid.produced_outputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
        ));

        let reflection_pyramid_coarse = stack
            .effects
            .iter()
            .find(|effect| {
                effect.kind == PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse
            })
            .expect("SSR should enable the coarse screen-space reflection reflection pyramid node");

        assert!(reflection_pyramid_coarse.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
        ));
        assert!(reflection_pyramid_coarse.produced_outputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
                .to_string()
        ));
        assert!(reflection_pyramid_coarse
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));

        let resolve = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionResolve)
            .expect("SSR should enable the screen-space reflection resolve node");

        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::HZB_FURTHEST.to_string()));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
        ));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
                .to_string()
        ));
        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string()));
        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::GBUFFER_MATERIAL.to_string()));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION.to_string()
        ));
        assert!(!resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string()));
        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

        let scene_composite = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::SceneComposite)
            .expect("SSR should feed the scene composite node");
        assert!(scene_composite
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(scene_composite
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
        assert!(scene_composite
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionResolve));

        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Uber)
            .expect("SSR should keep an effect-stack color node");
        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
        assert!(!effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(!effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string()));
        assert!(!effect_stack
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    }

    #[test]
    fn screen_space_reflection_resolve_temporal_declares_history_and_motion_vector_inputs() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    screen_space_reflection: RenderScreenSpaceReflectionSettings {
                        intensity: 0.5,
                        max_steps: 32,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                true,
                true,
                &AntiAliasSettings::off(),
            );

        assert!(stack.initial_resources.contains(
            &PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string()
        ));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));

        let resolve = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionResolve)
            .expect("temporal SSR should enable the screen-space reflection resolve node");

        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string()
        ));
        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION.to_string()
        ));
        assert!(resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::HZB_FURTHEST.to_string()));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
        ));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
                .to_string()
        ));
    }

    #[test]
    fn taa_resolve_declares_history_velocity_and_output_transfer_input() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings::default(),
                true,
                true,
                &AntiAliasSettings::taa(),
            );

        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
        let taa_resolve = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::TaaResolve)
            .expect("TAA should enable a temporal resolve node");
        assert!(taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert!(taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
        assert!(taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string()));
        assert!(taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string()));
        assert!(taa_resolve
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string()));
        assert!(taa_resolve
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::TAA_HISTORY_CURRENT.to_string()));

        let output_transfer = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
            .expect("TAA stack should keep final composite");
        assert!(output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string()));
        assert!(!output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert!(output_transfer
            .after
            .contains(&PostProcessEffectKind::TaaResolve));

        let graph = stack.validated_graph();
        let taa_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::TaaResolve)
            .expect("validated graph should keep the TAA resolve node");
        let final_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .expect("validated graph should keep final composite");
        assert!(taa_index < final_index);
    }

    #[test]
    fn without_history_resources_disables_taa_and_restores_scene_color_input() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings::default(),
                true,
                true,
                &AntiAliasSettings::taa(),
            )
            .without_history_resources();
        let graph = stack.validated_graph();

        assert!(!graph
            .nodes
            .iter()
            .any(|node| node.kind == PostProcessEffectKind::TaaResolve));
        assert!(!stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string()));
        assert!(!stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
        assert!(!graph.nodes.iter().any(|node| node
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string())));

        let output_transfer = graph
            .nodes
            .iter()
            .find(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .expect("history-stripped stack should keep final composite");
        assert!(output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert!(!output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string()));
    }

    #[test]
    fn without_history_resources_keeps_scene_velocity_for_motion_blur() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    motion_blur: RenderMotionBlurSettings {
                        shutter_angle: 0.5,
                        samples: 2,
                    },
                    ..Default::default()
                },
                true,
                true,
                &AntiAliasSettings::taa(),
            )
            .without_history_resources();

        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
    }

    #[test]
    fn effect_stack_depth_of_field_feeds_uber_from_dedicated_intermediate() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    depth_of_field: RenderDepthOfFieldSettings {
                        aperture: 0.75,
                        max_blur_radius: 4.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                false,
                false,
                &AntiAliasSettings::off(),
            );

        let depth_of_field = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::DepthOfField)
            .expect("DoF should enable a dedicated depth-of-field node");
        assert!(depth_of_field
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert!(depth_of_field
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(depth_of_field
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));

        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Uber)
            .expect("DoF should enable the effect stack node");

        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));
        assert!(!effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert_eq!(
            effect_stack.produced_outputs,
            expected_uber_effect_stack_outputs()
        );

        let graph = stack.validated_graph();
        let depth_of_field_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::DepthOfField)
            .expect("validated graph should keep the dedicated DoF node");
        let uber_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::Uber)
            .expect("validated graph should keep the DoF effect-stack node");
        assert!(depth_of_field_index < uber_index);
    }

    #[test]
    fn effect_stack_blur_feeds_uber_from_dedicated_intermediate() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    blur: RenderBlurSettings { radius: 3.0 },
                    ..Default::default()
                },
                false,
                false,
                &AntiAliasSettings::off(),
            );

        let blur = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Blur)
            .expect("blur should enable a dedicated blur node");
        assert!(blur
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert!(blur
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::BLURRED.to_string()));

        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Uber)
            .expect("blur should keep the effect stack node for remaining stack work");
        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::BLURRED.to_string()));
        assert!(!effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert_eq!(
            effect_stack.produced_outputs,
            expected_uber_effect_stack_outputs()
        );

        let graph = stack.validated_graph();
        let blur_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::Blur)
            .expect("validated graph should keep the dedicated blur node");
        let uber_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::Uber)
            .expect("validated graph should keep the blur-fed effect-stack node");
        assert!(blur_index < uber_index);
    }

    #[test]
    fn screen_space_reflection_resolve_feeds_scene_composite_before_output_transfer() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    screen_space_reflection: RenderScreenSpaceReflectionSettings {
                        intensity: 0.5,
                        max_steps: 32,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                false,
                false,
                &AntiAliasSettings::off(),
            );

        let resolve = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionResolve)
            .expect("SSR should enable the screen-space reflection resolve node");

        assert!(resolve
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

        let scene_composite = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::SceneComposite)
            .expect("SSR should feed the scene composite node");
        assert!(scene_composite
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(scene_composite
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
        assert!(scene_composite
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionResolve));

        let output_transfer = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
            .expect("SSR should keep final composite node");
        assert!(!output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(output_transfer.after.contains(&PostProcessEffectKind::Uber));

        let graph = stack.validated_graph();
        let graph_resolve = graph
            .nodes
            .iter()
            .find(|node| node.kind == PostProcessEffectKind::ScreenSpaceReflectionResolve)
            .expect("validated graph should keep the SSR resolve node");
        assert!(graph_resolve
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(graph_resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
        assert!(graph_resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
        assert!(graph_resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

        let graph_composite = graph
            .nodes
            .iter()
            .find(|node| node.kind == PostProcessEffectKind::SceneComposite)
            .expect("validated graph should keep the scene composite node");
        assert!(graph_composite
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(graph_composite
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
        assert!(graph_composite
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionResolve));

        let graph_final = graph
            .nodes
            .iter()
            .find(|node| node.kind == PostProcessEffectKind::OutputTransfer)
            .expect("validated graph should keep the final composite node");
        assert!(!graph_final
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(graph_final.after.contains(&PostProcessEffectKind::Uber));
    }

    #[test]
    fn effect_stack_motion_blur_declares_depth_and_reconstructed_motion_vector_inputs() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    motion_blur: RenderMotionBlurSettings {
                        shutter_angle: 0.5,
                        samples: 2,
                    },
                    ..Default::default()
                },
                false,
                false,
                &AntiAliasSettings::off(),
            );

        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
        let motion_blur = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::MotionBlur)
            .expect("motion blur should enable a dedicated motion blur node");
        assert!(motion_blur
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
        assert!(motion_blur
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(motion_blur
            .required_inputs
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
        assert!(motion_blur
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::MOTION_BLURRED.to_string()));

        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Uber)
            .expect("motion blur should enable the effect stack node");

        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::MOTION_BLURRED.to_string()));
        assert!(!effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(!effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
        assert_eq!(
            effect_stack.produced_outputs,
            expected_uber_effect_stack_outputs()
        );

        let graph = stack.validated_graph();
        let motion_blur_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::MotionBlur)
            .expect("validated graph should keep the dedicated motion blur node");
        let uber_index = graph
            .nodes
            .iter()
            .position(|node| node.kind == PostProcessEffectKind::Uber)
            .expect("validated graph should keep the motion-blur-fed effect-stack node");
        assert!(motion_blur_index < uber_index);
    }

    #[test]
    fn effect_stack_omits_depth_of_field_intermediate_outputs_when_dof_is_disabled() {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &Default::default(),
                &Default::default(),
                &RenderPostProcessEffectStackSettings {
                    vignette: RenderVignetteSettings {
                        intensity: 0.25,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                false,
                false,
                &AntiAliasSettings::off(),
            );

        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::Uber)
            .expect("vignette should enable the effect stack node");

        assert_eq!(
            effect_stack.produced_outputs,
            expected_uber_effect_stack_outputs()
        );
    }
}
