use crate::core::framework::render::RenderCapabilitySummary;

use super::{AntiAliasFallbackReason, AntiAliasFallbackReport, AntiAliasMode, TaaQualityPreset};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AntiAliasSettings {
    pub mode: AntiAliasMode,
    pub taa_quality: TaaQualityPreset,
}

impl Default for AntiAliasSettings {
    fn default() -> Self {
        Self::auto()
    }
}

impl AntiAliasSettings {
    pub const fn new(mode: AntiAliasMode) -> Self {
        Self {
            mode,
            taa_quality: TaaQualityPreset::Medium,
        }
    }

    pub const fn with_taa_quality(mut self, taa_quality: TaaQualityPreset) -> Self {
        self.taa_quality = taa_quality;
        self
    }

    pub const fn off() -> Self {
        Self::new(AntiAliasMode::Off)
    }

    pub const fn auto() -> Self {
        Self::new(AntiAliasMode::Auto)
    }

    pub const fn fxaa() -> Self {
        Self::new(AntiAliasMode::Fxaa)
    }

    pub const fn msaa(samples: u32) -> Self {
        Self::new(AntiAliasMode::Msaa { samples })
    }

    pub const fn taa() -> Self {
        Self::new(AntiAliasMode::Taa)
    }

    pub const fn taa_with_quality(taa_quality: TaaQualityPreset) -> Self {
        Self::taa().with_taa_quality(taa_quality)
    }

    pub const fn smaa() -> Self {
        Self::new(AntiAliasMode::Smaa)
    }

    pub const fn cas() -> Self {
        Self::new(AntiAliasMode::Cas)
    }

    pub const fn dlss() -> Self {
        Self::new(AntiAliasMode::Dlss)
    }

    pub const fn from_camera_msaa_samples(samples: u32) -> Self {
        if samples > 1 {
            Self::msaa(samples)
        } else {
            Self::auto()
        }
    }

    pub fn resolve(
        self,
        capabilities: &RenderCapabilitySummary,
        history_available: bool,
    ) -> AntiAliasFallbackReport {
        self.resolve_with_requested_graph_sample_count(
            capabilities,
            history_available,
            self.mode.graph_sample_count(),
        )
    }

    pub fn resolve_with_requested_graph_sample_count(
        self,
        capabilities: &RenderCapabilitySummary,
        history_available: bool,
        requested_graph_sample_count: u32,
    ) -> AntiAliasFallbackReport {
        let report = match self.mode {
            AntiAliasMode::Off => AntiAliasFallbackReport::exact(AntiAliasMode::Off),
            AntiAliasMode::Auto => resolve_auto(capabilities),
            AntiAliasMode::Fxaa => resolve_fxaa(capabilities),
            AntiAliasMode::Msaa { samples } => resolve_msaa(samples, capabilities),
            AntiAliasMode::Taa => resolve_taa(capabilities, history_available, self.taa_quality),
            AntiAliasMode::Smaa => resolve_smaa(capabilities),
            AntiAliasMode::Cas => fallback_to_screen_space(
                AntiAliasMode::Cas,
                AntiAliasFallbackReason::UnsupportedCas,
                capabilities,
            ),
            AntiAliasMode::Dlss => fallback_to_screen_space(
                AntiAliasMode::Dlss,
                AntiAliasFallbackReason::UnsupportedDlss,
                capabilities,
            ),
        };
        report.with_graph_sample_counts(
            requested_graph_sample_count.max(self.mode.graph_sample_count()),
            report.effective_mode.graph_sample_count(),
        )
    }
}

impl AntiAliasFallbackReport {
    pub const fn effective_settings(self) -> AntiAliasSettings {
        AntiAliasSettings::new(self.effective_mode).with_taa_quality(self.taa_quality)
    }
}

fn resolve_auto(capabilities: &RenderCapabilitySummary) -> AntiAliasFallbackReport {
    if capabilities.supports_fxaa {
        AntiAliasFallbackReport::fallback(
            AntiAliasMode::Auto,
            AntiAliasMode::Fxaa,
            AntiAliasFallbackReason::AutoResolvedToFxaa,
        )
    } else if capabilities.supports_smaa {
        AntiAliasFallbackReport::fallback(
            AntiAliasMode::Auto,
            AntiAliasMode::Smaa,
            AntiAliasFallbackReason::AutoResolvedToSmaa,
        )
    } else {
        AntiAliasFallbackReport::fallback(
            AntiAliasMode::Auto,
            AntiAliasMode::Off,
            AntiAliasFallbackReason::UnsupportedFxaa,
        )
    }
}

fn resolve_fxaa(capabilities: &RenderCapabilitySummary) -> AntiAliasFallbackReport {
    if capabilities.supports_fxaa {
        AntiAliasFallbackReport::exact(AntiAliasMode::Fxaa)
    } else {
        AntiAliasFallbackReport::fallback(
            AntiAliasMode::Fxaa,
            AntiAliasMode::Off,
            AntiAliasFallbackReason::UnsupportedFxaa,
        )
    }
}

fn resolve_smaa(capabilities: &RenderCapabilitySummary) -> AntiAliasFallbackReport {
    if capabilities.supports_smaa {
        AntiAliasFallbackReport::exact(AntiAliasMode::Smaa)
    } else {
        fallback_to_screen_space(
            AntiAliasMode::Smaa,
            AntiAliasFallbackReason::UnsupportedSmaa,
            capabilities,
        )
    }
}

fn resolve_msaa(samples: u32, capabilities: &RenderCapabilitySummary) -> AntiAliasFallbackReport {
    if samples > 1 && samples <= capabilities.max_supported_msaa_samples {
        return AntiAliasFallbackReport::exact(AntiAliasMode::Msaa { samples });
    }

    fallback_to_screen_space(
        AntiAliasMode::Msaa { samples },
        AntiAliasFallbackReason::UnsupportedMsaaSampleCount,
        capabilities,
    )
}

