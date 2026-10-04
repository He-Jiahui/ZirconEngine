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

    let exact =
        AntiAliasSettings::taa_with_quality(TaaQualityPreset::High).resolve(&capabilities, true);
    assert_eq!(exact.effective_mode, AntiAliasMode::Taa);
    assert_eq!(
        exact.effective_settings().taa_quality,
        TaaQualityPreset::High
    );

    let fallback =
        AntiAliasSettings::taa_with_quality(TaaQualityPreset::Low).resolve(&capabilities, false);
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

    let report =
        AntiAliasSettings::taa().resolve_with_requested_graph_sample_count(&capabilities, true, 4);

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
