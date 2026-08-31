use std::fmt;

use super::super::RenderPipelinePhase;
use super::PostProcessEffectKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PostProcessChainSlot {
    DepthOfField,
    TaaResolve,
    MotionBlur,
    Bloom,
    ExposureHistogram,
    ExposureResolve,
    SceneComposite,
    Blur,
    ColorLutBake,
    Uber,
    TerminalAntiAlias,
    PrimaryUpscale,
    SecondaryUpscale,
    OutputTransfer,
}

impl PostProcessChainSlot {
    pub const BACKBONE: [Self; 14] = [
        Self::DepthOfField,
        Self::TaaResolve,
        Self::MotionBlur,
        Self::Bloom,
        Self::ExposureHistogram,
        Self::ExposureResolve,
        Self::SceneComposite,
        Self::Blur,
        Self::ColorLutBake,
        Self::Uber,
        Self::TerminalAntiAlias,
        Self::PrimaryUpscale,
        Self::SecondaryUpscale,
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
            Self::PrimaryUpscale => "primary-upscale",
            Self::SecondaryUpscale => "secondary-upscale",
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
            Self::PrimaryUpscale => "post.primary-upscale",
            Self::SecondaryUpscale => "post.secondary-upscale",
            Self::OutputTransfer => "post.output-transfer",
        }
    }

    /// Classifies the compatibility slot by the canonical view-family phase.
    ///
    /// The old backbone remains available while graph compilation migrates to phase-based
    /// scheduling. This prevents individual executors from inferring colour-space or upscale
    /// order from neighbouring list entries.
    pub const fn pipeline_phase(self) -> RenderPipelinePhase {
        match self {
            Self::DepthOfField => RenderPipelinePhase::PreReconstructionScenePostProcess,
            Self::TaaResolve => RenderPipelinePhase::TemporalReconstruction,
            Self::MotionBlur
            | Self::Bloom
            | Self::ExposureHistogram
            | Self::ExposureResolve
            | Self::SceneComposite
            | Self::Blur => RenderPipelinePhase::PostReconstructionScenePostProcess,
            Self::ColorLutBake | Self::Uber => RenderPipelinePhase::DisplayMapping,
            Self::TerminalAntiAlias => RenderPipelinePhase::DisplayPostProcess,
            Self::PrimaryUpscale => RenderPipelinePhase::PrimarySpatialUpscale,
            Self::SecondaryUpscale => RenderPipelinePhase::SecondarySpatialUpscale,
            Self::OutputTransfer => RenderPipelinePhase::OutputTransform,
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
            PostProcessEffectKind::PrimaryUpscale => Self::PrimaryUpscale,
            PostProcessEffectKind::SecondaryUpscale => Self::SecondaryUpscale,
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
    use super::{PostProcessChainSlot, PostProcessEffectKind, RenderPipelinePhase};

    #[test]
    fn render_post_chain_backbone_order_is_stable() {
        let labels = PostProcessChainSlot::fixed_backbone()
            .iter()
            .map(|slot| slot.label())
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec![
                "depth-of-field",
                "taa-resolve",
                "motion-blur",
                "bloom",
                "exposure-histogram",
                "exposure-resolve",
                "scene-composite",
                "blur",
                "color-lut-bake",
                "uber",
                "terminal-anti-alias",
                "primary-upscale",
                "secondary-upscale",
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
                PostProcessEffectKind::PrimaryUpscale,
                PostProcessChainSlot::PrimaryUpscale,
            ),
            (
                PostProcessEffectKind::SecondaryUpscale,
                PostProcessChainSlot::SecondaryUpscale,
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
                "post.depth-of-field",
                "temporal.taa-resolve",
                "post.motion-blur",
                "post.bloom",
                "post.exposure.histogram",
                "post.exposure.resolve",
                "post.scene-composite",
                "post.blur",
                "post.color-lut-bake",
                "post.uber",
                "post.terminal-aa",
                "post.primary-upscale",
                "post.secondary-upscale",
                "post.output-transfer",
            ]
        );
    }

    #[test]
    fn render_post_chain_slots_map_to_view_family_phases() {
        let mappings = [
            (
                PostProcessChainSlot::DepthOfField,
                RenderPipelinePhase::PreReconstructionScenePostProcess,
            ),
            (
                PostProcessChainSlot::TaaResolve,
                RenderPipelinePhase::TemporalReconstruction,
            ),
            (
                PostProcessChainSlot::MotionBlur,
                RenderPipelinePhase::PostReconstructionScenePostProcess,
            ),
            (
                PostProcessChainSlot::Bloom,
                RenderPipelinePhase::PostReconstructionScenePostProcess,
            ),
            (
                PostProcessChainSlot::ExposureHistogram,
                RenderPipelinePhase::PostReconstructionScenePostProcess,
            ),
            (
                PostProcessChainSlot::ExposureResolve,
                RenderPipelinePhase::PostReconstructionScenePostProcess,
            ),
            (
                PostProcessChainSlot::SceneComposite,
                RenderPipelinePhase::PostReconstructionScenePostProcess,
            ),
            (
                PostProcessChainSlot::Blur,
                RenderPipelinePhase::PostReconstructionScenePostProcess,
            ),
            (
                PostProcessChainSlot::ColorLutBake,
                RenderPipelinePhase::DisplayMapping,
            ),
            (
                PostProcessChainSlot::Uber,
                RenderPipelinePhase::DisplayMapping,
            ),
            (
                PostProcessChainSlot::TerminalAntiAlias,
                RenderPipelinePhase::DisplayPostProcess,
            ),
            (
                PostProcessChainSlot::PrimaryUpscale,
                RenderPipelinePhase::PrimarySpatialUpscale,
            ),
            (
                PostProcessChainSlot::SecondaryUpscale,
                RenderPipelinePhase::SecondarySpatialUpscale,
            ),
            (
                PostProcessChainSlot::OutputTransfer,
                RenderPipelinePhase::OutputTransform,
            ),
        ];

        for (slot, expected_phase) in mappings {
            assert_eq!(slot.pipeline_phase(), expected_phase, "{slot}");
        }
    }
}
