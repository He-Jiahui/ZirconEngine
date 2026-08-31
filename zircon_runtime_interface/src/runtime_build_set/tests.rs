use std::collections::BTreeSet;

use super::{
    ZrRuntimeArtifactIdentityV1, ZrRuntimeArtifactManifestV1,
    ZrRuntimeArtifactManifestValidationError, ZrRuntimeBuildModeV1, ZrRuntimeBuildSetExpectationV1,
    ZrRuntimeBuildSetId, ZrRuntimeDigestV1, ZrRuntimeInterfaceSpecV1,
    ZrRuntimeModuleCompositionReceiptV1, ZrRuntimeModuleCompositionTargetV1,
    ZrRuntimeModuleProfileV1, ZrRuntimeSessionProfileV1, ZrRuntimeTargetModelV1,
    ZR_RUNTIME_ARTIFACT_MANIFEST_SCHEMA_V1, ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1,
};

const OTHER_BUILD_SET_ID: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
const ARTIFACT_DIGEST: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const HOST_ARTIFACT_DIGEST: &str =
    "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100";

#[test]
fn module_composition_receipt_roundtrips_typed_graph_and_session_identity() {
    let receipt = ZrRuntimeModuleCompositionReceiptV1::new(
        17,
        42,
        ZrRuntimeModuleCompositionTargetV1::EditorHost,
        Some(ZrRuntimeModuleProfileV1::Editor),
        ZrRuntimeSessionProfileV1::Editor,
        ZrRuntimeDigestV1::parse(ARTIFACT_DIGEST).unwrap(),
    );

    let json = serde_json::to_value(&receipt).expect("serialize composition receipt");
    let decoded: ZrRuntimeModuleCompositionReceiptV1 =
        serde_json::from_value(json).expect("deserialize composition receipt");

    assert_eq!(decoded, receipt);
    assert_eq!(
        decoded.schema_version,
        ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1
    );
    assert_eq!(decoded.catalog_generation, 17);
    assert_eq!(decoded.source_manifest_fingerprint, 42);
    assert_eq!(
        decoded.target_mode,
        ZrRuntimeModuleCompositionTargetV1::EditorHost
    );
    assert_eq!(
        decoded.module_profile,
        Some(ZrRuntimeModuleProfileV1::Editor)
    );
    assert_eq!(decoded.session_profile, ZrRuntimeSessionProfileV1::Editor);
    assert_eq!(decoded.composition_hash.as_str(), ARTIFACT_DIGEST);
}

#[test]
fn runtime_identity_digests_reject_non_hex_lowercase_characters() {
    let non_hex = "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg";

    assert!(ZrRuntimeDigestV1::parse(non_hex).is_err());
    assert!(ZrRuntimeBuildSetId::parse(non_hex).is_err());
}

#[test]
fn artifact_manifest_accepts_the_current_interface_build_set_and_target() {
    let (expected, manifest) = fixture();

    assert_eq!(manifest.validate_against(&expected), Ok(()));
}

#[test]
fn embedded_interface_spec_is_the_runtime_api_v8_source_of_truth() {
    let spec = ZrRuntimeInterfaceSpecV1::current().unwrap();

    assert_eq!(spec.family, "zircon.runtime.internal");
    assert_eq!(spec.spec_version, 1);
    assert_eq!(
        spec.runtime_api_version,
        crate::ZIRCON_RUNTIME_API_VERSION_V8
    );
    let entry_symbol = std::str::from_utf8(crate::ZR_RUNTIME_GET_API_SYMBOL_V8)
        .unwrap()
        .trim_end_matches('\0');
    assert_eq!(spec.entry_symbol, entry_symbol);
    assert_eq!(
        spec.runtime_api_required_slots,
        crate::runtime_build_set::ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES
            .iter()
            .map(|slot| (*slot).to_owned())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        spec.runtime_api_optional_slots,
        crate::runtime_build_set::ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES
            .iter()
            .map(|slot| (*slot).to_owned())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        spec.host_api_optional_slots,
        crate::runtime_build_set::ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES
            .iter()
            .map(|slot| (*slot).to_owned())
            .collect::<Vec<_>>()
    );
}

#[test]
fn artifact_manifest_rejects_a_build_set_id_that_is_not_bound_to_its_artifacts() {
    let (expected, mut manifest) = fixture();
    manifest.build_set_id = ZrRuntimeBuildSetId::parse(OTHER_BUILD_SET_ID).unwrap();

    assert!(matches!(
        manifest.validate_against(&expected),
        Err(ZrRuntimeArtifactManifestValidationError::BuildSetDigestInvalid { .. })
    ));
}

#[test]
fn artifact_manifest_rejects_an_interface_spec_digest_that_does_not_describe_its_spec() {
    let (_, mut manifest) = fixture();
    manifest.interface_spec_digest = ZrRuntimeDigestV1::parse(ARTIFACT_DIGEST).unwrap();
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected = expectation_for_manifest(&manifest, current_target());

    assert!(matches!(
        manifest.validate_against(&expected),
        Err(ZrRuntimeArtifactManifestValidationError::InterfaceSpecDigestInvalid { .. })
    ));
}

#[test]
fn artifact_manifest_rejects_a_payload_schema_digest_from_another_protocol() {
    let (_, mut manifest) = fixture();
    manifest.payload_schema_digest = ZrRuntimeDigestV1::parse(ARTIFACT_DIGEST).unwrap();
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected = expectation_for_manifest(&manifest, current_target());

    assert!(matches!(
        manifest.validate_against(&expected),
        Err(ZrRuntimeArtifactManifestValidationError::PayloadSchemaDigestMismatch { .. })
    ));
}

