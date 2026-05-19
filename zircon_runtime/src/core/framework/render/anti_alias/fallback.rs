use super::AntiAliasMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AntiAliasFallbackReason {
    AutoResolvedToFxaa,
    UnsupportedFxaa,
    UnsupportedSmaa,
    UnsupportedCas,
    UnsupportedDlss,
    UnsupportedTaa,
    UnsupportedMsaaSampleCount,
    MissingHistory,
}

impl AntiAliasFallbackReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::AutoResolvedToFxaa => "auto-resolved-to-fxaa",
            Self::UnsupportedFxaa => "unsupported-fxaa",
            Self::UnsupportedSmaa => "unsupported-smaa",
            Self::UnsupportedCas => "unsupported-cas",
            Self::UnsupportedDlss => "unsupported-dlss",
            Self::UnsupportedTaa => "unsupported-taa",
            Self::UnsupportedMsaaSampleCount => "unsupported-msaa-sample-count",
            Self::MissingHistory => "missing-history",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AntiAliasFallbackReport {
    pub requested_mode: AntiAliasMode,
    pub effective_mode: AntiAliasMode,
    pub reason: Option<AntiAliasFallbackReason>,
}

impl Default for AntiAliasFallbackReport {
    fn default() -> Self {
        Self::exact(AntiAliasMode::Off)
    }
}

impl AntiAliasFallbackReport {
    pub const fn exact(mode: AntiAliasMode) -> Self {
        Self {
            requested_mode: mode,
            effective_mode: mode,
            reason: None,
        }
    }

    pub const fn fallback(
        requested_mode: AntiAliasMode,
        effective_mode: AntiAliasMode,
        reason: AntiAliasFallbackReason,
    ) -> Self {
        Self {
            requested_mode,
            effective_mode,
            reason: Some(reason),
        }
    }

    pub const fn effective_mode_label(self) -> &'static str {
        self.effective_mode.label()
    }

    pub const fn fallback_reason_label(self) -> Option<&'static str> {
        match self.reason {
            Some(reason) => Some(reason.label()),
            None => None,
        }
    }
}
