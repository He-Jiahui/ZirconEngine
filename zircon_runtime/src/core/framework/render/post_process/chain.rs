use std::fmt;

use super::PostProcessEffectKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PostProcessChainSlot {
    TaaResolve,
    DepthOfField,
    MotionBlur,
    Bloom,
    ExposureHistogram,
    ExposureResolve,
    SceneComposite,
    Blur,
    ColorLutBake,
    Uber,
    TerminalAntiAlias,
    Upscale,
    OutputTransfer,
}

impl PostProcessChainSlot {
    pub const BACKBONE: [Self; 13] = [
        Self::TaaResolve,
        Self::DepthOfField,
        Self::MotionBlur,
        Self::Bloom,
        Self::ExposureHistogram,
        Self::ExposureResolve,
        Self::SceneComposite,
        Self::Blur,
        Self::ColorLutBake,
        Self::Uber,
        Self::TerminalAntiAlias,
        Self::Upscale,
        Self::OutputTransfer,
    ];

    pub const fn fixed_backbone() -> &'static [Self] {
        &Self::BACKBONE
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::TaaResolve => "taa-resolve",
            Self::DepthOfField => "depth-of-field",
            Self::MotionBlur => "motion-blur",
            Self::Bloom => "bloom",
            Self::ExposureHistogram => "exposure-histogram",
            Self::ExposureResolve => "exposure-resolve",
            Self::SceneComposite => "scene-composite",
            Self::Blur => "blur",
            Self::ColorLutBake => "color-lut-bake",
            Self::Uber => "uber",
            Self::TerminalAntiAlias => "terminal-anti-alias",
            Self::Upscale => "upscale",
            Self::OutputTransfer => "output-transfer",
        }
    }

    pub const fn planned_executor_id(self) -> &'static str {
        match self {
            Self::TaaResolve => "temporal.taa-resolve",
            Self::DepthOfField => "post.depth-of-field",
            Self::MotionBlur => "post.motion-blur",
            Self::Bloom => "post.bloom",
            Self::ExposureHistogram => "post.exposure.histogram",
            Self::ExposureResolve => "post.exposure.resolve",
            Self::SceneComposite => "post.scene-composite",
            Self::Blur => "post.blur",
            Self::ColorLutBake => "post.color-lut-bake",
            Self::Uber => "post.uber",
            Self::TerminalAntiAlias => "post.terminal-aa",
            Self::Upscale => "post.upscale",
            Self::OutputTransfer => "post.output-transfer",
        }
    }

    pub const fn from_current_effect_kind(kind: PostProcessEffectKind) -> Self {
        match kind {
            PostProcessEffectKind::TaaResolve => Self::TaaResolve,
            PostProcessEffectKind::DepthOfField => Self::DepthOfField,
            PostProcessEffectKind::MotionBlur => Self::MotionBlur,
            PostProcessEffectKind::Bloom => Self::Bloom,
            PostProcessEffectKind::ExposureHistogram => Self::ExposureHistogram,
            PostProcessEffectKind::ExposureResolve => Self::ExposureResolve,
            PostProcessEffectKind::SceneComposite => Self::SceneComposite,
            PostProcessEffectKind::Blur => Self::Blur,
            PostProcessEffectKind::ColorLutBake => Self::ColorLutBake,
            PostProcessEffectKind::Uber => Self::Uber,
            PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid
            | PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse
            | PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion
            | PostProcessEffectKind::ScreenSpaceReflectionResolve => Self::SceneComposite,
            PostProcessEffectKind::Upscale => Self::Upscale,
            PostProcessEffectKind::OutputTransfer => Self::OutputTransfer,
            PostProcessEffectKind::Fxaa | PostProcessEffectKind::Smaa => Self::TerminalAntiAlias,
        }
    }
}

impl fmt::Display for PostProcessChainSlot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::{PostProcessChainSlot, PostProcessEffectKind};

    #[test]
    fn render_post_chain_backbone_order_is_stable() {
        let labels = PostProcessChainSlot::fixed_backbone()
            .iter()
            .map(|slot| slot.label())
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec![
                "taa-resolve",
                "depth-of-field",
                "motion-blur",
                "bloom",
                "exposure-histogram",
                "exposure-resolve",
                "scene-composite",
                "blur",
                "color-lut-bake",
                "uber",
                "terminal-anti-alias",
                "upscale",
                "output-transfer",
            ]
        );
    }

    #[test]
    fn render_post_chain_current_effect_kinds_have_migration_slots() {
        let mappings = [
            (
                PostProcessEffectKind::TaaResolve,
                PostProcessChainSlot::TaaResolve,
            ),
            (PostProcessEffectKind::Bloom, PostProcessChainSlot::Bloom),
            (
                PostProcessEffectKind::DepthOfField,
                PostProcessChainSlot::DepthOfField,
            ),
            (
                PostProcessEffectKind::MotionBlur,
                PostProcessChainSlot::MotionBlur,
            ),
            (
                PostProcessEffectKind::ExposureHistogram,
                PostProcessChainSlot::ExposureHistogram,
            ),
            (
                PostProcessEffectKind::ExposureResolve,
                PostProcessChainSlot::ExposureResolve,
            ),
            (
                PostProcessEffectKind::SceneComposite,
                PostProcessChainSlot::SceneComposite,
            ),
            (PostProcessEffectKind::Blur, PostProcessChainSlot::Blur),
            (
                PostProcessEffectKind::ColorLutBake,
                PostProcessChainSlot::ColorLutBake,
            ),
            (PostProcessEffectKind::Uber, PostProcessChainSlot::Uber),
            (
                PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid,
                PostProcessChainSlot::SceneComposite,
            ),
            (
                PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse,
                PostProcessChainSlot::SceneComposite,
            ),
            (
                PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion,
                PostProcessChainSlot::SceneComposite,
            ),
            (
                PostProcessEffectKind::ScreenSpaceReflectionResolve,
                PostProcessChainSlot::SceneComposite,
            ),
            (
                PostProcessEffectKind::Upscale,
                PostProcessChainSlot::Upscale,
            ),
            (
                PostProcessEffectKind::OutputTransfer,
                PostProcessChainSlot::OutputTransfer,
            ),
            (
                PostProcessEffectKind::Fxaa,
                PostProcessChainSlot::TerminalAntiAlias,
            ),
            (
                PostProcessEffectKind::Smaa,
                PostProcessChainSlot::TerminalAntiAlias,
            ),
        ];

        for (kind, expected_slot) in mappings {
            assert_eq!(
                PostProcessChainSlot::from_current_effect_kind(kind),
                expected_slot,
                "{kind} should be assigned to the PP-M1 migration slot"
            );
        }
    }

    #[test]
    fn render_post_chain_planned_executor_ids_are_stable() {
        let executor_ids = PostProcessChainSlot::fixed_backbone()
            .iter()
            .map(|slot| slot.planned_executor_id())
            .collect::<Vec<_>>();

        assert_eq!(
            executor_ids,
            vec![
                "temporal.taa-resolve",
                "post.depth-of-field",
                "post.motion-blur",
                "post.bloom",
                "post.exposure.histogram",
                "post.exposure.resolve",
                "post.scene-composite",
                "post.blur",
                "post.color-lut-bake",
                "post.uber",
                "post.terminal-aa",
                "post.upscale",
                "post.output-transfer",
            ]
        );
    }
}
