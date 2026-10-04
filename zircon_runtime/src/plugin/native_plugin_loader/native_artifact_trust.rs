use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ExportTargetPlatform;
use crate::plugin::{PluginModuleKind, PluginPackageManifest, PluginPackageRole};

use super::native_artifact_staging::NativePluginArtifactStaging;
use super::NativePluginCandidate;

const HASH_BUFFER_BYTES: usize = 64 * 1024;
const MAX_ADMITTED_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
const MAX_CAPTURED_NATIVE_IMAGE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginArtifactDigest {
    pub sha256: String,
    pub byte_length: u64,
}

impl NativePluginArtifactDigest {
    pub fn capture(path: impl AsRef<Path>) -> Result<Self, NativePluginArtifactAdmissionError> {
        digest_path(path.as_ref())
    }

    pub(crate) fn capture_file(
        file: &File,
        path: impl AsRef<Path>,
    ) -> Result<Self, NativePluginArtifactAdmissionError> {
        digest_file(file, path.as_ref())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativePluginArtifactTrust {
    TrustedLocalFirstParty {
        authority_id: String,
    },
    SignedPackage {
        signer_id: String,
        algorithm: String,
    },
    IsolatedUntrusted {
        reason: String,
    },
}

impl NativePluginArtifactTrust {
    fn in_process_identity(&self) -> Result<&str, NativePluginArtifactAdmissionError> {
        match self {
            Self::TrustedLocalFirstParty { authority_id } if !authority_id.trim().is_empty() => {
                Ok(authority_id)
            }
            Self::SignedPackage { .. } => {
                Err(NativePluginArtifactAdmissionError::SignatureVerificationUnavailable)
            }
            Self::TrustedLocalFirstParty { .. } => {
                Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                    reason: "trusted local artifacts require a non-empty authority identity"
                        .to_string(),
                })
            }
            Self::IsolatedUntrusted { reason } => {
                Err(NativePluginArtifactAdmissionError::IsolationRequired {
                    reason: reason.clone(),
                })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginArtifactTarget {
    pub runtime_mode: RuntimeTargetMode,
    pub platform: ExportTargetPlatform,
}

impl NativePluginArtifactTarget {
    pub const fn new(runtime_mode: RuntimeTargetMode, platform: ExportTargetPlatform) -> Self {
        Self {
            runtime_mode,
            platform,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginArtifactExpectation {
    pub plugin_id: String,
    pub package_id: String,
    pub manifest_digest: NativePluginArtifactDigest,
    pub library_digest: NativePluginArtifactDigest,
    pub trust: NativePluginArtifactTrust,
    pub target: NativePluginArtifactTarget,
    pub module_kinds: Vec<PluginModuleKind>,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<NativePluginArtifactDependency>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginArtifactDependency {
    pub file_name: String,
    pub digest: NativePluginArtifactDigest,
}

impl NativePluginArtifactExpectation {
    #[allow(clippy::too_many_arguments)]
    pub fn trusted_local_first_party(
        plugin_id: impl Into<String>,
        package_id: impl Into<String>,
        manifest_digest: NativePluginArtifactDigest,
        library_digest: NativePluginArtifactDigest,
        authority_id: impl Into<String>,
        target: NativePluginArtifactTarget,
        module_kinds: impl IntoIterator<Item = PluginModuleKind>,
        capabilities: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            package_id: package_id.into(),
            manifest_digest,
            library_digest,
            trust: NativePluginArtifactTrust::TrustedLocalFirstParty {
                authority_id: authority_id.into(),
            },
            target,
            module_kinds: normalized_module_kinds(module_kinds),
            capabilities: normalized_strings(capabilities),
            dependencies: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NativePluginArtifactAuthority {
    expectations: Arc<HashMap<String, Vec<NativePluginArtifactExpectation>>>,
    signed_deadline: Option<chrono::DateTime<chrono::Utc>>,
}

impl NativePluginArtifactAuthority {
    pub fn deny_all() -> Self {
        Self::default()
    }

    pub fn from_expectations(
        expectations: impl IntoIterator<Item = NativePluginArtifactExpectation>,
    ) -> Result<Self, NativePluginArtifactAdmissionError> {
        let expectations = expectations.into_iter();
        let (lower, _) = expectations.size_hint();
        let mut by_plugin: HashMap<String, Vec<NativePluginArtifactExpectation>> =
            HashMap::with_capacity(lower);
        for expectation in expectations {
            expectation.trust.in_process_identity()?;
            validate_expectation(&expectation)?;
            let plugin_id = expectation.plugin_id.clone();
            let artifacts = by_plugin.entry(plugin_id.clone()).or_default();
            if artifacts.iter().any(|artifact| {
                artifact
                    .module_kinds
                    .iter()
                    .any(|kind| expectation.module_kinds.contains(kind))
            }) {
                return Err(
                    NativePluginArtifactAdmissionError::DuplicateAuthorityIdentity { plugin_id },
                );
            }
            artifacts.push(expectation);
        }
        Ok(Self {
            expectations: Arc::new(by_plugin),
            signed_deadline: None,
        })
    }

    pub fn from_verified_package(
        proof: super::VerifiedNativePackageProof,
    ) -> Result<Self, NativePluginArtifactAdmissionError> {
        Self::from_verified_packages([proof])
    }

    /// Combines independently verified installed-package proofs into one admission authority.
    ///
    /// Each proof may describe several module artifacts for one plugin. Separate proofs may not
    /// claim the same plugin identity; callers must resolve an installed package unambiguously
    /// before aggregation. The authority expires at the earliest proof deadline.
    pub fn from_verified_packages(
        proofs: impl IntoIterator<Item = super::VerifiedNativePackageProof>,
    ) -> Result<Self, NativePluginArtifactAdmissionError> {
        let mut by_plugin: HashMap<String, Vec<NativePluginArtifactExpectation>> = HashMap::new();
        let mut proof_count = 0usize;
        let mut signed_deadline = None;

        for proof in proofs {
            proof_count += 1;
            let (expectations, deadline) = proof.into_parts();
            if expectations.is_empty() {
                return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                    reason: "verified package proof contains no native module expectations"
                        .to_string(),
                });
            }
            let proof_plugins = expectations
                .iter()
                .map(|expectation| expectation.plugin_id.clone())
                .collect::<HashSet<_>>();
            if proof_plugins
                .iter()
                .any(|plugin_id| by_plugin.contains_key(plugin_id))
            {
                let plugin_id = proof_plugins
                    .iter()
                    .find(|plugin_id| by_plugin.contains_key(*plugin_id))
                    .cloned()
                    .unwrap_or_default();
                return Err(
                    NativePluginArtifactAdmissionError::DuplicateAuthorityIdentity { plugin_id },
                );
            }

            for expectation in expectations {
                validate_expectation(&expectation)?;
                if !matches!(&expectation.trust, NativePluginArtifactTrust::SignedPackage { signer_id, .. } if !signer_id.is_empty())
                {
                    return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                        reason: "verified package proof requires an authenticated signer"
                            .to_string(),
                    });
                }
                let artifacts = by_plugin.entry(expectation.plugin_id.clone()).or_default();
                if artifacts.iter().any(|artifact| {
                    artifact
                        .module_kinds
                        .iter()
                        .any(|kind| expectation.module_kinds.contains(kind))
                }) {
                    return Err(
                        NativePluginArtifactAdmissionError::DuplicateAuthorityIdentity {
                            plugin_id: expectation.plugin_id,
                        },
                    );
                }
                artifacts.push(expectation);
            }
            signed_deadline = Some(match signed_deadline {
                Some(current) if current <= deadline => current,
                _ => deadline,
            });
        }

        if proof_count == 0 {
            return Ok(Self::deny_all());
        }
        Ok(Self {
            expectations: Arc::new(by_plugin),
            signed_deadline,
        })
    }

    pub fn expectation(&self, plugin_id: &str) -> Option<&NativePluginArtifactExpectation> {
        self.expectations.get(plugin_id)?.first()
    }

    pub fn validate_runtime_target(
        &self,
        runtime_mode: RuntimeTargetMode,
    ) -> Result<(), NativePluginArtifactAdmissionError> {
        for expectation in self.expectations.values().flatten() {
            if expectation.target.runtime_mode != runtime_mode {
                return Err(NativePluginArtifactAdmissionError::TargetMismatch {
                    plugin_id: expectation.plugin_id.clone(),
                    reason: format!(
                        "authority targets {:?}, product targets {:?}",
                        expectation.target.runtime_mode, runtime_mode
                    ),
                });
            }
        }
        Ok(())
    }

    /// Reads build-time expectations embedded in a trusted executable, never a candidate sidecar.
    pub fn from_embedded_build_json(
        source: &'static str,
    ) -> Result<Self, NativePluginArtifactAdmissionError> {
        let expectations: Vec<NativePluginArtifactExpectation> = serde_json::from_str(source)
            .map_err(
                |error| NativePluginArtifactAdmissionError::InvalidAuthority {
                    reason: error.to_string(),
                },
            )?;
        Self::from_expectations(expectations)
    }

    /// Captures an explicitly selected source package while producing the trusted product binary.
    pub(crate) fn capture_trusted_build_package(
        expected_plugin_id: &str,
        source_manifest: &Path,
        authority_id: &str,
        target: NativePluginArtifactTarget,
    ) -> Result<Vec<NativePluginArtifactExpectation>, NativePluginArtifactAdmissionError> {
        let source = std::fs::read_to_string(source_manifest).map_err(|error| {
            NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: error.to_string(),
            }
        })?;
        let manifest: PluginPackageManifest = toml::from_str(&source).map_err(|error| {
            NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: error.to_string(),
            }
        })?;
        if !manifest.package_role.is_product_catalog_eligible() {
            return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: format!(
                    "native build authority cannot capture package {} with role {:?}",
                    manifest.id, manifest.package_role
                ),
            });
        }
        let module_kinds = [PluginModuleKind::Runtime, PluginModuleKind::Editor]
            .into_iter()
            .filter(|kind| {
                manifest
                    .modules
                    .iter()
                    .chain(
                        manifest
                            .feature_extensions
                            .iter()
                            .flat_map(|feature| feature.modules.iter()),
                    )
                    .any(|module| module.kind == *kind)
            })
            .collect::<Vec<_>>();
        Self::capture_trusted_local_package(
            expected_plugin_id,
            source_manifest,
            authority_id,
            target,
            module_kinds,
        )
    }

    /// Captures an explicitly selected local first-party package for the requested module kinds.
    ///
    /// This is an authority producer, not a discovery shortcut: callers must select the source
    /// manifest and target before invoking it. The resulting expectations pin the exact manifest,
    /// library, dependency closure, capability set, and trusted local authority identity.
    pub fn capture_trusted_local_package(
        expected_plugin_id: &str,
        source_manifest: &Path,
        authority_id: &str,
        target: NativePluginArtifactTarget,
        module_kinds: impl IntoIterator<Item = PluginModuleKind>,
    ) -> Result<Vec<NativePluginArtifactExpectation>, NativePluginArtifactAdmissionError> {
        let source = std::fs::read_to_string(source_manifest).map_err(|error| {
            NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: error.to_string(),
            }
        })?;
        let manifest: PluginPackageManifest = toml::from_str(&source).map_err(|error| {
            NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: error.to_string(),
            }
        })?;
        if manifest.id != expected_plugin_id {
            return Err(NativePluginArtifactAdmissionError::IdentityMismatch {
                plugin_id: expected_plugin_id.to_string(),
                expected: expected_plugin_id.to_string(),
                actual: manifest.id,
            });
        }
        if !manifest.package_role.is_product_catalog_eligible() {
            return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: format!(
                    "local native authority cannot capture package {} with role {:?}",
                    manifest.id, manifest.package_role
                ),
            });
        }
        let candidate = NativePluginCandidate {
            plugin_id: expected_plugin_id.to_string(),
            package_manifest: manifest,
            manifest_path: source_manifest.to_path_buf(),
            library_path: PathBuf::new(),
        };
        let module_kinds = normalized_module_kinds(module_kinds);
        if module_kinds.is_empty()
            || module_kinds
                .iter()
                .any(|kind| !matches!(kind, PluginModuleKind::Runtime | PluginModuleKind::Editor))
        {
            return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: "local native authority requires runtime or editor module kinds"
                    .to_string(),
            });
        }
        if module_kinds.iter().any(|kind| {
            !candidate
                .package_manifest
                .modules
                .iter()
                .chain(
                    candidate
                        .package_manifest
                        .feature_extensions
                        .iter()
                        .flat_map(|feature| feature.modules.iter()),
                )
                .any(|module| module.kind == *kind)
        }) {
            return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: format!(
                    "local native authority requested an undeclared module for {}",
                    candidate.plugin_id
                ),
            });
        }
        let manifest_digest = NativePluginArtifactDigest::capture(source_manifest)?;
        super::candidate_from_manifest::native_library_paths_for_candidate(
            &candidate,
            &module_kinds,
        )
        .into_iter()
        .map(|(path, kinds)| {
            let mut expectation = NativePluginArtifactExpectation::trusted_local_first_party(
                expected_plugin_id,
                candidate.package_manifest.package_id(),
                manifest_digest.clone(),
                NativePluginArtifactDigest::capture(&path)?,
                authority_id,
                target.clone(),
                kinds.iter().copied(),
                capabilities_for_modules(&candidate.package_manifest, &kinds),
            );
            capture_native_dependency_closure(&path, &mut expectation.dependencies)?;
            Ok(expectation)
        })
        .collect()
    }

    pub(crate) fn admit(
        &self,
        candidate: &NativePluginCandidate,
        library_path: &Path,
        requested_module_kinds: &[PluginModuleKind],
    ) -> Result<NativePluginArtifactAdmissionReceipt, NativePluginArtifactAdmissionError> {
        let package_role = candidate.package_manifest.package_role;
        if !package_role.is_product_catalog_eligible() {
            return Err(NativePluginArtifactAdmissionError::IneligiblePackageRole {
                plugin_id: candidate.plugin_id.clone(),
                role: package_role,
            });
        }
        let expectation = self
            .expectations
            .get(&candidate.plugin_id)
            .and_then(|artifacts| {
                artifacts.iter().find(|artifact| {
                    requested_module_kinds
                        .iter()
                        .all(|kind| artifact.module_kinds.contains(kind))
                })
            })
            .ok_or_else(|| NativePluginArtifactAdmissionError::MissingAuthority {
                plugin_id: candidate.plugin_id.clone(),
            })?;
        let trust_identity = match (&expectation.trust, self.signed_deadline) {
            (NativePluginArtifactTrust::SignedPackage { signer_id, .. }, Some(deadline)) => {
                if chrono::Utc::now() >= deadline {
                    return Err(NativePluginArtifactAdmissionError::AuthorityExpired {
                        plugin_id: candidate.plugin_id.clone(),
                    });
                }
                signer_id.as_str()
            }
            _ => expectation.trust.in_process_identity()?,
        };
        if candidate.package_manifest.id != expectation.plugin_id
            || candidate.plugin_id != expectation.plugin_id
            || candidate.package_manifest.package_id() != expectation.package_id
        {
            return Err(NativePluginArtifactAdmissionError::IdentityMismatch {
                plugin_id: candidate.plugin_id.clone(),
                expected: format!("{} ({})", expectation.plugin_id, expectation.package_id),
                actual: format!(
                    "{} ({})",
                    candidate.package_manifest.id,
                    candidate.package_manifest.package_id()
                ),
            });
        }
        ensure_target_supported(&candidate.package_manifest, &expectation.target)?;

        let requested = normalized_module_kinds(requested_module_kinds.iter().copied());
        if requested
            .iter()
            .any(|module_kind| !expectation.module_kinds.contains(module_kind))
        {
            return Err(NativePluginArtifactAdmissionError::ModuleKindMismatch {
                plugin_id: candidate.plugin_id.clone(),
                expected: expectation.module_kinds.clone(),
                actual: requested,
            });
        }

        let actual_capabilities =
            capabilities_for_modules(&candidate.package_manifest, requested_module_kinds);
        if actual_capabilities != expectation.capabilities {
            return Err(NativePluginArtifactAdmissionError::CapabilityMismatch {
                plugin_id: candidate.plugin_id.clone(),
                expected: expectation.capabilities.clone(),
                actual: actual_capabilities,
            });
        }

        verify_manifest(candidate, &expectation.manifest_digest)?;
        let admitted_path = library_path.canonicalize().map_err(|error| {
            NativePluginArtifactAdmissionError::ArtifactRead {
                plugin_id: candidate.plugin_id.clone(),
                artifact: "native library",
                path: library_path.to_path_buf(),
                reason: error.to_string(),
            }
        })?;
        let staging = NativePluginArtifactStaging::prepare(
            &candidate.plugin_id,
            &admitted_path,
            &expectation.library_digest,
            &expectation.dependencies,
        )?;
        let staged_path = staging.library_path().to_path_buf();
        let library_guard = open_library_for_admission(&staged_path)?;
        let actual_digest = digest_file(&library_guard, &staged_path)?;
        if actual_digest != expectation.library_digest {
            return Err(NativePluginArtifactAdmissionError::DigestMismatch {
                plugin_id: candidate.plugin_id.clone(),
                artifact: "native library",
                path: library_path.to_path_buf(),
                expected: expectation.library_digest.clone(),
                actual: actual_digest,
            });
        }

        if self
            .signed_deadline
            .is_some_and(|deadline| chrono::Utc::now() >= deadline)
        {
            return Err(NativePluginArtifactAdmissionError::AuthorityExpired {
                plugin_id: candidate.plugin_id.clone(),
            });
        }
        let staging = Arc::new(staging);
        #[cfg(windows)]
        NativePluginArtifactStaging::register_admitted(&staging)?;
        Ok(NativePluginArtifactAdmissionReceipt {
            plugin_id: expectation.plugin_id.clone(),
            package_id: expectation.package_id.clone(),
            library_path: staged_path,
            library_digest: expectation.library_digest.clone(),
            trust_identity: trust_identity.to_string(),
            target: expectation.target.clone(),
            module_kinds: requested_module_kinds.to_vec(),
            capabilities: expectation.capabilities.clone(),
            library_guard: Arc::new(library_guard),
            staging,
        })
    }
}

