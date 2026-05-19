use super::super::{RenderCapabilityMismatchDetail, RenderCapabilitySummary};
use super::AdvancedRenderFeature;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdvancedProviderStatus {
    NotRequested,
    Ready,
    Degraded,
}

impl AdvancedProviderStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "not-requested",
            Self::Ready => "ready",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdvancedRenderDegradationReason {
    BackendCapabilityMissing,
    ProviderMissing,
}

impl AdvancedRenderDegradationReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::BackendCapabilityMissing => "backend-capability-missing",
            Self::ProviderMissing => "provider-missing",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancedRenderDegradation {
    pub feature: AdvancedRenderFeature,
    pub reason: AdvancedRenderDegradationReason,
    pub missing_capability: Option<RenderCapabilityMismatchDetail>,
}

impl AdvancedRenderDegradation {
    pub const fn missing_capability(
        feature: AdvancedRenderFeature,
        detail: RenderCapabilityMismatchDetail,
    ) -> Self {
        Self {
            feature,
            reason: AdvancedRenderDegradationReason::BackendCapabilityMissing,
            missing_capability: Some(detail),
        }
    }

    pub const fn missing_provider(feature: AdvancedRenderFeature) -> Self {
        Self {
            feature,
            reason: AdvancedRenderDegradationReason::ProviderMissing,
            missing_capability: None,
        }
    }

    pub const fn reason_label(&self) -> &'static str {
        self.reason.label()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdvancedProviderAvailability {
    pub virtual_geometry_provider_id: Option<String>,
    pub hybrid_gi_provider_id: Option<String>,
}

impl AdvancedProviderAvailability {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_virtual_geometry_provider(mut self, provider_id: impl Into<String>) -> Self {
        self.virtual_geometry_provider_id = Some(provider_id.into());
        self
    }

    pub fn with_hybrid_gi_provider(mut self, provider_id: impl Into<String>) -> Self {
        self.hybrid_gi_provider_id = Some(provider_id.into());
        self
    }

    pub fn provider_id(&self, feature: AdvancedRenderFeature) -> Option<&str> {
        match feature {
            AdvancedRenderFeature::VirtualGeometry => self.virtual_geometry_provider_id.as_deref(),
            AdvancedRenderFeature::HybridGlobalIllumination => {
                self.hybrid_gi_provider_id.as_deref()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancedProviderReport {
    pub feature: AdvancedRenderFeature,
    pub requested: bool,
    pub provider_id: Option<String>,
    pub status: AdvancedProviderStatus,
    pub degradations: Vec<AdvancedRenderDegradation>,
}

impl AdvancedProviderReport {
    pub fn from_inputs(
        feature: AdvancedRenderFeature,
        requested: bool,
        capabilities: &RenderCapabilitySummary,
        availability: &AdvancedProviderAvailability,
    ) -> Self {
        let provider_id = availability.provider_id(feature).map(str::to_string);
        if !requested {
            return Self {
                feature,
                requested,
                provider_id,
                status: AdvancedProviderStatus::NotRequested,
                degradations: Vec::new(),
            };
        }

        let mut degradations = Vec::new();
        for required_capability in feature.required_capabilities() {
            if !required_capability.is_satisfied_by(capabilities) {
                degradations.push(AdvancedRenderDegradation::missing_capability(
                    feature,
                    RenderCapabilityMismatchDetail::new(*required_capability),
                ));
            }
        }
        if provider_id.is_none() {
            degradations.push(AdvancedRenderDegradation::missing_provider(feature));
        }

        let status = if degradations.is_empty() {
            AdvancedProviderStatus::Ready
        } else {
            AdvancedProviderStatus::Degraded
        };

        Self {
            feature,
            requested,
            provider_id,
            status,
            degradations,
        }
    }

    pub fn enabled(&self) -> bool {
        self.requested && self.status == AdvancedProviderStatus::Ready
    }

    pub fn degradation_reason_labels(&self) -> Vec<&'static str> {
        self.degradations
            .iter()
            .map(AdvancedRenderDegradation::reason_label)
            .collect()
    }
}
