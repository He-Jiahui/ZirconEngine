use crate::core::framework::render::{RenderBloomSettings, RenderColorGradingSettings};

use super::{PostProcessEffectKind, PostProcessEffectSettings, PostProcessPassGraph};

pub struct PostProcessGraphResourceNames;

impl PostProcessGraphResourceNames {
    pub const SCENE_COLOR: &'static str = "scene-color";
    pub const SCENE_DEPTH: &'static str = "scene-depth";
    pub const HISTORY_COLOR: &'static str = "history-scene-color";
    pub const BLOOM: &'static str = "bloom-texture";
    pub const COLOR_GRADED: &'static str = "postprocess.color-graded";
    pub const HISTORY_RESOLVED: &'static str = "postprocess.history-resolved";
    pub const FINAL_COLOR: &'static str = "final-color";
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
        let bloom_enabled = bloom.intensity > 0.0;
        let color_grading_enabled = *color_grading != RenderColorGradingSettings::default();
        let history_enabled = history_resolve_enabled && history_available;
        let mut initial_resources = vec![
            PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            PostProcessGraphResourceNames::SCENE_DEPTH.to_string(),
        ];
        if history_available {
            initial_resources.push(PostProcessGraphResourceNames::HISTORY_COLOR.to_string());
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
            final_inputs.push(PostProcessGraphResourceNames::HISTORY_RESOLVED.to_string());
            final_after.push(PostProcessEffectKind::HistoryResolve);
        }
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

        Self {
            initial_resources,
            effects: vec![
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
                        PostProcessGraphResourceNames::HISTORY_COLOR,
                    ])
                    .with_produced_outputs([PostProcessGraphResourceNames::HISTORY_RESOLVED])
                    .with_after(history_after),
                PostProcessEffectSettings::new(PostProcessEffectKind::FinalComposite)
                    .with_required_inputs(final_inputs)
                    .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COLOR])
                    .with_after(final_after),
            ],
        }
    }

    pub fn validated_graph(&self) -> PostProcessPassGraph {
        PostProcessPassGraph::validate_stack(self)
            .expect("default post-process stack descriptor must validate")
    }

    pub fn without_history_resources(&self) -> Self {
        let mut stack = self.clone();
        stack
            .initial_resources
            .retain(|resource| resource != PostProcessGraphResourceNames::HISTORY_COLOR);
        for effect in &mut stack.effects {
            if effect.kind == PostProcessEffectKind::HistoryResolve {
                effect.enabled = false;
            }
            effect
                .required_inputs
                .retain(|resource| resource != PostProcessGraphResourceNames::HISTORY_COLOR);
            effect
                .required_inputs
                .retain(|resource| resource != PostProcessGraphResourceNames::HISTORY_RESOLVED);
            effect
                .after
                .retain(|dependency| *dependency != PostProcessEffectKind::HistoryResolve);
        }
        stack
    }
}
