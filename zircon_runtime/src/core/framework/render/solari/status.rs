use super::super::{RenderCapabilityMismatchDetail, RenderCapabilitySummary};
use super::{SolariCapabilityRequirement, SolariSettings};

const SOLARI_MAX_DEGRADATION_COUNT: usize = SolariCapabilityRequirement::ALL.len() + 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SolariRuntimeStatus {
    NotRequested,
    Ready,
    CapabilityMissing,
    ProviderMissing,
    ExperimentalDisabled,
    Unavailable,
}

impl SolariRuntimeStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "not-requested",
            Self::Ready => "ready",
            Self::CapabilityMissing => "capability-missing",
            Self::ProviderMissing => "provider-missing",
            Self::ExperimentalDisabled => "experimental-disabled",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SolariDegradationReason {
    BackendCapabilityMissing,
    ProviderMissing,
    ExperimentalDisabled,
    ProviderUnavailable,
}

impl SolariDegradationReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::BackendCapabilityMissing => "backend-capability-missing",
            Self::ProviderMissing => "provider-missing",
            Self::ExperimentalDisabled => "experimental-disabled",
            Self::ProviderUnavailable => "provider-unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolariRuntimeDegradation {
    pub reason: SolariDegradationReason,
    pub missing_capability: Option<RenderCapabilityMismatchDetail>,
    pub message: Option<String>,
}

impl SolariRuntimeDegradation {
    pub const fn missing_capability(detail: RenderCapabilityMismatchDetail) -> Self {
        Self {
            reason: SolariDegradationReason::BackendCapabilityMissing,
            missing_capability: Some(detail),
            message: None,
        }
    }

    pub const fn missing_provider() -> Self {
        Self {
            reason: SolariDegradationReason::ProviderMissing,
            missing_capability: None,
            message: None,
        }
    }

    pub fn experimental_disabled() -> Self {
        Self {
            reason: SolariDegradationReason::ExperimentalDisabled,
            missing_capability: None,
            message: Some("SolariExperimental requires the explicit experimental gate".to_string()),
        }
    }

    pub fn provider_unavailable(message: impl Into<String>) -> Self {
        Self {
            reason: SolariDegradationReason::ProviderUnavailable,
            missing_capability: None,
            message: Some(message.into()),
        }
    }

    pub const fn reason_label(&self) -> &'static str {
        self.reason.label()
    }
}

/// provider 的注册身份与可用状态快照，构造 viewport 报告时与后端能力合并。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolariProviderAvailability {
    pub provider_id: Option<String>,
    pub provider_status: SolariRuntimeStatus,
    pub provider_message: Option<String>,
}

impl SolariProviderAvailability {
    pub fn missing() -> Self {
        Self {
            provider_id: None,
            provider_status: SolariRuntimeStatus::ProviderMissing,
            provider_message: None,
        }
    }

    pub fn ready(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: Some(provider_id.into()),
            provider_status: SolariRuntimeStatus::Ready,
            provider_message: None,
        }
    }

    pub fn unavailable(provider_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            provider_id: Some(provider_id.into()),
            provider_status: SolariRuntimeStatus::Unavailable,
            provider_message: Some(message.into()),
        }
    }
}

impl Default for SolariProviderAvailability {
    fn default() -> Self {
        Self::missing()
    }
}

/// 区分未请求、能力不足、未注册、实验门控和 provider 不可用，供资格与统计诊断读取。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolariRuntimeReport {
    pub requested: bool,
    pub provider_id: Option<String>,
    pub status: SolariRuntimeStatus,
    pub settings: SolariSettings,
    pub degradations: Vec<SolariRuntimeDegradation>,
}

impl SolariRuntimeReport {
    /// 汇总全部退化原因，再按能力、provider、门控、可用性优先级选取单一状态。
    pub fn from_inputs(
        requested: bool,
        settings: SolariSettings,
        capabilities: &RenderCapabilitySummary,
        availability: &SolariProviderAvailability,
    ) -> Self {
        if !requested {
            return Self {
                requested,
                provider_id: availability.provider_id.clone(),
                status: SolariRuntimeStatus::NotRequested,
                settings,
                degradations: Vec::new(),
            };
        }

        let mut degradations = Vec::with_capacity(SOLARI_MAX_DEGRADATION_COUNT);
        for requirement in SolariCapabilityRequirement::ALL {
            let capability = requirement.capability_kind();
            if !capability.is_satisfied_by(capabilities) {
                degradations.push(SolariRuntimeDegradation::missing_capability(
                    RenderCapabilityMismatchDetail::new(capability),
                ));
            }
        }
        if availability.provider_id.is_none() {
            degradations.push(SolariRuntimeDegradation::missing_provider());
        }
        if !settings.experimental_enabled {
            degradations.push(SolariRuntimeDegradation::experimental_disabled());
        }
        if availability.provider_status == SolariRuntimeStatus::Unavailable {
            degradations.push(SolariRuntimeDegradation::provider_unavailable(
                availability
                    .provider_message
                    .clone()
                    .unwrap_or_else(|| "Solari provider is unavailable".to_string()),
            ));
        }

        let status =
            if degradations.iter().any(|degradation| {
                degradation.reason == SolariDegradationReason::BackendCapabilityMissing
            }) {
                SolariRuntimeStatus::CapabilityMissing
            } else if degradations
                .iter()
                .any(|degradation| degradation.reason == SolariDegradationReason::ProviderMissing)
            {
                SolariRuntimeStatus::ProviderMissing
            } else if degradations.iter().any(|degradation| {
                degradation.reason == SolariDegradationReason::ExperimentalDisabled
            }) {
                SolariRuntimeStatus::ExperimentalDisabled
            } else if degradations.iter().any(|degradation| {
                degradation.reason == SolariDegradationReason::ProviderUnavailable
            }) {
                SolariRuntimeStatus::Unavailable
            } else {
                SolariRuntimeStatus::Ready
            };

        Self {
            requested,
            provider_id: availability.provider_id.clone(),
            status,
            settings,
            degradations,
        }
    }

    pub fn enabled(&self) -> bool {
        self.requested && self.status == SolariRuntimeStatus::Ready
    }

    pub fn degradation_reason_labels(&self) -> Vec<&'static str> {
        self.degradations
            .iter()
            .map(SolariRuntimeDegradation::reason_label)
            .collect()
    }
}

impl Default for SolariRuntimeReport {
    fn default() -> Self {
        Self {
            requested: false,
            provider_id: None,
            status: SolariRuntimeStatus::NotRequested,
            settings: SolariSettings::default(),
            degradations: Vec::new(),
        }
    }
}

#[cfg(test)]
#[path = "tests/status.rs"]
mod tests;

#[cfg(test)]
#[path = "status/tests/preallocated_degradations_tests.rs"]
mod preallocated_degradations_tests;
