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
