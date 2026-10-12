use super::*;

#[test]
fn capture_request_json_roundtrip_preserves_quality_and_clip_contract() {
    let request = ReflectionProbeCaptureRequest::new(
        "probe-lobby",
        AssetUri::parse("res://generated/reflection_probes/lobby.zcube").unwrap(),
        [1.0, 2.0, 3.0],
        7,
    )
    .with_clip_planes(0.25, 512.0)
    .with_face_size(256)
    .with_quality(ReflectionProbeCaptureQuality::High)
    .with_capture_layer_mask(0x0000_0042);

    let json = request.encode_json().unwrap();
    let decoded = ReflectionProbeCaptureRequest::decode_json(&json).unwrap();

    assert_eq!(decoded, request);
    assert_eq!(decoded.capture_layer_mask, 0x0000_0042);
    let ibl_request = decoded.ibl_bake_request([1, 2, 3, 4]);
    assert_eq!(ibl_request.source_mip_count(), 9);
    assert_eq!(ibl_request.pmrem_mip_count(), 8);

    let render_request = decoded.render_request().unwrap();
    assert_eq!(render_request.capture_id(), "probe-lobby");
    assert_eq!(render_request.scene_revision(), 7);
    assert_eq!(render_request.environment_revision(), 7);
    assert_eq!(render_request.output_generation(), 7);
    assert_eq!(
        render_request.quality(),
        SourceCubemapPrefilterQuality::High
    );
    assert_eq!(
        render_request
            .capture_layer_mask()
            .to_scene_schema_v1_mask_lossy(),
        0x0000_0042
    );
    assert_eq!(
        render_request.persistence_output_uri(),
        Some("res://generated/reflection_probes/lobby.zcube")
    );
    assert!(render_request.persistence_artifact_request().is_none());
    assert_eq!(decoded.source_hash(), None);

    let hashed = decoded
        .clone()
        .with_source_hash([9, 8, 7, 6])
        .render_request()
        .unwrap();
    assert_eq!(
        decoded.clone().with_source_hash([9, 8, 7, 6]).source_hash(),
        Some([9, 8, 7, 6])
    );
    assert_eq!(
        hashed.persistence_artifact_request(),
        Some(decoded.ibl_bake_request([9, 8, 7, 6]))
    );
    let runtime_cache_request = hashed
        .runtime_cache_artifact_request()
        .expect("hashed capture must request runtime-cache persistence");
    assert_eq!(runtime_cache_request.bake_key(), hashed.ibl_bake_key());
    assert_ne!(
        runtime_cache_request.bake_key(),
        hashed.persistence_artifact_request().unwrap().bake_key()
    );
    assert_eq!(
        runtime_cache_request.required_contents(),
        hashed
            .persistence_artifact_request()
            .unwrap()
            .required_contents()
    );

    let explicit = decoded
        .render_request_with_artifact_request(ibl_request)
        .unwrap();
    assert_eq!(explicit.persistence_artifact_request(), Some(ibl_request));
}

#[test]
fn capture_request_rejects_non_power_of_two_face_size() {
    let request = ReflectionProbeCaptureRequest::new(
        "probe",
        AssetUri::parse("res://generated/reflection_probes/probe.zcube").unwrap(),
        [0.0; 3],
        1,
    )
    .with_face_size(192);

    assert_eq!(
        request.validate(),
        Err(ReflectionProbeCaptureRequestError::InvalidFaceSize(192))
    );
}

#[test]
fn capture_request_rejects_non_project_output_uri() {
    let request = ReflectionProbeCaptureRequest::new(
        "probe",
        AssetUri::parse("lib://probes/probe.zcube").unwrap(),
        [0.0; 3],
        1,
    );

    assert_eq!(
        request.validate(),
        Err(
            ReflectionProbeCaptureRequestError::UnsupportedOutputUriScheme(
                ResourceScheme::Library,
            )
        )
    );
}

#[test]
fn capture_request_requires_an_unlabeled_zcube_output() {
    let labeled = ReflectionProbeCaptureRequest::new(
        "probe",
        AssetUri::parse("res://generated/reflection_probes/probe.zcube#source").unwrap(),
        [0.0; 3],
        1,
    );
    let wrong_extension = ReflectionProbeCaptureRequest::new(
        "probe",
        AssetUri::parse("res://generated/reflection_probes/probe.bin").unwrap(),
        [0.0; 3],
        1,
    );

    assert_eq!(
        labeled.validate(),
        Err(ReflectionProbeCaptureRequestError::OutputUriHasLabel)
    );
    assert_eq!(
        wrong_extension.validate(),
        Err(
            ReflectionProbeCaptureRequestError::UnsupportedOutputUriExtension(
                "generated/reflection_probes/probe.bin".to_owned(),
            )
        )
    );
}

#[test]
fn capture_request_hard_rejects_v1_without_an_explicit_capture_mask() {
    let v1 = r#"{
            "schema_version": 1,
            "probe_id": "legacy",
            "output_uri": "lib://probes/legacy.zcube",
            "position": [0.0, 0.0, 0.0],
            "near_plane": 0.1,
            "far_plane": 200.0,
            "face_size": 128,
            "quality": "normal",
            "source_revision": 1
        }"#;

    assert!(ReflectionProbeCaptureRequest::decode_json(v1).is_err());
}
