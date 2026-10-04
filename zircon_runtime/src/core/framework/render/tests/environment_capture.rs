use super::*;

#[test]
fn request_keeps_revision_identity_and_rejects_invalid_position() {
    let request = RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
        .unwrap()
        .with_clip_planes(0.25, 512.0)
        .unwrap()
        .with_face_size(256)
        .unwrap()
        .with_quality(SourceCubemapPrefilterQuality::High);
    assert_eq!(request.capture_id(), "atrium");
    assert_eq!(request.scene_revision(), 9);
    assert_eq!(request.environment_revision(), 9);
    assert_eq!(request.output_generation(), 9);
    assert_eq!(request.face_size(), 256);
    assert_eq!(request.quality(), SourceCubemapPrefilterQuality::High);

    assert_eq!(
        RenderEnvironmentCaptureRequest::new("atrium", [f32::NAN; 3], 9),
        Err(RenderEnvironmentCaptureRequestError::NonFinitePosition)
    );
}

#[test]
fn typed_probe_target_survives_request_mutators_and_output_identity() {
    let cubemap = ResourceId::from_stable_label("lib://probes/atrium.zcube");
    let request = RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
        .unwrap()
        .with_reflection_probe_target(42, cubemap)
        .with_clip_planes(0.25, 512.0)
        .unwrap()
        .with_face_size(256)
        .unwrap();

    assert_eq!(request.reflection_probe_target(), Some((42, cubemap)));
    let output = RenderEnvironmentCaptureOutputIdentity::from_request(&request);
    assert_eq!(output.reflection_probe_target(), Some((42, cubemap)));
    assert!(output.has_reflection_probe_target());
}

#[test]
fn persistence_target_keeps_asset_and_runtime_cache_identities_distinct() {
    let base = RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
        .unwrap()
        .with_face_size(256)
        .unwrap();
    let artifact_request = IblBakeArtifactRequest::new(
        IblBakeKey::source_cubemap(9, [1, 2, 3, 4]),
        256,
        source_cubemap_mip_count(256),
    );
    let request = base
        .clone()
        .with_persistence_output_uri("lib://probes/atrium.zcube")
        .unwrap()
        .with_persistence_artifact_request(artifact_request)
        .unwrap()
        .with_clip_planes(0.25, 512.0)
        .unwrap()
        .with_quality(SourceCubemapPrefilterQuality::High);

    assert_eq!(
        request.persistence_output_uri(),
        Some("lib://probes/atrium.zcube")
    );
    let output = RenderEnvironmentCaptureOutputIdentity::from_request(&request);
    assert_eq!(
        output.persistence_output_uri(),
        Some("lib://probes/atrium.zcube")
    );
    assert_eq!(
        output.persistence_artifact_request(),
        Some(artifact_request)
    );
    let runtime_cache_request = output
        .runtime_cache_artifact_request()
        .expect("persisted capture must expose its renderer-owned cache identity");
    assert_eq!(
        runtime_cache_request,
        request.runtime_cache_artifact_request().unwrap()
    );
    assert_eq!(runtime_cache_request.bake_key(), request.ibl_bake_key());
    assert_ne!(
        runtime_cache_request.bake_key(),
        artifact_request.bake_key()
    );
    assert_eq!(runtime_cache_request.source_face_size(), 256);
    assert_eq!(runtime_cache_request.source_mip_count(), 9);
    assert_eq!(
        RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .ibl_bake_key(),
        RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .with_persistence_output_uri("lib://other/path.zcube")
            .unwrap()
            .ibl_bake_key()
    );
    assert!(matches!(
        base.clone()
            .with_persistence_output_uri("lib://probes/atrium.zcube")
            .unwrap()
            .with_persistence_artifact_request(artifact_request)
            .unwrap()
            .with_face_size(128),
        Err(RenderEnvironmentCaptureRequestError::PersistenceArtifactSourceLayoutMismatch { .. })
    ));
    assert_eq!(
        RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .with_persistence_output_uri("   "),
        Err(RenderEnvironmentCaptureRequestError::EmptyPersistenceOutputUri)
    );
    assert_eq!(
        RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .with_persistence_artifact_request(artifact_request),
        Err(RenderEnvironmentCaptureRequestError::PersistenceArtifactRequiresOutputUri)
    );
    assert!(matches!(
        RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .with_persistence_output_uri("lib://probes/atrium.zcube")
            .unwrap()
            .with_persistence_artifact_request(artifact_request.with_pmrem_layout(64, 1)),
        Err(RenderEnvironmentCaptureRequestError::PersistenceArtifactPmremLayoutMismatch { .. })
    ));
    assert!(matches!(
        RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9)
            .unwrap()
            .with_persistence_output_uri("lib://probes/atrium.zcube")
            .unwrap()
            .with_persistence_artifact_request(
                artifact_request.with_required_contents(IblBakeArtifactContents::IEM)
            ),
        Err(RenderEnvironmentCaptureRequestError::UnsupportedPersistenceArtifactContents(
            bits
        )) if bits == IblBakeArtifactContents::IEM.bits()
    ));
}