#[test]
fn artifact_manifest_rejects_a_different_runtime_data_model() {
    let (expected, mut manifest) = fixture();
    manifest.target.pointer_width = 32;
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected = expectation_for_manifest(&manifest, expected.target().clone());

    assert!(matches!(
        manifest.validate_against(&expected),
        Err(ZrRuntimeArtifactManifestValidationError::TargetModelMismatch { .. })
    ));
}

#[test]
fn artifact_manifest_rejects_a_host_executable_from_another_build_set() {
    let (_, mut manifest) = fixture();
    manifest.host_artifacts[0].sha256 = ZrRuntimeDigestV1::parse(ARTIFACT_DIGEST).unwrap();
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected =
        expectation_for_manifest(&manifest, current_target()).with_host_artifact(host_artifact());

    assert_eq!(
        manifest.validate_against(&expected),
        Err(
            ZrRuntimeArtifactManifestValidationError::HostArtifactDigestMismatch {
                file_name: "zircon_editor.exe".to_owned(),
            }
        )
    );
}

#[test]
fn artifact_manifest_rejects_missing_required_capabilities() {
    let (_, mut manifest) = fixture();
    manifest.capabilities.clear();
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected = expectation_for_manifest(&manifest, current_target());

    assert_eq!(
        manifest.validate_against(&expected),
        Err(
            ZrRuntimeArtifactManifestValidationError::MissingRequiredCapabilities {
                capabilities: vec!["runtime.viewport.present".to_owned()],
            }
        )
    );
}

#[test]
fn artifact_manifest_rejects_an_empty_or_duplicate_host_artifact_set() {
    let (_, mut manifest) = fixture();
    manifest.host_artifacts.clear();
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected = expectation_for_manifest(&manifest, current_target());

    assert_eq!(
        manifest.validate_against(&expected),
        Err(ZrRuntimeArtifactManifestValidationError::HostArtifactsMissing)
    );

    let (_, mut manifest) = fixture();
    manifest.host_artifacts.push(host_artifact());
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected = expectation_for_manifest(&manifest, current_target());

    assert_eq!(
        manifest.validate_against(&expected),
        Err(
            ZrRuntimeArtifactManifestValidationError::DuplicateHostArtifact {
                file_name: "zircon_editor.exe".to_owned(),
            }
        )
    );
}

#[test]
fn artifact_manifest_rejects_an_invalid_runtime_feature_name() {
    let (_, mut manifest) = fixture();
    manifest.runtime_features.insert("Target Client".to_owned());
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected = expectation_for_manifest(&manifest, current_target());

    assert_eq!(
        manifest.validate_against(&expected),
        Err(
            ZrRuntimeArtifactManifestValidationError::RuntimeFeatureInvalid {
                feature: "Target Client".to_owned(),
            }
        )
    );
}

#[test]
fn artifact_manifest_deserialization_rejects_unknown_fields() {
    let (_, manifest) = fixture();
    let mut value = serde_json::to_value(manifest).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_owned(), serde_json::Value::Null);

    assert!(serde_json::from_value::<ZrRuntimeArtifactManifestV1>(value).is_err());
}

fn fixture() -> (ZrRuntimeBuildSetExpectationV1, ZrRuntimeArtifactManifestV1) {
    let mut manifest = manifest_for(current_target());
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let expected = expectation_for_manifest(&manifest, manifest.target.clone());
    (expected, manifest)
}

fn expectation_for_manifest(
    manifest: &ZrRuntimeArtifactManifestV1,
    target: ZrRuntimeTargetModelV1,
) -> ZrRuntimeBuildSetExpectationV1 {
    ZrRuntimeBuildSetExpectationV1::new(
        manifest.build_set_id.clone(),
        target,
        ["runtime.viewport.present".to_owned()],
    )
    .unwrap()
}

fn manifest_for(target: ZrRuntimeTargetModelV1) -> ZrRuntimeArtifactManifestV1 {
    let interface_spec = ZrRuntimeInterfaceSpecV1::current().unwrap();
    ZrRuntimeArtifactManifestV1 {
        schema_version: ZR_RUNTIME_ARTIFACT_MANIFEST_SCHEMA_V1,
        build_set_id: ZrRuntimeBuildSetId::parse(OTHER_BUILD_SET_ID).unwrap(),
        build_mode: ZrRuntimeBuildModeV1::Debug,
        runtime_features: BTreeSet::from(["target-client".to_owned()]),
        interface_spec_digest: interface_spec.digest().unwrap(),
        interface_spec,
        payload_schema_digest: super::current_runtime_payload_schema_set_digest(),
        target,
        artifact: ZrRuntimeArtifactIdentityV1::new(
            "zircon_runtime.dll",
            ZrRuntimeDigestV1::parse(ARTIFACT_DIGEST).unwrap(),
        )
        .unwrap(),
        host_artifacts: vec![host_artifact()],
        capabilities: BTreeSet::from(["runtime.viewport.present".to_owned()]),
    }
}

fn current_target() -> ZrRuntimeTargetModelV1 {
    ZrRuntimeTargetModelV1::current()
}

fn host_artifact() -> ZrRuntimeArtifactIdentityV1 {
    ZrRuntimeArtifactIdentityV1::new(
        "zircon_editor.exe",
        ZrRuntimeDigestV1::parse(HOST_ARTIFACT_DIGEST).unwrap(),
    )
    .unwrap()
}
