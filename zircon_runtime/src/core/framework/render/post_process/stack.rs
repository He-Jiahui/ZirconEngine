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
    pub const GBUFFER_ALBEDO: &'static str = "gbuffer-albedo";
    pub const GBUFFER_NORMAL: &'static str = "gbuffer-normal";
    pub const AMBIENT_OCCLUSION: &'static str = "ambient-occlusion";
    pub const GLOBAL_ILLUMINATION: &'static str = "global-illumination";
    pub const LIGHT_LIST: &'static str = "light-list";
    // Temporal resources use distinct names so a pass cannot silently read and overwrite the same history slot.
    pub const HISTORY_PREVIOUS_SCENE_COLOR: &'static str = "history.previous.scene-color";
    pub const HISTORY_CURRENT_SCENE_COLOR: &'static str = "history.current.scene-color";
    pub const HISTORY_OUTPUT_SCENE_COLOR: &'static str = "postprocess.history-resolved";
    pub const BLOOM: &'static str = "bloom-texture";
    pub const COLOR_GRADED: &'static str = "postprocess.color-graded";
    pub const EFFECT_STACKED: &'static str = "postprocess.effect-stacked";
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
        let mut initial_resources = vec![
            PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            PostProcessGraphResourceNames::SCENE_DEPTH.to_string(),
        ];
        if history_available {
            initial_resources
                .push(PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR.to_string());
        }
        if effect_stack.screen_space_reflection.is_enabled() {
            initial_resources.push(PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string());
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
        if effect_stack.screen_space_reflection.is_enabled()
            && !effect_stack_inputs
                .iter()
                .any(|resource| resource.as_str() == PostProcessGraphResourceNames::GBUFFER_NORMAL)
        {
            effect_stack_inputs.push(PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string());
        }
        let effect_stack_after = final_after.clone();
        if effect_stack_enabled {
            final_inputs = vec![PostProcessGraphResourceNames::EFFECT_STACKED.to_string()];
            final_after = vec![PostProcessEffectKind::EffectStack];
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
                    .with_produced_outputs([PostProcessGraphResourceNames::EFFECT_STACKED])
                    .with_after(effect_stack_after),
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
                && resource != PostProcessGraphResourceNames::HISTORY_CURRENT_SCENE_COLOR
        });
        for effect in &mut stack.effects {
            if effect.kind == PostProcessEffectKind::HistoryResolve {
                effect.enabled = false;
            }
            effect.required_inputs.retain(|resource| {
                resource != PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR
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
        || effect_stack.screen_space_reflection.is_enabled()
        || effect_stack.fog.density > 0.0
}

#[cfg(test)]
mod tests {
    use super::{PostProcessGraphResourceNames, PostProcessStackDescriptor};
    use crate::core::framework::render::{
        AntiAliasSettings, PostProcessEffectKind, RenderPostProcessEffectStackSettings,
        RenderScreenSpaceReflectionSettings,
    };

    #[test]
    fn effect_stack_ssr_declares_depth_and_normal_inputs() {
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

        let effect_stack = stack
            .effects
            .iter()
            .find(|effect| effect.kind == PostProcessEffectKind::EffectStack)
            .expect("SSR should enable the effect stack node");

        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
        assert!(effect_stack
            .required_inputs
            .contains(&PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string()));
    }
}