#[derive(Clone, Debug)]
pub struct NativePluginArtifactAdmissionReceipt {
    pub plugin_id: String,
    pub package_id: String,
    pub library_path: PathBuf,
    pub library_digest: NativePluginArtifactDigest,
    pub trust_identity: String,
    pub target: NativePluginArtifactTarget,
    pub module_kinds: Vec<PluginModuleKind>,
    pub capabilities: Vec<String>,
    library_guard: Arc<File>,
    staging: Arc<NativePluginArtifactStaging>,
}

impl NativePluginArtifactAdmissionReceipt {
    pub(super) fn admitted_library_path(&self) -> PathBuf {
        let _ = &self.library_guard;
        self.library_path.clone()
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum NativePluginArtifactAdmissionError {
    #[error("native plugin {plugin_id} signed artifact authority has expired")]
    AuthorityExpired { plugin_id: String },
    #[error("native plugin package signature verification is unavailable; signed-package in-process admission denied")]
    SignatureVerificationUnavailable,
    #[error("native plugin {plugin_id} has no artifact authority; in-process loading denied")]
    MissingAuthority { plugin_id: String },
    #[error("native plugin {plugin_id} package role {role:?} is not eligible for in-process product loading")]
    IneligiblePackageRole {
        plugin_id: String,
        role: PluginPackageRole,
    },
    #[error("native plugin artifact authority is invalid: {reason}")]
    InvalidAuthority { reason: String },
    #[error("native plugin authority contains duplicate identity {plugin_id}")]
    DuplicateAuthorityIdentity { plugin_id: String },
    #[error("native plugin requires process isolation before loading: {reason}")]
    IsolationRequired { reason: String },
    #[error("native plugin {plugin_id} identity mismatch: expected {expected}, actual {actual}")]
    IdentityMismatch {
        plugin_id: String,
        expected: String,
        actual: String,
    },
    #[error("native plugin {plugin_id} target mismatch: {reason}")]
    TargetMismatch { plugin_id: String, reason: String },
    #[error(
        "native plugin {plugin_id} module kind mismatch: expected {expected:?}, actual {actual:?}"
    )]
    ModuleKindMismatch {
        plugin_id: String,
        expected: Vec<PluginModuleKind>,
        actual: Vec<PluginModuleKind>,
    },
    #[error(
        "native plugin {plugin_id} capability mismatch: expected {expected:?}, actual {actual:?}"
    )]
    CapabilityMismatch {
        plugin_id: String,
        expected: Vec<String>,
        actual: Vec<String>,
    },
    #[error("native plugin {plugin_id} {artifact} read failed at {}: {reason}", path.display())]
    ArtifactRead {
        plugin_id: String,
        artifact: &'static str,
        path: PathBuf,
        reason: String,
    },
    #[error("native plugin {plugin_id} {artifact} digest mismatch at {}: expected {expected:?}, actual {actual:?}", path.display())]
    DigestMismatch {
        plugin_id: String,
        artifact: &'static str,
        path: PathBuf,
        expected: NativePluginArtifactDigest,
        actual: NativePluginArtifactDigest,
    },
}

