use crate::core::framework::render::{
    AntiAliasMode, AntiAliasSettings, RenderBloomSettings, RenderColorGradingSettings,
};

use super::{
    PostProcessEffectKind, PostProcessEffectSettings, PostProcessPassGraph,
    RenderPostProcessEffectStackSettings,
};

pub struct PostProcessGraphResourceNames;

impl PostProcessGraphResourceNames {
    pub const SCENE_COLOR: &'static str = "scene-color";
    pub const SCENE_DEPTH: &'static str = "scene-depth";
    pub const SHADOW_MAP: &'static str = "shadow-map";
    pub const SCENE_MOTION_VECTOR: &'static str = "scene-motion-vector";
    pub const MOTION_VECTOR_TILE_MAX: &'static str = "postprocess.motion-vector.tile-max";
    pub const MOTION_VECTOR_TILE_MAX_COARSE: &'static str =
        "postprocess.motion-vector.tile-max.coarse";
    pub const MOTION_VECTOR_NEIGHBOR_MAX: &'static str = "postprocess.motion-vector.neighbor-max";
    pub const GBUFFER_ALBEDO: &'static str = "gbuffer-albedo";
    pub const GBUFFER_NORMAL: &'static str = "gbuffer-normal";
    pub const GBUFFER_MATERIAL: &'static str = "gbuffer-material";
    pub const AMBIENT_OCCLUSION: &'static str = "ambient-occlusion";
    pub const GLOBAL_ILLUMINATION: &'static str = "global-illumination";
    pub const LIGHT_LIST: &'static str = "light-list";
    // Temporal resources use distinct names so a pass cannot silently read and overwrite the same history slot.
    pub const HISTORY_PREVIOUS_SCENE_COLOR: &'static str = "history.previous.scene-color";
    pub const HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION: &'static str =
        "history.previous.screen-space-reflection";
    pub const HISTORY_CURRENT_SCENE_COLOR: &'static str = "history.current.scene-color";
    pub const HISTORY_OUTPUT_SCENE_COLOR: &'static str = "postprocess.history-resolved";
    pub const BLOOM: &'static str = "bloom-texture";
    pub const COLOR_GRADED: &'static str = "postprocess.color-graded";
    pub const EFFECT_STACKED: &'static str = "postprocess.effect-stacked";
    pub const DEPTH_OF_FIELD_COC: &'static str = "postprocess.depth-of-field.coc";
    pub const DEPTH_OF_FIELD_BOKEH: &'static str = "postprocess.depth-of-field.bokeh";
    pub const SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID: &'static str =
        "postprocess.screen-space-reflection.depth-pyramid";
    pub const SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE: &'static str =
        "postprocess.screen-space-reflection.depth-pyramid.coarse";
    pub const SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID: &'static str =
        "postprocess.screen-space-reflection.reflection-pyramid";
    pub const SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE: &'static str =
        "postprocess.screen-space-reflection.reflection-pyramid.coarse";
    pub const SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION: &'static str =
        "postprocess.screen-space-reflection.specular-occlusion";
    pub const SCREEN_SPACE_REFLECTION_HISTORY: &'static str =
        "postprocess.screen-space-reflection.history";
    pub const FINAL_COMPOSITED: &'static str = "postprocess.final-composited";
    pub const FINAL_COLOR: &'static str = "final-color";
    pub const VIEWPORT_OUTPUT: &'static str = "viewport-output";
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
        history_resolve_enabled: bool,
        history_available: bool,
    ) -> Self {
        Self::from_extract_settings_with_anti_alias(
            bloom,
            color_grading,
            history_resolve_enabled,
            history_available,
            &AntiAliasSettings::off(),
        )
    }

    pub fn from_extract_settings_with_anti_alias(
        bloom: &RenderBloomSettings,
        color_grading: &RenderColorGradingSettings,
        history_resolve_enabled: bool,
        history_available: bool,
        anti_alias: &AntiAliasSettings,
    ) -> Self {
        Self::from_extract_settings_with_effect_stack_and_anti_alias(
            bloom,
            color_grading,
            &RenderPostProcessEffectStackSettings::default(),
            history_resolve_enabled,
            history_available,
            anti_alias,
        )
    }

    pub fn from_extract_settings_with_effect_stack_and_anti_alias(
        bloom: &RenderBloomSettings,
        color_grading: &RenderColorGradingSettings,
        effect_stack: &RenderPostProcessEffectStackSettings,
        history_resolve_enabled: bool,
        history_available: bool,
        anti_alias: &AntiAliasSettings,
    ) -> Self {
        let bloom_enabled = bloom.intensity > 0.0;
        let color_grading_enabled = *color_grading != RenderColorGradingSettings::default();
        let effect_stack_enabled = effect_stack.is_enabled();
        let history_enabled = history_resolve_enabled && history_available;
        let fxaa_enabled = anti_alias.mode == AntiAliasMode::Fxaa;
        let ssr_enabled = effect_stack.screen_space_reflection.is_enabled();
        let ssr_temporal_enabled = ssr_enabled && history_enabled;
        let motion_vector_effects_enabled = effect_stack.motion_blur.is_enabled() || ssr_enabled;
        let mut initial_resources = vec![
            PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            PostProcessGraphResourceNames::SCENE_DEPTH.to_string(),
        ];
        if history_available {
            initial_resources
                .push(PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR.to_string());
        }
        if ssr_temporal_enabled {
            initial_resources.push(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string(),
            );
        }
        if ssr_enabled {
            initial_resources.push(PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string());
            initial_resources.push(PostProcessGraphResourceNames::GBUFFER_MATERIAL.to_string());
            initial_resources.push(PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string());
        }
        if motion_vector_effects_enabled {
            initial_resources.push(PostProcessGraphResourceNames::SCENE_MOTION_VECTOR.to_string());
            initial_resources
                .push(PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX.to_string());
            initial_resources
                .push(PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE.to_string());
            initial_resources
                .push(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string());
        }

        let mut final_inputs = vec![PostProcessGraphResourceNames::SCENE_COLOR.to_string()];
        let mut final_after = Vec::new();
        if bloom_enabled {
            final_inputs.push(PostProcessGraphResourceNames::BLOOM.to_string());
            final_after.push(PostProcessEffectKind::Bloom);
        }
        if color_grading_enabled {
            final_inputs.push(PostProcessGraphResourceNames::COLOR_GRADED.to_string());
            final_after.push(PostProcessEffectKind::ColorGrading);
        }
        if history_enabled {
            final_inputs
                .push(PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR.to_string());
            final_after.push(PostProcessEffectKind::HistoryResolve);
        }
        let mut effect_stack_inputs = final_inputs.clone();
        if effect_stack_requires_scene_depth(effect_stack)
            && !effect_stack_inputs
                .iter()
                .any(|resource| resource.as_str() == PostProcessGraphResourceNames::SCENE_DEPTH)
        {
            effect_stack_inputs.push(PostProcessGraphResourceNames::SCENE_DEPTH.to_string());
        }
        if effect_stack.motion_blur.is_enabled()
            && !effect_stack_inputs.iter().any(|resource| {
                resource.as_str() == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
            })
        {
            effect_stack_inputs
                .push(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string());
        }
        let effect_stack_after = final_after.clone();
        if effect_stack_enabled {
            final_inputs = vec![PostProcessGraphResourceNames::EFFECT_STACKED.to_string()];
            final_after = vec![PostProcessEffectKind::EffectStack];
        }
        if ssr_enabled {
            final_inputs
                .push(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string());
            final_after.push(PostProcessEffectKind::ScreenSpaceReflectionResolve);
        }
        let final_composite_output = if fxaa_enabled {
            PostProcessGraphResourceNames::FINAL_COMPOSITED
        } else {
            PostProcessGraphResourceNames::FINAL_COLOR
        };
        let color_grading_after = if bloom_enabled {
            vec![PostProcessEffectKind::Bloom]
        } else {
            Vec::new()
        };
        let history_after = if color_grading_enabled {
            vec![PostProcessEffectKind::ColorGrading]
        } else if bloom_enabled {
            vec![PostProcessEffectKind::Bloom]
        } else {
            Vec::new()
        };

        let mut effects = vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::Bloom)
                .with_enabled(bloom_enabled)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::BLOOM]),
            PostProcessEffectSettings::new(PostProcessEffectKind::ColorGrading)
                .with_enabled(color_grading_enabled)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::COLOR_GRADED])
                .with_after(color_grading_after),
            PostProcessEffectSettings::new(PostProcessEffectKind::HistoryResolve)
                .with_enabled(history_enabled)
                .with_required_inputs([
                    PostProcessGraphResourceNames::SCENE_COLOR,
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR,
                ])
                .with_produced_outputs([PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR])
                .with_after(history_after),
        ];
        if effect_stack_enabled {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::EffectStack)
                    .with_required_inputs(effect_stack_inputs)
                    .with_produced_outputs(effect_stack_outputs(effect_stack))
                    .with_after(effect_stack_after.clone()),
            );
        }
        if ssr_enabled {
            effects.push(
                PostProcessEffectSettings::new(
                    PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid,
                )
                .with_required_inputs(screen_space_reflection_depth_pyramid_inputs())
                .with_produced_outputs([
                    PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID,
                ])
                .with_after(effect_stack_after.clone()),
            );
            effects.push(
                PostProcessEffectSettings::new(
                    PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid,
                )
                .with_required_inputs(screen_space_reflection_reflection_pyramid_inputs())
                .with_produced_outputs([
                    PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
                ])
                .with_after(effect_stack_after.clone()),
            );
            effects.push(
                PostProcessEffectSettings::new(
                    PostProcessEffectKind::ScreenSpaceReflectionDepthPyramidCoarse,
                )
                .with_required_inputs(screen_space_reflection_depth_pyramid_coarse_inputs())
                .with_produced_outputs([
                    PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE,
                ])
                .with_after([PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid]),
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
                .with_after(effect_stack_after.clone()),
            );
            let mut resolve_after = effect_stack_after.clone();
            resolve_after.push(PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid);
            resolve_after.push(PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid);
            resolve_after.push(PostProcessEffectKind::ScreenSpaceReflectionDepthPyramidCoarse);
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
        effects.push(
            PostProcessEffectSettings::new(PostProcessEffectKind::FinalComposite)
                .with_required_inputs(final_inputs)
                .with_produced_outputs([final_composite_output])
                .with_after(final_after),
        );
        if anti_alias.mode != AntiAliasMode::Off {
            effects.push(
                PostProcessEffectSettings::new(PostProcessEffectKind::Fxaa)
                    .with_enabled(fxaa_enabled)
                    .with_required_inputs([PostProcessGraphResourceNames::FINAL_COMPOSITED])
                    .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COLOR])
                    .with_after([PostProcessEffectKind::FinalComposite]),
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
            resource != PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR
                && resource
                    != PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
                && resource != PostProcessGraphResourceNames::HISTORY_CURRENT_SCENE_COLOR
        });
        for effect in &mut stack.effects {
            if effect.kind == PostProcessEffectKind::HistoryResolve {
                effect.enabled = false;
            }
            effect.required_inputs.retain(|resource| {
                resource != PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR
                    && resource
                        != PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
                    && resource != PostProcessGraphResourceNames::HISTORY_CURRENT_SCENE_COLOR
                    && resource != PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR
            });
            effect.produced_outputs.retain(|resource| {
                resource != PostProcessGraphResourceNames::HISTORY_CURRENT_SCENE_COLOR
                    && resource != PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR
            });
            effect
                .after
                .retain(|dependency| *dependency != PostProcessEffectKind::HistoryResolve);
        }
        stack
    }
}

