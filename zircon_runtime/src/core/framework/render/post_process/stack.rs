use crate::core::framework::render::{
    AntiAliasMode, AntiAliasSettings, RenderBloomSettings, RenderColorGradingSettings,
    RenderExposureMode, RenderExposureSettings,
};

use super::{
    PostProcessEffectKind, PostProcessEffectSettings, PostProcessGraphResourceNames,
    PostProcessPassGraph, RenderPostProcessEffectStackSettings,
};

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
        } else if terminal_anti_alias_enabled {
            vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
        } else {
            final_inputs.clone()
        };
        let output_transfer_after = if upscale_required {
            vec![PostProcessEffectKind::Upscale]
        } else if let Some(terminal_anti_alias_effect) = terminal_anti_alias_effect {
            vec![terminal_anti_alias_effect]
        } else {
            final_after.clone()
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
        } else if upscale_required || terminal_anti_alias_enabled {
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
        if let Some(terminal_anti_alias_effect) = terminal_anti_alias_effect {
            effects.push(
                PostProcessEffectSettings::new(terminal_anti_alias_effect)
                    .with_required_inputs([PostProcessGraphResourceNames::TONEMAPPED])
                    .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COMPOSITED])
                    .with_after([PostProcessEffectKind::Uber]),
            );
        }
        if upscale_required {
            let upscale_input = if terminal_anti_alias_enabled {
                PostProcessGraphResourceNames::FINAL_COMPOSITED
            } else {
                PostProcessGraphResourceNames::TONEMAPPED
            };
            let upscale_after = if let Some(terminal_anti_alias_effect) = terminal_anti_alias_effect
            {
                vec![terminal_anti_alias_effect]
            } else {
                vec![PostProcessEffectKind::Uber]
            };
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::Upscale)
                    .with_required_inputs([upscale_input])
                    .with_produced_outputs([PostProcessGraphResourceNames::UPSCALED])
                    .with_after(upscale_after),
            );
        }
        effects.push(
            PostProcessEffectSettings::new(PostProcessEffectKind::OutputTransfer)
                .with_required_inputs(output_transfer_input)
                .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COLOR])
                .with_after(output_transfer_after),
        );

        Self {
            initial_resources,
            effects,
        }
    }

    pub fn validated_graph(&self) -> PostProcessPassGraph {
        PostProcessPassGraph::validate_stack(self)
            .expect("default post-process stack descriptor must validate")
    }

    pub fn with_hybrid_gi_lighting_input(mut self) -> Self {
        for required_resource in [
            PostProcessGraphResourceNames::HYBRID_GI_LIGHTING,
            PostProcessGraphResourceNames::SCENE_VELOCITY,
        ] {
            if !self
                .initial_resources
                .iter()
                .any(|resource| resource == required_resource)
            {
                self.initial_resources.push(required_resource.to_string());
            }
        }
        for effect in &mut self.effects {
            if effect.kind != PostProcessEffectKind::Uber {
                continue;
            }
            if !effect
                .required_inputs
                .iter()
                .any(|resource| resource == PostProcessGraphResourceNames::HYBRID_GI_LIGHTING)
            {
                effect
                    .required_inputs
                    .push(PostProcessGraphResourceNames::HYBRID_GI_LIGHTING.to_string());
            }
        }
        self
    }

    pub fn with_effect_disabled(mut self, kind: PostProcessEffectKind) -> Self {
        let disabled_outputs = self
            .effects
            .iter()
            .filter(|effect| effect.kind == kind)
            .flat_map(|effect| effect.produced_outputs.iter().cloned())
            .collect::<Vec<_>>();
        for effect in &mut self.effects {
            if effect.kind == kind {
                effect.enabled = false;
            }
            effect
                .required_inputs
                .retain(|resource| !disabled_outputs.contains(resource));
            effect.after.retain(|dependency| *dependency != kind);
        }
        self
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
        let needs_scene_velocity = stack
            .initial_resources
            .iter()
            .any(|resource| resource == PostProcessGraphResourceNames::HYBRID_GI_LIGHTING);
        if !needs_reconstructed_motion_vectors && !needs_scene_velocity {
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
mod tests;
