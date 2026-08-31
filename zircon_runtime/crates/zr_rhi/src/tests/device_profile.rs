use crate::{
    AdapterSelectionError, AdapterSelectionPolicy, DeviceGeneration, DeviceId,
    DiagnosticReadbackBudget, GpuMemoryBudget, RejectedAdapterReason, RenderAdapterCatalog,
    RenderAdapterClass, RenderAdapterFacts, RenderAdapterSelector, RenderBackendKind,
    RenderDeviceFeature, RenderDeviceFeatureSet, RenderDeviceLimits, RenderDeviceNegotiationError,
    RenderDeviceProfile, RenderDeviceQueueTopology, RenderDeviceRequestFailure,
    RenderDeviceRequestPolicy, SubmissionLimits,
};

fn features(features: &[RenderDeviceFeature]) -> RenderDeviceFeatureSet {
    let mut supported = RenderDeviceFeatureSet::default();
    for feature in features {
        supported.insert(*feature);
    }
    supported
}

fn adapter(
    backend: RenderBackendKind,
    name: &str,
    vendor_id: u32,
    device_id: u32,
    adapter_class: RenderAdapterClass,
) -> RenderAdapterFacts {
    RenderAdapterFacts::new(
        backend,
        name,
        vendor_id,
        device_id,
        "test-driver",
        adapter_class,
        None,
        RenderDeviceFeatureSet::default(),
    )
}

#[test]
fn mvp_baseline_does_not_request_optional_device_features() {
    let supported = features(&[
        RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget,
        RenderDeviceFeature::GpuTimestamp,
        RenderDeviceFeature::BindlessMaterialArrays,
    ]);

    let negotiation = RenderDeviceRequestPolicy::mvp_baseline()
        .negotiate(&supported)
        .expect("the MVP baseline has no optional device feature requirement");

    assert!(negotiation.requested_features().is_empty());
    assert!(negotiation.unavailable_features().is_empty());
}

#[test]
fn optional_profile_features_are_requested_only_when_the_adapter_supports_them() {
    let policy = RenderDeviceRequestPolicy::mvp_baseline()
        .with_optional_feature(RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget)
        .with_optional_feature(RenderDeviceFeature::GpuTimestamp);
    let supported = features(&[RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget]);

    let negotiation = policy
        .negotiate(&supported)
        .expect("optional capabilities must not reject an MVP-capable adapter");

    assert!(negotiation
        .requested_features()
        .contains(RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget));
    assert!(!negotiation
        .requested_features()
        .contains(RenderDeviceFeature::GpuTimestamp));
    assert!(negotiation
        .unavailable_features()
        .contains(RenderDeviceFeature::GpuTimestamp));
}

#[test]
fn unsupported_hard_feature_returns_a_typed_negotiation_error() {
    let policy = RenderDeviceRequestPolicy::mvp_baseline()
        .with_required_feature(RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget);

    assert_eq!(
        policy.negotiate(&RenderDeviceFeatureSet::default()),
        Err(RenderDeviceNegotiationError::RequiredFeatureUnavailable {
            feature: RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget,
        })
    );
}

#[test]
fn device_request_failure_keeps_adapter_feature_limit_and_backend_diagnostics_together() {
    let adapter = adapter(
        RenderBackendKind::Dx12,
        "discrete",
        0x10de,
        0x2468,
        RenderAdapterClass::Discrete,
    );
    let negotiation = RenderDeviceRequestPolicy::mvp_baseline()
        .with_optional_feature(RenderDeviceFeature::GpuTimestamp)
        .negotiate(&RenderDeviceFeatureSet::default())
        .expect("an unavailable optional feature must retain a fallback receipt");
    let failure = RenderDeviceRequestFailure::new(
        adapter.clone(),
        negotiation,
        RenderDeviceLimits {
            max_bind_groups: 5,
            max_texture_dimension_2d: 16_384,
            max_texture_array_layers: 2_048,
            max_sampled_textures_per_shader_stage: 16,
            max_binding_array_elements_per_shader_stage: 0,
            max_binding_array_sampler_elements_per_shader_stage: 0,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_buffer_binding_size: 128 * 1024 * 1024,
        },
        "native driver rejected the descriptor",
    );

    assert_eq!(failure.adapter(), &adapter);
    assert!(failure
        .feature_negotiation()
        .unavailable_features()
        .contains(RenderDeviceFeature::GpuTimestamp));
    assert_eq!(failure.requested_limits().max_bind_groups, 5);
    assert_eq!(
        failure.backend_detail(),
        "native driver rejected the descriptor"
    );
}

#[test]
fn adapter_selection_is_stable_when_enumeration_order_changes() {
    let integrated = adapter(
        RenderBackendKind::Dx12,
        "integrated",
        0x8086,
        0x1234,
        RenderAdapterClass::Integrated,
    );
    let discrete = adapter(
        RenderBackendKind::Vulkan,
        "discrete",
        0x10de,
        0x5678,
        RenderAdapterClass::Discrete,
    );
    let policy = AdapterSelectionPolicy::default();

    let forward = RenderAdapterCatalog::new(vec![integrated.clone(), discrete.clone()])
        .select(&policy)
        .expect("one adapter must be selected");
    let reverse = RenderAdapterCatalog::new(vec![discrete.clone(), integrated])
        .select(&policy)
        .expect("selection must not depend on enumeration order");

    assert_eq!(forward, reverse);
    assert_eq!(forward.selected(), &discrete);
}

