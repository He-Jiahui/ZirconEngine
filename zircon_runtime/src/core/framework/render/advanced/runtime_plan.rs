use super::super::{RenderCapabilitySummary, RenderProductProfile, RenderProfileBundle};
use super::{AdvancedProviderAvailability, AdvancedProviderReport, AdvancedRenderFeature};

/// 每个 viewport 的高级特性资格计划；提交时仍与编译后管线的运行标志取交集。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancedProfileRuntimePlan {
    pub profile: RenderProductProfile,
    pub reports: Vec<AdvancedProviderReport>,
}

impl AdvancedProfileRuntimePlan {
    /// 对未请求项也保留报告，便于诊断区分未请求与请求后退化。
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
        let mut features = Vec::with_capacity(self.reports.len());
        features.extend(
            self.reports
                .iter()
                .filter(|report| report.enabled())
                .map(|report| report.feature),
        );
        features
    }

    pub fn degraded_reports(&self) -> Vec<&AdvancedProviderReport> {
        let mut reports = Vec::with_capacity(self.reports.len());
        reports.extend(
            self.reports
                .iter()
                .filter(|report| !report.degradations.is_empty()),
        );
        reports
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
#[path = "tests/runtime_plan.rs"]
mod tests;