#[test]
fn capture_ibl_key_is_stable_and_separates_quality_and_scene_identity() {
    let base = RenderEnvironmentCaptureRequest::with_revisions(
        "atrium",
        [1.0, 2.0, 3.0],
        0.25,
        512.0,
        256,
        SourceCubemapPrefilterQuality::Normal,
        9,
        10,
        11,
    )
    .unwrap();
    let same = base.clone();
    let high = base
        .clone()
        .with_quality(SourceCubemapPrefilterQuality::High);
    let layer_7 = base
        .clone()
        .with_capture_layer_mask(RenderLayerSet::layer(7));
    let newer_scene = RenderEnvironmentCaptureRequest::with_revisions(
        "atrium",
        [1.0, 2.0, 3.0],
        0.25,
        512.0,
        256,
        SourceCubemapPrefilterQuality::Normal,
        12,
        10,
        11,
    )
    .unwrap();

    assert_eq!(base.ibl_bake_key(), same.ibl_bake_key());
    assert_ne!(base.ibl_bake_key(), high.ibl_bake_key());
    assert_ne!(base.ibl_bake_key(), layer_7.ibl_bake_key());
    assert_ne!(base.ibl_bake_key(), newer_scene.ibl_bake_key());

    let positive_zero_position =
        RenderEnvironmentCaptureRequest::new("signed-zero", [0.0, 1.0, 2.0], 1).unwrap();
    let negative_zero_position =
        RenderEnvironmentCaptureRequest::new("signed-zero", [-0.0, 1.0, 2.0], 1).unwrap();
    assert_eq!(positive_zero_position, negative_zero_position);
    assert_eq!(
        positive_zero_position.ibl_bake_key(),
        negative_zero_position.ibl_bake_key()
    );
    assert_eq!(
        negative_zero_position.position()[0].to_bits(),
        0.0_f32.to_bits()
    );
}

#[test]
fn capture_layer_mask_is_explicit_and_defaults_to_all_scene_schema_v1_layers() {
    let default_request = RenderEnvironmentCaptureRequest::new("atrium", [0.0; 3], 1).unwrap();
    let sky_only = default_request
        .clone()
        .with_capture_layer_mask(RenderLayerSet::none());

    assert_eq!(
        default_request
            .capture_layer_mask()
            .to_scene_schema_v1_mask_lossy(),
        u32::MAX
    );
    assert!(sky_only.capture_layer_mask().is_empty());
    assert_ne!(default_request.ibl_bake_key(), sky_only.ibl_bake_key());
    let default_output = RenderEnvironmentCaptureOutputIdentity::from_request(&default_request);
    let sky_only_output = RenderEnvironmentCaptureOutputIdentity::from_request(&sky_only);
    assert_eq!(
        default_output
            .capture_layer_mask()
            .to_scene_schema_v1_mask_lossy(),
        u32::MAX
    );
    assert!(sky_only_output.capture_layer_mask().is_empty());
    assert_ne!(default_output, sky_only_output);
}

#[test]
fn capture_ibl_key_versions_raster_source_independently_of_filter_recipe() {
    let request = RenderEnvironmentCaptureRequest::new("atrium", [1.0, 2.0, 3.0], 9).unwrap();

    assert_eq!(
        request.ibl_bake_key(),
        request.ibl_bake_key_with_raster_algorithm_version(
            RENDER_ENVIRONMENT_CAPTURE_RASTER_ALGORITHM_VERSION,
        ),
    );
    assert_ne!(
        request.ibl_bake_key(),
        request.ibl_bake_key_with_raster_algorithm_version(
            RENDER_ENVIRONMENT_CAPTURE_RASTER_ALGORITHM_VERSION - 1,
        ),
    );
}

#[test]
fn status_rejects_output_before_terminal_phase() {
    let request = RenderEnvironmentCaptureRequest::new("atrium", [0.0; 3], 12).unwrap();
    let handle = RenderEnvironmentCaptureHandle::new(7).unwrap();
    let output = RenderEnvironmentCaptureOutputIdentity::from_request(&request);
    assert_eq!(
        RenderEnvironmentCaptureStatus::queued(handle)
            .unwrap()
            .phase(),
        RenderEnvironmentCapturePhase::Queued
    );
    assert_eq!(
        RenderEnvironmentCaptureStatus::new(
            handle,
            RenderEnvironmentCapturePhase::Capturing,
            1,
            6,
            Some(output),
            None
        ),
        Err(RenderEnvironmentCaptureStatusError::OutputBeforeTerminal)
    );
    assert_eq!(
        RenderEnvironmentCaptureStatus::new(
            handle,
            RenderEnvironmentCapturePhase::Succeeded,
            6,
            6,
            None,
            None
        ),
        Err(RenderEnvironmentCaptureStatusError::MissingSuccessfulOutput)
    );
}