#[test]
fn adapter_selection_reports_when_every_candidate_is_rejected() {
    let catalog = RenderAdapterCatalog::new(vec![adapter(
        RenderBackendKind::Other,
        "software",
        0,
        0,
        RenderAdapterClass::Cpu,
    )]);

    assert!(matches!(
        catalog.select(&AdapterSelectionPolicy::default()),
        Err(AdapterSelectionError::NoEligibleAdapter { rejected })
            if rejected.len() == 1
                && rejected[0].reason == RejectedAdapterReason::SoftwareNotAllowed
    ));
}

#[test]
fn adapter_selection_receipt_honors_override_and_records_lower_priority_candidates() {
    let integrated = adapter(
        RenderBackendKind::Dx12,
        "integrated",
        0x8086,
        0x1234,
        RenderAdapterClass::Integrated,
    );
    let discrete = adapter(
        RenderBackendKind::Vulkan,
        "discrete",
        0x10de,
        0x5678,
        RenderAdapterClass::Discrete,
    );
    let policy = AdapterSelectionPolicy::default().with_adapter_override(RenderAdapterSelector {
        backend: Some(RenderBackendKind::Dx12),
        vendor_id: Some(0x8086),
        device_id: Some(0x1234),
    });

    let receipt = RenderAdapterCatalog::new(vec![discrete.clone(), integrated.clone()])
        .select(&policy)
        .expect("the explicit override must be selected before performance preference");

    assert_eq!(receipt.selected(), &integrated);
    assert_eq!(receipt.rejected().len(), 1);
    assert_eq!(receipt.rejected()[0].adapter, discrete);
    assert_eq!(
        receipt.rejected()[0].reason,
        RejectedAdapterReason::OverrideMismatch
    );
}

#[test]
fn adapter_selection_denylist_filters_a_preferred_candidate_before_ranking() {
    let integrated = adapter(
        RenderBackendKind::Dx12,
        "integrated",
        0x8086,
        0x1234,
        RenderAdapterClass::Integrated,
    );
    let discrete = adapter(
        RenderBackendKind::Vulkan,
        "discrete",
        0x10de,
        0x5678,
        RenderAdapterClass::Discrete,
    );
    let policy = AdapterSelectionPolicy::default().deny_adapter(RenderAdapterSelector {
        backend: Some(RenderBackendKind::Vulkan),
        vendor_id: Some(0x10de),
        device_id: Some(0x5678),
    });

    let receipt = RenderAdapterCatalog::new(vec![integrated.clone(), discrete.clone()])
        .select(&policy)
        .expect("the non-denied integrated adapter remains eligible");

    assert_eq!(receipt.selected(), &integrated);
    assert_eq!(receipt.rejected().len(), 1);
    assert_eq!(receipt.rejected()[0].adapter, discrete);
    assert_eq!(receipt.rejected()[0].reason, RejectedAdapterReason::Denied);
}

#[test]
fn immutable_device_profile_keeps_identity_and_negotiated_feature_receipt_together() {
    let adapter = adapter(
        RenderBackendKind::Dx12,
        "discrete",
        0x10de,
        0x2468,
        RenderAdapterClass::Discrete,
    );
    let policy = RenderDeviceRequestPolicy::mvp_baseline()
        .with_optional_feature(RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget);
    let negotiation = policy
        .negotiate(&features(&[
            RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget,
        ]))
        .expect("the adapter supports the selected HDR profile");

    let profile = RenderDeviceProfile::new(
        DeviceId::new(41),
        DeviceGeneration::initial(),
        adapter,
        negotiation,
        RenderDeviceLimits {
            max_bind_groups: 5,
            max_texture_dimension_2d: 16_384,
            max_texture_array_layers: 2_048,
            max_sampled_textures_per_shader_stage: 16,
            max_binding_array_elements_per_shader_stage: 0,
            max_binding_array_sampler_elements_per_shader_stage: 0,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_buffer_binding_size: 128 * 1024 * 1024,
        },
        RenderDeviceQueueTopology::single_serialized_queue(),
        GpuMemoryBudget::reference_1080p_mid(),
        SubmissionLimits::default(),
        DiagnosticReadbackBudget::default(),
    );

    assert_eq!(profile.device_id(), DeviceId::new(41));
    assert_eq!(profile.generation(), DeviceGeneration::initial());
    assert!(profile
        .requested_features()
        .contains(RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget));
    assert_eq!(profile.device_limits().max_bind_groups, 5);
    assert_eq!(profile.queue_topology().physical_queue_count, 1);
    assert_eq!(
        profile.memory_budget(),
        GpuMemoryBudget::reference_1080p_mid()
    );
    assert_eq!(profile.submission_limits(), SubmissionLimits::default());
    assert_eq!(
        profile.diagnostic_readback_budget(),
        DiagnosticReadbackBudget::default()
    );
    assert!(profile.queue_topology().supports_compute_commands);
    assert!(!profile.queue_topology().supports_async_compute);
}