fn resolve_taa(
    capabilities: &RenderCapabilitySummary,
    history_available: bool,
    taa_quality: TaaQualityPreset,
) -> AntiAliasFallbackReport {
    if !history_available {
        return fallback_to_screen_space_with_quality(
            AntiAliasMode::Taa,
            AntiAliasFallbackReason::MissingHistory,
            taa_quality,
            capabilities,
        );
    }
    if capabilities.supports_taa {
        AntiAliasFallbackReport::exact_with_taa_quality(AntiAliasMode::Taa, taa_quality)
    } else {
        fallback_to_screen_space_with_quality(
            AntiAliasMode::Taa,
            AntiAliasFallbackReason::UnsupportedTaa,
            taa_quality,
            capabilities,
        )
    }
}

fn fallback_to_screen_space(
    requested_mode: AntiAliasMode,
    reason: AntiAliasFallbackReason,
    capabilities: &RenderCapabilitySummary,
) -> AntiAliasFallbackReport {
    fallback_to_screen_space_with_quality(
        requested_mode,
        reason,
        TaaQualityPreset::Medium,
        capabilities,
    )
}

fn fallback_to_screen_space_with_quality(
    requested_mode: AntiAliasMode,
    reason: AntiAliasFallbackReason,
    taa_quality: TaaQualityPreset,
    capabilities: &RenderCapabilitySummary,
) -> AntiAliasFallbackReport {
    let effective_mode = if capabilities.supports_fxaa {
        AntiAliasMode::Fxaa
    } else if capabilities.supports_smaa {
        AntiAliasMode::Smaa
    } else {
        AntiAliasMode::Off
    };
    AntiAliasFallbackReport::fallback_with_taa_quality(
        requested_mode,
        effective_mode,
        taa_quality,
        reason,
    )
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        AntiAliasMode, AntiAliasSettings, RenderCapabilitySummary, TaaQualityPreset,
    };

    #[test]
    fn taa_quality_survives_exact_and_fallback_resolution() {
        let capabilities = RenderCapabilitySummary {
            supports_fxaa: true,
            supports_taa: true,
            ..RenderCapabilitySummary::default()
        };

        let exact = AntiAliasSettings::taa_with_quality(TaaQualityPreset::High)
            .resolve(&capabilities, true);
        assert_eq!(exact.effective_mode, AntiAliasMode::Taa);
        assert_eq!(
            exact.effective_settings().taa_quality,
            TaaQualityPreset::High
        );

        let fallback = AntiAliasSettings::taa_with_quality(TaaQualityPreset::Low)
            .resolve(&capabilities, false);
        assert_eq!(fallback.effective_mode, AntiAliasMode::Fxaa);
        assert_eq!(
            fallback.effective_settings().taa_quality,
            TaaQualityPreset::Low
        );
    }

    #[test]
    fn taa_resolution_reports_camera_msaa_sample_count_normalization() {
        let capabilities = RenderCapabilitySummary {
            supports_taa: true,
            supports_fxaa: true,
            max_supported_msaa_samples: 4,
            ..RenderCapabilitySummary::default()
        };

        let report = AntiAliasSettings::taa().resolve_with_requested_graph_sample_count(
            &capabilities,
            true,
            4,
        );

        assert_eq!(report.requested_mode, AntiAliasMode::Taa);
        assert_eq!(report.effective_mode, AntiAliasMode::Taa);
        assert_eq!(report.reason, None);
        assert_eq!(report.requested_graph_sample_count(), 4);
        assert_eq!(report.effective_graph_sample_count(), 1);
        assert!(report.graph_sample_count_normalized);
        assert!(report.taa_msaa_conflict_normalized());
        assert_eq!(report.normalization_count(), 1);
    }

    #[test]
    fn unsupported_terminal_aa_reports_slot_normalization() {
        let capabilities = RenderCapabilitySummary {
            supports_fxaa: true,
            ..RenderCapabilitySummary::default()
        };

        let report = AntiAliasSettings::smaa().resolve(&capabilities, false);

        assert_eq!(report.requested_mode, AntiAliasMode::Smaa);
        assert_eq!(report.effective_mode, AntiAliasMode::Fxaa);
        assert!(report.terminal_slot_normalized);
        assert!(!report.graph_sample_count_normalized);
        assert_eq!(report.normalization_count(), 1);
    }

    #[test]
    fn smaa_resolution_keeps_terminal_mode_when_supported() {
        let capabilities = RenderCapabilitySummary {
            supports_smaa: true,
            ..RenderCapabilitySummary::default()
        };

        let report = AntiAliasSettings::smaa().resolve(&capabilities, false);

        assert_eq!(report.requested_mode, AntiAliasMode::Smaa);
        assert_eq!(report.effective_mode, AntiAliasMode::Smaa);
        assert_eq!(report.reason, None);
        assert!(!report.terminal_slot_normalized);
        assert_eq!(report.normalization_count(), 0);
    }

    #[test]
    fn auto_resolution_uses_smaa_when_fxaa_is_unavailable() {
        let capabilities = RenderCapabilitySummary {
            supports_smaa: true,
            ..RenderCapabilitySummary::default()
        };

        let report = AntiAliasSettings::auto().resolve(&capabilities, false);

        assert_eq!(report.requested_mode, AntiAliasMode::Auto);
        assert_eq!(report.effective_mode, AntiAliasMode::Smaa);
        assert_eq!(
            report.reason,
            Some(super::AntiAliasFallbackReason::AutoResolvedToSmaa)
        );
        assert!(!report.terminal_slot_normalized);
        assert_eq!(report.normalization_count(), 0);
    }
}
