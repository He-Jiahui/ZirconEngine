#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderHybridGiFallbackReason {
    BakedLightingUnavailable,
}

impl RenderHybridGiFallbackReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::BakedLightingUnavailable => "baked-lighting-unavailable",
        }
    }
}
