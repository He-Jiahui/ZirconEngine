use super::super::SolariSettings;
use super::*;
use crate::core::framework::render::RenderCapabilityKind;

#[test]
fn solari_report_is_not_requested_for_default_profiles() {
    let report = SolariRuntimeReport::from_inputs(
        false,
        SolariSettings::default(),
        &solari_capabilities(),
        &SolariProviderAvailability::ready("solari"),
    );

    assert_eq!(report.status, SolariRuntimeStatus::NotRequested);
    assert!(report.degradations.is_empty());
    assert!(!report.enabled());
}

#[test]
fn solari_report_rejects_missing_bevy_solari_capabilities() {
    let report = SolariRuntimeReport::from_inputs(
        true,
        SolariSettings::experimental_enabled(),
        &RenderCapabilitySummary::default(),
        &SolariProviderAvailability::ready("solari"),
    );

    assert_eq!(report.status, SolariRuntimeStatus::CapabilityMissing);
    assert_eq!(
        report.degradations[0].missing_capability,
        Some(RenderCapabilityMismatchDetail::new(
            RenderCapabilityKind::InlineRayQuery,
        ))
    );
    assert!(report
        .degradation_reason_labels()
        .contains(&"backend-capability-missing"));
}

#[test]
fn solari_report_distinguishes_provider_missing_experimental_gate_and_unavailable_provider() {
    let missing = SolariRuntimeReport::from_inputs(
        true,
        SolariSettings::experimental_enabled(),
        &solari_capabilities(),
        &SolariProviderAvailability::missing(),
    );
    assert_eq!(missing.status, SolariRuntimeStatus::ProviderMissing);

    let disabled = SolariRuntimeReport::from_inputs(
        true,
        SolariSettings::default(),
        &solari_capabilities(),
        &SolariProviderAvailability::ready("solari"),
    );
    assert_eq!(disabled.status, SolariRuntimeStatus::ExperimentalDisabled);

    let unavailable = SolariRuntimeReport::from_inputs(
        true,
        SolariSettings::experimental_enabled(),
        &solari_capabilities(),
        &SolariProviderAvailability::unavailable("solari", "no pass executor"),
    );
    assert_eq!(unavailable.status, SolariRuntimeStatus::Unavailable);
    assert_eq!(
        unavailable.degradation_reason_labels(),
        vec!["provider-unavailable"]
    );
}

fn solari_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        acceleration_structures_supported: true,
        inline_ray_query: true,
        supports_buffer_binding_array: true,
        supports_texture_binding_array: true,
        supports_non_uniform_resource_indexing: true,
        supports_partially_bound_binding_array: true,
        ..RenderCapabilitySummary::default()
    }
}
