#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphComputePipelineFamily {
    pub name: String,
    pub interface_generation: u64,
}

impl RenderGraphComputePipelineFamily {
    pub fn new(name: impl Into<String>, interface_generation: u64) -> Self {
        Self {
            name: name.into(),
            interface_generation,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("compute pipeline fallback family must not be empty".to_string());
        }
        if self.interface_generation == 0 {
            return Err(format!(
                "compute pipeline fallback family `{}` requires a non-zero interface generation",
                self.name
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderGraphComputePipelineFallbackPolicy {
    #[default]
    Reject,
    LastGood(RenderGraphComputePipelineFamily),
}

impl RenderGraphComputePipelineFallbackPolicy {
    pub fn last_good(family: impl Into<String>, interface_generation: u64) -> Self {
        Self::LastGood(RenderGraphComputePipelineFamily::new(
            family,
            interface_generation,
        ))
    }

    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Reject => Ok(()),
            Self::LastGood(family) => family.validate(),
        }
    }

    pub const fn family(&self) -> Option<&RenderGraphComputePipelineFamily> {
        match self {
            Self::Reject => None,
            Self::LastGood(family) => Some(family),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphComputePipelineResolutionStatus {
    Ready,
    UsingLastGood,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphComputePipelineResolution {
    pub status: RenderGraphComputePipelineResolutionStatus,
    pub family: Option<RenderGraphComputePipelineFamily>,
    pub candidate_artifact_fingerprint: u64,
    pub resolved_artifact_fingerprint: u64,
    pub device_id: Option<u64>,
    pub device_generation: Option<u64>,
    pub candidate_failure: Option<String>,
}

impl RenderGraphComputePipelineResolution {
    pub fn ready(
        policy: &RenderGraphComputePipelineFallbackPolicy,
        artifact_fingerprint: u64,
        device_epoch: Option<(u64, u64)>,
    ) -> Self {
        Self {
            status: RenderGraphComputePipelineResolutionStatus::Ready,
            family: policy.family().cloned(),
            candidate_artifact_fingerprint: artifact_fingerprint,
            resolved_artifact_fingerprint: artifact_fingerprint,
            device_id: device_epoch.map(|epoch| epoch.0),
            device_generation: device_epoch.map(|epoch| epoch.1),
            candidate_failure: None,
        }
    }

    pub fn using_last_good(
        family: RenderGraphComputePipelineFamily,
        candidate_artifact_fingerprint: u64,
        resolved_artifact_fingerprint: u64,
        device_epoch: (u64, u64),
        candidate_failure: impl Into<String>,
    ) -> Self {
        Self {
            status: RenderGraphComputePipelineResolutionStatus::UsingLastGood,
            family: Some(family),
            candidate_artifact_fingerprint,
            resolved_artifact_fingerprint,
            device_id: Some(device_epoch.0),
            device_generation: Some(device_epoch.1),
            candidate_failure: Some(candidate_failure.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RenderGraphComputePipelineFallbackPolicy, RenderGraphComputePipelineResolution,
        RenderGraphComputePipelineResolutionStatus,
    };

    #[test]
    fn compute_pipeline_fallback_is_rejected_unless_explicitly_versioned() {
        let default_policy = RenderGraphComputePipelineFallbackPolicy::default();
        assert_eq!(
            default_policy,
            RenderGraphComputePipelineFallbackPolicy::Reject
        );
        assert!(default_policy.validate().is_ok());

        let invalid = RenderGraphComputePipelineFallbackPolicy::last_good("ao.evaluate", 0);
        assert!(invalid.validate().is_err());

        let compatible = RenderGraphComputePipelineFallbackPolicy::last_good("ao.evaluate", 2);
        assert!(compatible.validate().is_ok());
        let family = compatible.family().expect("last-good family");
        assert_eq!(family.name, "ao.evaluate");
        assert_eq!(family.interface_generation, 2);
    }

    #[test]
    fn ready_resolution_preserves_explicit_compatibility_identity() {
        let policy = RenderGraphComputePipelineFallbackPolicy::last_good("ao.spatial", 2);
        let resolution = RenderGraphComputePipelineResolution::ready(&policy, 41, Some((7, 3)));

        assert_eq!(
            resolution.status,
            RenderGraphComputePipelineResolutionStatus::Ready
        );
        assert_eq!(resolution.candidate_artifact_fingerprint, 41);
        assert_eq!(resolution.resolved_artifact_fingerprint, 41);
        assert_eq!(resolution.device_id, Some(7));
        assert_eq!(resolution.device_generation, Some(3));
        assert!(resolution.candidate_failure.is_none());
        assert_eq!(
            resolution
                .family
                .as_ref()
                .expect("resolution family")
                .interface_generation,
            2
        );
    }
}
