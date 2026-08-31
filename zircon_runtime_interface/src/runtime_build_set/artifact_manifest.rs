use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ZrRuntimeArtifactIdentityV1, ZrRuntimeArtifactManifestValidationError, ZrRuntimeBuildModeV1,
    ZrRuntimeBuildSetExpectationV1, ZrRuntimeBuildSetId, ZrRuntimeDigestV1,
    ZrRuntimeIdentityEncodingError, ZrRuntimeInterfaceSpecV1, ZrRuntimeTargetModelV1,
};

pub const ZR_RUNTIME_ARTIFACT_MANIFEST_SCHEMA_V1: u32 = 1;

/// Sidecar identity for an internal Runtime DLL. The App validates it before `Library::new`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZrRuntimeArtifactManifestV1 {
    pub schema_version: u32,
    pub build_set_id: ZrRuntimeBuildSetId,
    pub build_mode: ZrRuntimeBuildModeV1,
    pub runtime_features: BTreeSet<String>,
    pub interface_spec_digest: ZrRuntimeDigestV1,
    pub interface_spec: ZrRuntimeInterfaceSpecV1,
    pub payload_schema_digest: ZrRuntimeDigestV1,
    pub target: ZrRuntimeTargetModelV1,
    pub artifact: ZrRuntimeArtifactIdentityV1,
    pub host_artifacts: Vec<ZrRuntimeArtifactIdentityV1>,
    pub capabilities: BTreeSet<String>,
}

impl ZrRuntimeArtifactManifestV1 {
    pub fn derived_build_set_id(
        &self,
    ) -> Result<ZrRuntimeBuildSetId, ZrRuntimeIdentityEncodingError> {
        serde_json::to_vec(&RuntimeBuildSetIdentityV1 {
            artifact: &self.artifact,
            build_mode: self.build_mode,
            capabilities: &self.capabilities,
            host_artifacts: &self.host_artifacts,
            interface_spec_digest: &self.interface_spec_digest,
            payload_schema_digest: &self.payload_schema_digest,
            runtime_features: &self.runtime_features,
            target: &self.target,
        })
        .map(ZrRuntimeDigestV1::sha256)
        .map(ZrRuntimeBuildSetId::from_sha256_digest)
        .map_err(|error| ZrRuntimeIdentityEncodingError::BuildSetEncode {
            message: error.to_string(),
        })
    }

    pub fn validate_against(
        &self,
        expected: &ZrRuntimeBuildSetExpectationV1,
    ) -> Result<(), ZrRuntimeArtifactManifestValidationError> {
        if self.schema_version != ZR_RUNTIME_ARTIFACT_MANIFEST_SCHEMA_V1 {
            return Err(
                ZrRuntimeArtifactManifestValidationError::SchemaVersionMismatch {
                    expected: ZR_RUNTIME_ARTIFACT_MANIFEST_SCHEMA_V1,
                    actual: self.schema_version,
                },
            );
        }
        let derived_build_set_id = self.derived_build_set_id().map_err(|error| {
            ZrRuntimeArtifactManifestValidationError::BuildSetIdentityEncoding {
                message: error.to_string(),
            }
        })?;
        if self.build_set_id != derived_build_set_id {
            return Err(
                ZrRuntimeArtifactManifestValidationError::BuildSetDigestInvalid {
                    expected: derived_build_set_id.as_str().to_owned(),
                    actual: self.build_set_id.as_str().to_owned(),
                },
            );
        }
        self.artifact.validate().map_err(|error| {
            ZrRuntimeArtifactManifestValidationError::ArtifactIdentityInvalid {
                message: error.to_string(),
            }
        })?;
        if let Some(feature) = self.runtime_features.iter().find(|feature| {
            feature.is_empty()
                || !feature.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'-' | b'_')
                })
        }) {
            return Err(
                ZrRuntimeArtifactManifestValidationError::RuntimeFeatureInvalid {
                    feature: feature.clone(),
                },
            );
        }
        if self.host_artifacts.is_empty() {
            return Err(ZrRuntimeArtifactManifestValidationError::HostArtifactsMissing);
        }
        let mut host_file_names = std::collections::BTreeSet::new();
        for host_artifact in &self.host_artifacts {
            host_artifact.validate().map_err(|error| {
                ZrRuntimeArtifactManifestValidationError::HostArtifactIdentityInvalid {
                    message: error.to_string(),
                }
            })?;
            if !host_file_names.insert(&host_artifact.file_name) {
                return Err(
                    ZrRuntimeArtifactManifestValidationError::DuplicateHostArtifact {
                        file_name: host_artifact.file_name.clone(),
                    },
                );
            }
        }
        let actual_spec_digest = self.interface_spec.digest().map_err(|error| {
            ZrRuntimeArtifactManifestValidationError::InterfaceSpecEncoding {
                message: error.to_string(),
            }
        })?;
        if self.interface_spec_digest != actual_spec_digest {
            return Err(
                ZrRuntimeArtifactManifestValidationError::InterfaceSpecDigestInvalid {
                    expected: actual_spec_digest.as_str().to_owned(),
                    actual: self.interface_spec_digest.as_str().to_owned(),
                },
            );
        }
        if self.build_set_id != expected.build_set_id().clone() {
            return Err(ZrRuntimeArtifactManifestValidationError::BuildSetMismatch {
                expected: expected.build_set_id().as_str().to_owned(),
                actual: self.build_set_id.as_str().to_owned(),
            });
        }
        if self.interface_spec != expected.interface_spec().clone() {
            return Err(ZrRuntimeArtifactManifestValidationError::InterfaceSpecMismatch);
        }
        if self.payload_schema_digest != expected.payload_schema_digest().clone() {
            return Err(
                ZrRuntimeArtifactManifestValidationError::PayloadSchemaDigestMismatch {
                    expected: expected.payload_schema_digest().as_str().to_owned(),
                    actual: self.payload_schema_digest.as_str().to_owned(),
                },
            );
        }
        if self.target.validate().is_err() || self.target != expected.target().clone() {
            return Err(
                ZrRuntimeArtifactManifestValidationError::TargetModelMismatch {
                    expected: expected.target().clone(),
                    actual: self.target.clone(),
                },
            );
        }
        if let Some(host_artifact) = expected.host_artifact() {
            let Some(manifest_host) = self
                .host_artifacts
                .iter()
                .find(|artifact| artifact.file_name == host_artifact.file_name)
            else {
                return Err(
                    ZrRuntimeArtifactManifestValidationError::HostArtifactNotInBuildSet {
                        file_name: host_artifact.file_name.clone(),
                    },
                );
            };
            if manifest_host.sha256 != host_artifact.sha256 {
                return Err(
                    ZrRuntimeArtifactManifestValidationError::HostArtifactDigestMismatch {
                        file_name: host_artifact.file_name.clone(),
                    },
                );
            }
        }
        let missing = expected
            .required_capabilities()
            .difference(&self.capabilities)
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(
                ZrRuntimeArtifactManifestValidationError::MissingRequiredCapabilities {
                    capabilities: missing,
                },
            );
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct RuntimeBuildSetIdentityV1<'a> {
    artifact: &'a ZrRuntimeArtifactIdentityV1,
    build_mode: ZrRuntimeBuildModeV1,
    capabilities: &'a BTreeSet<String>,
    host_artifacts: &'a [ZrRuntimeArtifactIdentityV1],
    interface_spec_digest: &'a ZrRuntimeDigestV1,
    payload_schema_digest: &'a ZrRuntimeDigestV1,
    runtime_features: &'a BTreeSet<String>,
    target: &'a ZrRuntimeTargetModelV1,
}