fn validate_expectation(
    expectation: &NativePluginArtifactExpectation,
) -> Result<(), NativePluginArtifactAdmissionError> {
    if expectation.plugin_id.trim().is_empty()
        || expectation.package_id.trim().is_empty()
        || expectation.manifest_digest.sha256.len() != 64
        || expectation.library_digest.sha256.len() != 64
        || !expectation
            .manifest_digest
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || !expectation
            .library_digest
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || expectation.module_kinds.is_empty()
    {
        return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
            reason: format!(
                "expectation for {:?} requires identities, SHA-256 digests, and module kinds",
                expectation.plugin_id
            ),
        });
    }
    Ok(())
}

fn ensure_target_supported(
    manifest: &PluginPackageManifest,
    target: &NativePluginArtifactTarget,
) -> Result<(), NativePluginArtifactAdmissionError> {
    if manifest.distribution.is_none() {
        return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
            reason: format!("native plugin {} requires a declared ABI and engine compatibility distribution contract", manifest.id),
        });
    }
    let host_platform = if cfg!(target_os = "windows") {
        Some(ExportTargetPlatform::Windows)
    } else {
        None
    };
    if host_platform != Some(target.platform) {
        return Err(NativePluginArtifactAdmissionError::TargetMismatch {
            plugin_id: manifest.id.clone(),
            reason: format!(
                "artifact target {:?} has no supported immutable load path on this host",
                target.platform
            ),
        });
    }
    if !manifest.supported_targets.is_empty()
        && !manifest.supported_targets.contains(&target.runtime_mode)
    {
        return Err(NativePluginArtifactAdmissionError::TargetMismatch {
            plugin_id: manifest.id.clone(),
            reason: format!("runtime mode {:?} is not declared", target.runtime_mode),
        });
    }
    if !manifest.supported_platforms.is_empty()
        && !manifest.supported_platforms.contains(&target.platform)
    {
        return Err(NativePluginArtifactAdmissionError::TargetMismatch {
            plugin_id: manifest.id.clone(),
            reason: format!("platform {:?} is not declared", target.platform),
        });
    }
    Ok(())
}

