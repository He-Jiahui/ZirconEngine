use crate::core::framework::render::RenderCapabilitySummary;

use super::{AntiAliasFallbackReason, AntiAliasFallbackReport, AntiAliasMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AntiAliasSettings {
    pub mode: AntiAliasMode,
}

impl Default for AntiAliasSettings {
    fn default() -> Self {
        Self::auto()
    }
}

impl AntiAliasSettings {
    pub const fn new(mode: AntiAliasMode) -> Self {
        Self { mode }
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
        match self.mode {
            AntiAliasMode::Off => AntiAliasFallbackReport::exact(AntiAliasMode::Off),
            AntiAliasMode::Auto => resolve_auto(capabilities),
            AntiAliasMode::Fxaa => resolve_fxaa(capabilities),
            AntiAliasMode::Msaa { samples } => resolve_msaa(samples, capabilities),
            AntiAliasMode::Taa => resolve_taa(capabilities, history_available),
            AntiAliasMode::Smaa => fallback_to_screen_space(
                AntiAliasMode::Smaa,
                AntiAliasFallbackReason::UnsupportedSmaa,
                capabilities,
            ),
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
        }
    }
}

impl AntiAliasFallbackReport {
    pub const fn effective_settings(self) -> AntiAliasSettings {
        AntiAliasSettings::new(self.effective_mode)
    }
}

fn resolve_auto(capabilities: &RenderCapabilitySummary) -> AntiAliasFallbackReport {
    if capabilities.supports_fxaa {
        AntiAliasFallbackReport::fallback(
            AntiAliasMode::Auto,
            AntiAliasMode::Fxaa,
            AntiAliasFallbackReason::AutoResolvedToFxaa,
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
) -> AntiAliasFallbackReport {
    if !history_available {
        return fallback_to_screen_space(
            AntiAliasMode::Taa,
            AntiAliasFallbackReason::MissingHistory,
            capabilities,
        );
    }
    if capabilities.supports_taa {
        AntiAliasFallbackReport::exact(AntiAliasMode::Taa)
    } else {
        fallback_to_screen_space(
            AntiAliasMode::Taa,
            AntiAliasFallbackReason::UnsupportedTaa,
            capabilities,
        )
    }
}

fn fallback_to_screen_space(
    requested_mode: AntiAliasMode,
    reason: AntiAliasFallbackReason,
    capabilities: &RenderCapabilitySummary,
) -> AntiAliasFallbackReport {
    let effective_mode = if capabilities.supports_fxaa {
        AntiAliasMode::Fxaa
    } else {
        AntiAliasMode::Off
    };
    AntiAliasFallbackReport::fallback(requested_mode, effective_mode, reason)
}