fn effect_stack_requires_scene_depth(effect_stack: &RenderPostProcessEffectStackSettings) -> bool {
    effect_stack.depth_of_field.is_enabled()
        || effect_stack.motion_blur.is_enabled()
        || effect_stack.fog.density > 0.0
}

fn effect_stack_outputs(effect_stack: &RenderPostProcessEffectStackSettings) -> Vec<&'static str> {
    let mut outputs = vec![PostProcessGraphResourceNames::EFFECT_STACKED];
    if effect_stack.depth_of_field.is_enabled() {
        outputs.push(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC);
        outputs.push(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH);
    }
    outputs
}

fn screen_space_reflection_specular_occlusion_inputs() -> Vec<&'static str> {
    vec![
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
    ]
}

fn screen_space_reflection_depth_pyramid_inputs() -> Vec<&'static str> {
    vec![PostProcessGraphResourceNames::SCENE_DEPTH]
}

fn screen_space_reflection_reflection_pyramid_inputs() -> Vec<&'static str> {
    vec![PostProcessGraphResourceNames::SCENE_COLOR]
}

fn screen_space_reflection_depth_pyramid_coarse_inputs() -> Vec<&'static str> {
    vec![PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID]
}

fn screen_space_reflection_reflection_pyramid_coarse_inputs() -> Vec<&'static str> {
    vec![PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID]
}