fn capabilities_for_modules(
    manifest: &PluginPackageManifest,
    requested_module_kinds: &[PluginModuleKind],
) -> Vec<String> {
    let mut capabilities = manifest.capabilities.clone();
    capabilities.extend(
        manifest
            .modules
            .iter()
            .chain(
                manifest
                    .feature_extensions
                    .iter()
                    .flat_map(|feature| feature.modules.iter()),
            )
            .filter(|module| requested_module_kinds.contains(&module.kind))
            .flat_map(|module| module.capabilities.iter().cloned()),
    );
    normalized_strings(capabilities)
}

fn normalized_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_unstable();
    values.dedup();
    values
}

fn normalized_module_kinds(
    values: impl IntoIterator<Item = PluginModuleKind>,
) -> Vec<PluginModuleKind> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_unstable_by_key(|kind| match kind {
        PluginModuleKind::Runtime => 0,
        PluginModuleKind::Editor => 1,
        PluginModuleKind::Native => 2,
        PluginModuleKind::Vm => 3,
    });
    values.dedup();
    values
}

/// Captures only the package-local DLLs reachable from the admitted library's PE import graph.
/// Unreferenced siblings stay outside the authority, while transitive imports are still pinned
/// before the loader can execute the main image.
fn capture_native_dependency_closure(
    main_path: &Path,
    dependencies: &mut Vec<NativePluginArtifactDependency>,
) -> Result<(), NativePluginArtifactAdmissionError> {
    let native_root =
        main_path
            .parent()
            .ok_or_else(|| NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: "native source has no parent directory".to_string(),
            })?;
    let main_name = main_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| NativePluginArtifactAdmissionError::InvalidAuthority {
            reason: "native main library filename must be UTF-8".to_string(),
        })?
        .to_ascii_lowercase();

    let mut sibling_dlls = HashMap::<String, PathBuf>::new();
    for entry in std::fs::read_dir(native_root).map_err(|error| {
        NativePluginArtifactAdmissionError::InvalidAuthority {
            reason: error.to_string(),
        }
    })? {
        let entry =
            entry.map_err(
                |error| NativePluginArtifactAdmissionError::InvalidAuthority {
                    reason: error.to_string(),
                },
            )?;
        let path = entry.path();
        let Some(extension) = path.extension() else {
            continue;
        };
        if !extension.eq_ignore_ascii_case("dll") {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| {
            NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: error.to_string(),
            }
        })?;
        if file_type.is_symlink() || !file_type.is_file() {
            return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: "native dependency source must be a regular file".to_string(),
            });
        }
        let file_name = entry.file_name();
        let name = file_name.to_str().ok_or_else(|| {
            NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: "native dependency filename must be UTF-8".to_string(),
            }
        })?;
        let key = normalized_native_dll_name(name)?;
        if sibling_dlls.insert(key, path).is_some() {
            return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
                reason: "native dependency source has ambiguous DLL filename".to_string(),
            });
        }
    }

    let mut pending = vec![main_path.to_path_buf()];
    let mut visited = HashSet::new();
    visited.insert(main_path.to_path_buf());
    while let Some(path) = pending.pop() {
        let imports = native_imports_for_capture(&path)?;
        for imported in imports {
            let Some(dependency_path) = sibling_dlls.get(&imported).cloned() else {
                continue;
            };
            if imported == main_name || !visited.insert(dependency_path.clone()) {
                continue;
            }
            let file_name = dependency_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| NativePluginArtifactAdmissionError::InvalidAuthority {
                    reason: "native dependency filename must be UTF-8".to_string(),
                })?;
            dependencies.push(NativePluginArtifactDependency {
                file_name: file_name.to_string(),
                digest: NativePluginArtifactDigest::capture(&dependency_path)?,
            });
            pending.push(dependency_path);
        }
    }
    dependencies.sort_unstable_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(())
}

