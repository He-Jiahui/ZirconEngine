use super::*;
use crate::ReflectionProbeCaptureRequestError;
use zircon_runtime::asset::assets::decode_ibl_pmrem_rgba16f_texture;
use zircon_runtime::core::framework::render::{
    source_cubemap_mip_count, source_cubemap_mip_size, IblBakeArtifactContents,
    IblBakeArtifactDescriptor, IblBakeArtifactPayload, IblBakeKey, RenderEnvironmentCaptureHandle,
    RenderEnvironmentCaptureOutputIdentity, RenderEnvironmentCaptureRequest,
    SourceCubemapPrefilterQuality, RGBA16F_TEXEL_SIZE_BYTES, SOURCE_CUBEMAP_FACE_COUNT,
    SOURCE_CUBEMAP_MIN_FACE_SIZE,
};
use zircon_runtime::core::resource::{
    ResourceEventKind, ResourceEventTryRecvError, ResourceScheme,
};

#[test]
fn captured_source_encoder_validates_project_output_before_zcube_encoding() {
    let face_size = SOURCE_CUBEMAP_MIN_FACE_SIZE;
    let mip_count = source_cubemap_mip_count(face_size);
    let source_byte_len = (0..mip_count)
        .map(|mip| {
            let mip_size = source_cubemap_mip_size(face_size, mip) as usize;
            SOURCE_CUBEMAP_FACE_COUNT * mip_size * mip_size * RGBA16F_TEXEL_SIZE_BYTES
        })
        .sum();
    let request = RenderEnvironmentCaptureRequest::with_revisions(
        "atrium",
        [0.0; 3],
        0.1,
        200.0,
        face_size,
        SourceCubemapPrefilterQuality::Normal,
        1,
        1,
        1,
    )
    .unwrap()
    .with_persistence_output_uri("lib://probes/atrium.zcube")
    .unwrap();
    let payload = RenderEnvironmentCaptureSourcePayload::new(
        RenderEnvironmentCaptureHandle::new(1).unwrap(),
        RenderEnvironmentCaptureOutputIdentity::from_request(&request),
        face_size,
        mip_count,
        vec![0; source_byte_len],
    )
    .unwrap();

    assert!(matches!(
        encode_reflection_probe_capture_source(payload),
        Err(CapturedReflectionProbeConsumeError::Request(
            ReflectionProbeCaptureRequestError::UnsupportedOutputUriScheme(ResourceScheme::Library)
        ))
    ));
}

#[test]
fn runtime_cache_registration_requires_explicit_source_hash_before_io() {
    let request = ReflectionProbeCaptureRequest::new(
        "atrium",
        AssetUri::parse("res://generated/reflection_probes/atrium.zcube").unwrap(),
        [0.0, 1.0, 0.0],
        1,
    );
    let placement = CapturedReflectionProbePlacement::box_probe(
        17,
        "lib://probes/atrium.pmrem",
        [4.0, 4.0, 4.0],
        1.0,
    );
    let cache_store = IblBakeArtifactCacheStore::new("E:/zircon-runtime-cache-contract");

    let result = register_captured_reflection_probe_from_runtime_cache(
        &ProjectAssetManager::default(),
        &cache_store,
        &request,
        &placement,
    );

    assert!(matches!(
        result,
        Err(CapturedReflectionProbeConsumeError::MissingSourceHash)
    ));
}

#[test]
fn runtime_cache_registration_stays_distinct_from_asset_derived_source_staging() {
    let source = include_str!("../consume.rs");
    let runtime = source
        .split_once("pub fn register_captured_reflection_probe_from_runtime_cache(")
        .and_then(|(_, tail)| tail.split_once("fn register_captured_reflection_probe_blob"))
        .map(|(runtime, _)| runtime)
        .expect("runtime cache registration owner");

    assert!(runtime.contains("read_runtime_cache(&artifact_request)"));
    assert!(runtime.contains("MissingSourceHash"));
    assert!(runtime.contains("runtime_cache_artifact_request()"));
    assert!(!runtime.contains("ibl_bake_request(source_hash)"));
    assert!(!runtime.contains("IblSourceCubemapStagingStore"));
    assert!(!runtime.contains("write_source_cubemap"));
}

