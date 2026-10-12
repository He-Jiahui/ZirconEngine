use std::fmt;

use super::super::RenderPipelinePhase;
use super::PostProcessEffectKind;

/// 旧后处理骨干链的稳定位置，用于把效果种类映射到计划执行器和 ViewFamily 阶段。
/// 图编译仍以资源依赖与阶段校验为准；调用方不能仅凭相邻 slot 推断纹理尺寸或颜色空间。
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
#[path = "tests/chain.rs"]
mod tests;