fn native_imports_for_capture(
    path: &Path,
) -> Result<Vec<String>, NativePluginArtifactAdmissionError> {
    let mut file = open_library_for_admission(path)?;
    let length = file
        .metadata()
        .map_err(|error| NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "native library",
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?
        .len();
    if length > MAX_CAPTURED_NATIVE_IMAGE_BYTES {
        return Err(NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "native library",
            path: path.to_path_buf(),
            reason: "native image exceeds capture budget".to_string(),
        });
    }
    let expected_len =
        usize::try_from(length).map_err(|_| NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "native library",
            path: path.to_path_buf(),
            reason: "native image cannot fit the capture buffer".to_string(),
        })?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(expected_len).map_err(|_| {
        NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "native library",
            path: path.to_path_buf(),
            reason: "native image capture buffer allocation failed".to_string(),
        }
    })?;
    (&mut file)
        .take(length.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "native library",
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    let current_len = file
        .metadata()
        .map_err(|error| NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "native library",
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?
        .len();
    if bytes.len() != expected_len || current_len != length {
        return Err(NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "native library",
            path: path.to_path_buf(),
            reason: "native image changed while imports were captured".to_string(),
        });
    }
    let pe = goblin::pe::PE::parse(&bytes).map_err(|error| {
        NativePluginArtifactAdmissionError::InvalidAuthority {
            reason: format!("native PE parse failed for {}: {error}", path.display()),
        }
    })?;
    pe.libraries
        .iter()
        .map(|name| normalized_native_dll_name(name))
        .collect()
}

