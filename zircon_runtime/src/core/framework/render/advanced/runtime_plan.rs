use super::super::{RenderCapabilitySummary, RenderProductProfile, RenderProfileBundle};
use super::{AdvancedProviderAvailability, AdvancedProviderReport, AdvancedRenderFeature};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancedProfileRuntimePlan {
    pub profile: RenderProductProfile,
    pub reports: Vec<AdvancedProviderReport>,
}

impl AdvancedProfileRuntimePlan {
    pub fn from_profile_bundle(
        bundle: &RenderProfileBundle,
        capabilities: &RenderCapabilitySummary,
        availability: &AdvancedProviderAvailability,
    ) -> Self {
        let reports = AdvancedRenderFeature::ALL
            .into_iter()
            .map(|feature| {
                AdvancedProviderReport::from_inputs(
                    feature,
                    bundle.has_feature(feature.product_feature()),
                    capabilities,
                    availability,
                )
            })
            .collect();

        Self {
            profile: bundle.profile(),
            reports,
        }
    }

    pub fn report_for(&self, feature: AdvancedRenderFeature) -> Option<&AdvancedProviderReport> {
        self.reports.iter().find(|report| report.feature == feature)
    }

    pub fn enabled_features(&self) -> Vec<AdvancedRenderFeature> {
        self.reports
            .iter()
            .filter(|report| report.enabled())
            .map(|report| report.feature)
            .collect()
    }

    pub fn degraded_reports(&self) -> Vec<&AdvancedProviderReport> {
        self.reports
            .iter()
            .filter(|report| !report.degradations.is_empty())
            .collect()
    }

    pub fn virtual_geometry_enabled(&self) -> bool {
        self.report_for(AdvancedRenderFeature::VirtualGeometry)
            .is_some_and(AdvancedProviderReport::enabled)
    }

    pub fn hybrid_global_illumination_enabled(&self) -> bool {
        self.report_for(AdvancedRenderFeature::HybridGlobalIllumination)
            .is_some_and(AdvancedProviderReport::enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{RenderCapabilityKind, RenderCapabilityMismatchDetail};

    #[test]
    fn default_render_plan_does_not_request_advanced_providers() {
        let plan = AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::default_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new()
                .with_virtual_geometry_provider("vg")
                .with_hybrid_gi_provider("hgi"),
        );

        assert_eq!(plan.profile, RenderProductProfile::DefaultRender);
        assert!(plan.enabled_features().is_empty());
        assert!(plan.degraded_reports().is_empty());
        assert!(!plan.virtual_geometry_enabled());
        assert!(!plan.hybrid_global_illumination_enabled());
    }

    #[test]
    fn advanced_render_plan_enables_ready_providers() {
        let plan = AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new()
                .with_virtual_geometry_provider("vg")
                .with_hybrid_gi_provider("hgi"),
        );

        assert_eq!(
            plan.enabled_features(),
            vec![
                AdvancedRenderFeature::VirtualGeometry,
                AdvancedRenderFeature::HybridGlobalIllumination,
            ]
        );
        assert!(plan.degraded_reports().is_empty());
        assert_eq!(
            plan.report_for(AdvancedRenderFeature::VirtualGeometry)
                .and_then(|report| report.provider_id.as_deref()),
            Some("vg")
        );
    }

    #[test]
    fn advanced_render_plan_reports_missing_capability_and_provider() {
        let plan = AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &RenderCapabilitySummary {
                backend_name: "m9a-test".to_string(),
                virtual_geometry_supported: false,
                hybrid_global_illumination_supported: true,
                supports_storage_buffers: true,
                supports_indirect_draw: true,
                supports_buffer_readback: true,
                ..RenderCapabilitySummary::default()
            },
            &AdvancedProviderAvailability::new().with_hybrid_gi_provider("hgi"),
        );

        let vg = plan
            .report_for(AdvancedRenderFeature::VirtualGeometry)
            .expect("virtual geometry report");
        assert!(!vg.enabled());
        assert_eq!(
            vg.degradation_reason_labels(),
            vec!["backend-capability-missing", "provider-missing"]
        );
        assert_eq!(
            vg.degradations[0].missing_capability,
            Some(RenderCapabilityMismatchDetail::new(
                RenderCapabilityKind::VirtualGeometry,
            ))
        );

        let hgi = plan
            .report_for(AdvancedRenderFeature::HybridGlobalIllumination)
            .expect("hybrid GI report");
        assert!(hgi.enabled());
        assert_eq!(hgi.provider_id.as_deref(), Some("hgi"));
    }

    fn advanced_capabilities() -> RenderCapabilitySummary {
        RenderCapabilitySummary {
            backend_name: "m9a-test".to_string(),
            virtual_geometry_supported: true,
            hybrid_global_illumination_supported: true,
            supports_storage_buffers: true,
            supports_indirect_draw: true,
            supports_buffer_readback: true,
            ..RenderCapabilitySummary::default()
        }
    }
}