fn screen_space_reflection_resolve_inputs(ssr_temporal_enabled: bool) -> Vec<&'static str> {
    let mut inputs = vec![
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE,
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
        AntiAliasSettings, PostProcessEffectKind, RenderDepthOfFieldSettings,
        RenderMotionBlurSettings, RenderPostProcessEffectStackSettings,
        RenderScreenSpaceReflectionSettings, RenderVignetteSettings,
    };

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

        let depth_pyramid = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid)
            .expect("SSR should enable the screen-space reflection depth pyramid node");

        assert!(depth_pyramid
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(depth_pyramid.produced_outputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID.to_string()
        ));

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

        let depth_pyramid_coarse = stack
            .effects
            .iter()
            .find(|effect| {
                effect.kind == PostProcessEffectKind::ScreenSpaceReflectionDepthPyramidCoarse
            })
            .expect("SSR should enable the coarse screen-space reflection depth pyramid node");

        assert!(depth_pyramid_coarse.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID.to_string()
        ));
        assert!(depth_pyramid_coarse.produced_outputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE
                .to_string()
        ));
        assert!(depth_pyramid_coarse
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid));

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
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID.to_string()
        ));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE
                .to_string()
        ));
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
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionDepthPyramidCoarse));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::EffectStack)
            .expect("SSR should keep an effect-stack color node");
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

        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR.to_string()));
        assert!(stack.initial_resources.contains(
            &PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string()
        ));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_MOTION_VECTOR.to_string()));
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
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID.to_string()
        ));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE
                .to_string()
        ));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
        ));
        assert!(resolve.required_inputs.contains(
            &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
                .to_string()
        ));
    }

    #[test]
    fn effect_stack_depth_of_field_declares_depth_and_intermediate_outputs() {
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

        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::EffectStack)
            .expect("DoF should enable the effect stack node");

        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert_eq!(
            effect_stack.produced_outputs,
            [
                PostProcessGraphResourceNames::EFFECT_STACKED,
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
        );

        let graph = stack.validated_graph();
        let graph_effect_stack = graph
            .nodes
            .iter()
            .find(|node| node.kind == PostProcessEffectKind::EffectStack)
            .expect("validated graph should keep the DoF effect-stack node");
        assert!(graph_effect_stack
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC.to_string()));
        assert!(graph_effect_stack
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH.to_string()));
    }

    #[test]
    fn screen_space_reflection_resolve_produces_history_for_final_composite() {
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
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionDepthPyramidCoarse));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
        assert!(resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

        let final_composite = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::FinalComposite)
            .expect("SSR should keep final composite node");
        assert!(final_composite
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
        assert!(final_composite
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionResolve));

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
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionDepthPyramid));
        assert!(graph_resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
        assert!(graph_resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionDepthPyramidCoarse));
        assert!(graph_resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
        assert!(graph_resolve
            .after
            .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

        let graph_final = graph
            .nodes
            .iter()
            .find(|node| node.kind == PostProcessEffectKind::FinalComposite)
            .expect("validated graph should keep the final composite node");
        assert!(graph_final
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
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
            .contains(&PostProcessGraphResourceNames::SCENE_MOTION_VECTOR.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE.to_string()));
        assert!(stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::EffectStack)
            .expect("motion blur should enable the effect stack node");

        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
        assert_eq!(
            effect_stack.produced_outputs,
            [PostProcessGraphResourceNames::EFFECT_STACKED]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
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
            .find(|effect| effect.kind == PostProcessEffectKind::EffectStack)
            .expect("vignette should enable the effect stack node");

        assert_eq!(
            effect_stack.produced_outputs,
            [PostProcessGraphResourceNames::EFFECT_STACKED]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
    }
}