fn normalized_native_dll_name(name: &str) -> Result<String, NativePluginArtifactAdmissionError> {
    if name.is_empty()
        || !name.is_ascii()
        || name.trim() != name
        || name.contains(['/', '\\', ':'])
        || !name.to_ascii_lowercase().ends_with(".dll")
        || name.bytes().any(|byte| byte < 32)
    {
        return Err(NativePluginArtifactAdmissionError::InvalidAuthority {
            reason: "native dependency must be one plain DLL filename".to_string(),
        });
    }
    Ok(name.to_ascii_lowercase())
}

fn verify_manifest(
    candidate: &NativePluginCandidate,
    expected: &NativePluginArtifactDigest,
) -> Result<(), NativePluginArtifactAdmissionError> {
    let path = &candidate.manifest_path;
    let read_error = |reason: String| NativePluginArtifactAdmissionError::ArtifactRead {
        plugin_id: candidate.plugin_id.clone(),
        artifact: "manifest",
        path: path.to_path_buf(),
        reason,
    };
    if expected.byte_length > MAX_ADMITTED_MANIFEST_BYTES {
        return Err(read_error(
            "manifest exceeds native admission budget".to_string(),
        ));
    }
    let mut file = open_library_for_admission(path)?;
    let actual = digest_file(&file, path)?;
    if &actual != expected {
        return Err(NativePluginArtifactAdmissionError::DigestMismatch {
            plugin_id: candidate.plugin_id.clone(),
            artifact: "manifest",
            path: path.to_path_buf(),
            expected: expected.clone(),
            actual,
        });
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|error| read_error(error.to_string()))?;
    let mut source = String::with_capacity(expected.byte_length as usize);
    file.take(MAX_ADMITTED_MANIFEST_BYTES + 1)
        .read_to_string(&mut source)
        .map_err(|error| read_error(error.to_string()))?;
    let manifest: PluginPackageManifest =
        toml::from_str(&source).map_err(|error| read_error(error.to_string()))?;
    if manifest != candidate.package_manifest {
        return Err(read_error(
            "discovery manifest no longer matches the verified manifest generation".to_string(),
        ));
    }
    Ok(())
}