#[test]
fn captured_probe_placement_json_roundtrip_preserves_runtime_fields() {
    let placement = CapturedReflectionProbePlacement::box_probe(
        17,
        "lib://reflection-probes/atrium.pmrem",
        [8.0, 4.0, 6.0],
        1.5,
    );
    let json = placement.encode_json().unwrap();
    let decoded = CapturedReflectionProbePlacement::decode_json(&json).unwrap();

    assert_eq!(decoded, placement);
    decoded.validate().unwrap();
    assert!(!json.contains("bake_timing"));

    let mut legacy = serde_json::to_value(&placement).unwrap();
    legacy.as_object_mut().expect("placement object").insert(
        "bake_timing".to_owned(),
        serde_json::Value::String("EditorManual".to_owned()),
    );
    assert!(CapturedReflectionProbePlacement::decode_json(&legacy.to_string()).is_err());
}

// A descriptor-sized PMREM section is enough here: registration validates layout, not radiance.
fn captured_pmrem_blob(fill: u8) -> IblBakeArtifactBlob {
    let descriptor = IblBakeArtifactDescriptor::current(
        IblBakeKey::source_cubemap(1, [7; 4]),
        128,
        8,
        IblBakeArtifactContents::PMREM,
    );
    let bytes = vec![fill; descriptor.expected_payload_size_bytes()];
    IblBakeArtifactBlob::from_payload(
        IblBakeArtifactPayload::decode(descriptor, &bytes).expect("descriptor-sized PMREM payload"),
    )
}

fn payload_checksum_hex(blob: &IblBakeArtifactBlob) -> String {
    blob.payload_checksum()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

// Unreal Engine/Source/Runtime/CoreUObject/Tests/Serialization/EditorBulkDataTests.cpp:
// FEditorBulkData::Identifiers keeps one identifier across UpdatePayload, while
// ::UpdatePayloadCompressedBuffer and ::SharedBufferWithID tie GetPayloadId to the payload bytes.
// A recapture likewise keeps the PMREM ResourceId; its content identity follows the artifact
// bytes, so revision-keyed probe caches see the replacement (official review R05-R01).
#[test]
fn unreal_equivalent_recaptured_probe_keeps_identity_and_publishes_new_content_revision() {
    let asset_manager = ProjectAssetManager::default();
    let resources = asset_manager.resource_manager();
    let events = resources.subscribe();
    let request = ReflectionProbeCaptureRequest::new(
        "atrium",
        AssetUri::parse("res://generated/reflection_probes/atrium.zcube").unwrap(),
        [0.0, 1.0, 0.0],
        1,
    );
    let placement = CapturedReflectionProbePlacement::box_probe(
        17,
        "lib://probes/atrium.pmrem",
        [4.0, 4.0, 4.0],
        1.0,
    );
    let first_blob = captured_pmrem_blob(0x11);
    let recaptured_blob = captured_pmrem_blob(0x22);

    let first =
        register_captured_reflection_probe_blob(&asset_manager, &request, &placement, &first_blob)
            .unwrap();
    let id = first.texture_id;
    let published = resources
        .registry()
        .get(id)
        .cloned()
        .expect("captured PMREM record");
    assert_eq!(first.probe.baked_cubemap(), Some(id));
    assert_eq!(published.source_hash, payload_checksum_hex(&first_blob));
    let added = events
        .try_recv()
        .expect("first capture publishes the PMREM");
    assert_eq!(added.kind, ResourceEventKind::Added);
    assert_eq!(added.id, id);

    let repeated =
        register_captured_reflection_probe_blob(&asset_manager, &request, &placement, &first_blob)
            .unwrap();
    assert_eq!(repeated.texture_id, id);
    assert!(matches!(
        events.try_recv(),
        Err(ResourceEventTryRecvError::Empty)
    ));
    assert_eq!(
        resources.registry().get(id).map(|record| record.revision),
        Some(published.revision)
    );

    let recaptured = register_captured_reflection_probe_blob(
        &asset_manager,
        &request,
        &placement,
        &recaptured_blob,
    )
    .unwrap();
    assert_eq!(recaptured.texture_id, id);
    let updated = events
        .try_recv()
        .expect("recaptured content publishes an update");
    assert_eq!(updated.kind, ResourceEventKind::Updated);
    assert_eq!(updated.id, id);
    assert_eq!(updated.revision, published.revision + 1);
    let snapshot = asset_manager
        .load_texture_asset_snapshot(id)
        .expect("recaptured PMREM snapshot");
    assert_eq!(snapshot.revision(), updated.revision);
    assert_eq!(
        snapshot.record().source_hash,
        payload_checksum_hex(&recaptured_blob)
    );
    assert_ne!(snapshot.record().source_hash, published.source_hash);
    assert!(decode_ibl_pmrem_rgba16f_texture(&snapshot)
        .expect("PMREM container")
        .iter()
        .all(|byte| *byte == 0x22));
}
