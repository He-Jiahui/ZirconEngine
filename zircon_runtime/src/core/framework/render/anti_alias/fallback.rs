use super::{AntiAliasMode, TaaQualityPreset};

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
    pub taa_quality: TaaQualityPreset,
    pub reason: Option<AntiAliasFallbackReason>,
}

impl Default for AntiAliasFallbackReport {
    fn default() -> Self {
        Self::exact(AntiAliasMode::Off)
    }
}

impl AntiAliasFallbackReport {
    pub const fn exact(mode: AntiAliasMode) -> Self {
        Self::exact_with_taa_quality(mode, TaaQualityPreset::Medium)
    }

    pub const fn exact_with_taa_quality(
        mode: AntiAliasMode,
        taa_quality: TaaQualityPreset,
    ) -> Self {
        Self {
            requested_mode: mode,
            effective_mode: mode,
            taa_quality,
            reason: None,
        }
    }

    pub const fn fallback(
        requested_mode: AntiAliasMode,
        effective_mode: AntiAliasMode,
        reason: AntiAliasFallbackReason,
    ) -> Self {
        Self::fallback_with_taa_quality(
            requested_mode,
            effective_mode,
            TaaQualityPreset::Medium,
            reason,
        )
    }

    pub const fn fallback_with_taa_quality(
        requested_mode: AntiAliasMode,
        effective_mode: AntiAliasMode,
        taa_quality: TaaQualityPreset,
        reason: AntiAliasFallbackReason,
    ) -> Self {
        Self {
            requested_mode,
            effective_mode,
            taa_quality,
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

    pub const fn requested_graph_sample_count(self) -> u32 {
        self.requested_mode.graph_sample_count()
    }

    pub const fn effective_graph_sample_count(self) -> u32 {
        self.effective_mode.graph_sample_count()
    }
}