fn digest_path(
    path: &Path,
) -> Result<NativePluginArtifactDigest, NativePluginArtifactAdmissionError> {
    let file =
        File::open(path).map_err(|error| NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "artifact",
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;
    digest_file(&file, path)
}

pub(super) fn open_library_for_admission(
    path: &Path,
) -> Result<File, NativePluginArtifactAdmissionError> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Deny write and delete sharing until LoadLibrary owns the verified image.
        const FILE_SHARE_READ: u32 = 1;
        options.share_mode(FILE_SHARE_READ);
    }
    options
        .open(path)
        .map_err(|error| NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<admission>".to_string(),
            artifact: "native library",
            path: path.to_path_buf(),
            reason: error.to_string(),
        })
}

pub(super) fn digest_file(
    file: &File,
    path: &Path,
) -> Result<NativePluginArtifactDigest, NativePluginArtifactAdmissionError> {
    let mut file =
        file.try_clone()
            .map_err(|error| NativePluginArtifactAdmissionError::ArtifactRead {
                plugin_id: "<capture>".to_string(),
                artifact: "artifact",
                path: path.to_path_buf(),
                reason: error.to_string(),
            })?;
    file.seek(SeekFrom::Start(0)).map_err(|error| {
        NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "artifact",
            path: path.to_path_buf(),
            reason: error.to_string(),
        }
    })?;
    let before =
        file.metadata()
            .map_err(|error| NativePluginArtifactAdmissionError::ArtifactRead {
                plugin_id: "<capture>".to_string(),
                artifact: "artifact",
                path: path.to_path_buf(),
                reason: error.to_string(),
            })?;
    let mut sha256 = Sha256::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_BYTES];
    let mut byte_length = 0_u64;
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            NativePluginArtifactAdmissionError::ArtifactRead {
                plugin_id: "<capture>".to_string(),
                artifact: "artifact",
                path: path.to_path_buf(),
                reason: error.to_string(),
            }
        })?;
        if read == 0 {
            break;
        }
        sha256.update(&buffer[..read]);
        byte_length = byte_length.saturating_add(read as u64);
    }
    let after =
        file.metadata()
            .map_err(|error| NativePluginArtifactAdmissionError::ArtifactRead {
                plugin_id: "<capture>".to_string(),
                artifact: "artifact",
                path: path.to_path_buf(),
                reason: error.to_string(),
            })?;
    if before.len() != byte_length || after.len() != byte_length {
        return Err(NativePluginArtifactAdmissionError::ArtifactRead {
            plugin_id: "<capture>".to_string(),
            artifact: "artifact",
            path: path.to_path_buf(),
            reason: "artifact changed while its digest was captured".to_string(),
        });
    }
    Ok(NativePluginArtifactDigest {
        sha256: format!("{:x}", sha256.finalize()),
        byte_length,
    })
}

#[cfg(test)]
#[path = "tests/native_artifact_trust.rs"]
mod tests;
