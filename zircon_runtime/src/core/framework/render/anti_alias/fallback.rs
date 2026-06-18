use super::{AntiAliasMode, TaaQualityPreset};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AntiAliasFallbackReason {
    AutoResolvedToFxaa,
    AutoResolvedToSmaa,
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
            Self::AutoResolvedToSmaa => "auto-resolved-to-smaa",
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
    pub requested_graph_sample_count: u32,
    pub effective_graph_sample_count: u32,
    pub graph_sample_count_normalized: bool,
    pub terminal_slot_normalized: bool,
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
        let graph_sample_count = mode.graph_sample_count();
        Self {
            requested_mode: mode,
            effective_mode: mode,
            taa_quality,
            reason: None,
            requested_graph_sample_count: graph_sample_count,
            effective_graph_sample_count: graph_sample_count,
            graph_sample_count_normalized: false,
            terminal_slot_normalized: false,
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
        let requested_graph_sample_count = requested_mode.graph_sample_count();
        let effective_graph_sample_count = effective_mode.graph_sample_count();
        Self {
            requested_mode,
            effective_mode,
            taa_quality,
            reason: Some(reason),
            requested_graph_sample_count,
            effective_graph_sample_count,
            graph_sample_count_normalized: requested_graph_sample_count
                != effective_graph_sample_count,
            terminal_slot_normalized: terminal_slot_normalized(requested_mode, effective_mode),
        }
    }

    pub const fn with_graph_sample_counts(
        mut self,
        requested_graph_sample_count: u32,
        effective_graph_sample_count: u32,
    ) -> Self {
        self.requested_graph_sample_count =
            normalize_graph_sample_count(requested_graph_sample_count);
        self.effective_graph_sample_count =
            normalize_graph_sample_count(effective_graph_sample_count);
        self.graph_sample_count_normalized =
            self.requested_graph_sample_count != self.effective_graph_sample_count;
        self
    }

    pub const fn with_terminal_slot_normalized(mut self, terminal_slot_normalized: bool) -> Self {
        self.terminal_slot_normalized = terminal_slot_normalized;
        self
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
        self.requested_graph_sample_count
    }

    pub const fn effective_graph_sample_count(self) -> u32 {
        self.effective_graph_sample_count
    }

    pub const fn normalization_count(self) -> usize {
        let graph_sample_count = if self.graph_sample_count_normalized {
            1
        } else {
            0
        };
        let terminal_slot = if self.terminal_slot_normalized { 1 } else { 0 };
        graph_sample_count + terminal_slot
    }

    pub const fn taa_msaa_conflict_normalized(self) -> bool {
        matches!(self.requested_mode, AntiAliasMode::Taa)
            && self.requested_graph_sample_count > 1
            && self.effective_graph_sample_count == 1
    }
}

const fn normalize_graph_sample_count(sample_count: u32) -> u32 {
    if sample_count > 1 {
        sample_count
    } else {
        1
    }
}

const fn terminal_slot_normalized(
    requested_mode: AntiAliasMode,
    effective_mode: AntiAliasMode,
) -> bool {
    match (requested_mode, effective_mode) {
        (AntiAliasMode::Smaa, AntiAliasMode::Smaa)
        | (AntiAliasMode::Cas, AntiAliasMode::Cas)
        | (AntiAliasMode::Dlss, AntiAliasMode::Dlss) => false,
        (AntiAliasMode::Smaa | AntiAliasMode::Cas | AntiAliasMode::Dlss, _) => true,
        _ => false,
    }
}
