use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PostProcessEffectKind {
    Bloom,
    ColorGrading,
    HistoryResolve,
    FinalComposite,
}

impl PostProcessEffectKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Bloom => "bloom",
            Self::ColorGrading => "color-grading",
            Self::HistoryResolve => "history-resolve",
            Self::FinalComposite => "final-composite",
        }
    }
}

impl fmt::Display for PostProcessEffectKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
