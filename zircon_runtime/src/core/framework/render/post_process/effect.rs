use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PostProcessEffectKind {
    Blur,
    Bloom,
    ColorLutBake,
    DepthOfField,
    ExposureHistogram,
    ExposureResolve,
    MotionBlur,
    SceneComposite,
    TaaResolve,
    Uber,
    ScreenSpaceReflectionReflectionPyramid,
    ScreenSpaceReflectionReflectionPyramidCoarse,
    ScreenSpaceReflectionSpecularOcclusion,
    ScreenSpaceReflectionResolve,
    Upscale,
    OutputTransfer,
    Fxaa,
    Smaa,
}

impl PostProcessEffectKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Blur => "blur",
            Self::Bloom => "bloom",
            Self::ColorLutBake => "color-lut-bake",
            Self::DepthOfField => "depth-of-field",
            Self::ExposureHistogram => "exposure-histogram",
            Self::ExposureResolve => "exposure-resolve",
            Self::MotionBlur => "motion-blur",
            Self::SceneComposite => "scene-composite",
            Self::TaaResolve => "taa-resolve",
            Self::Uber => "uber",
            Self::ScreenSpaceReflectionReflectionPyramid => {
                "screen-space-reflection-reflection-pyramid"
            }
            Self::ScreenSpaceReflectionReflectionPyramidCoarse => {
                "screen-space-reflection-reflection-pyramid-coarse"
            }
            Self::ScreenSpaceReflectionSpecularOcclusion => {
                "screen-space-reflection-specular-occlusion"
            }
            Self::ScreenSpaceReflectionResolve => "screen-space-reflection-resolve",
            Self::Upscale => "upscale",
            Self::OutputTransfer => "output-transfer",
            Self::Fxaa => "fxaa",
            Self::Smaa => "smaa",
        }
    }
}

impl fmt::Display for PostProcessEffectKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostProcessEffectSettings {
    pub kind: PostProcessEffectKind,
    pub enabled: bool,
    pub required_inputs: Vec<String>,
    pub produced_outputs: Vec<String>,
    pub after: Vec<PostProcessEffectKind>,
}

impl PostProcessEffectSettings {
    pub fn new(kind: PostProcessEffectKind) -> Self {
        Self {
            kind,
            enabled: true,
            required_inputs: Vec::new(),
            produced_outputs: Vec::new(),
            after: Vec::new(),
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_required_inputs(
        mut self,
        resources: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.required_inputs = resources.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_produced_outputs(
        mut self,
        resources: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.produced_outputs = resources.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_after(
        mut self,
        dependencies: impl IntoIterator<Item = PostProcessEffectKind>,
    ) -> Self {
        self.after = dependencies.into_iter().collect();
        self
    }
}
