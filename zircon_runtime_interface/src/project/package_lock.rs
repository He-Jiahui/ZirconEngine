use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, str::FromStr};

use super::{ProjectGuid, ProjectManifestDigest};

pub const PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1: u32 = 1;
pub const PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1: u32 = 1;
pub const MAX_PROJECT_PACKAGE_LOCK_ENTRIES: usize = 512;
pub const MAX_PROJECT_PACKAGE_LOCK_CAPABILITIES: usize = 64;
pub const MAX_PROJECT_PACKAGE_LOCK_TEXT: usize = 128;
// Leave space for the existing 64 KiB helper response envelope and newline.
pub const MAX_PROJECT_PACKAGE_LOCK_BYTES: usize = 60 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageLockRuntimeMode {
    ClientRuntime,
    EditorHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageLockPlatform {
    Windows,
    Linux,
    Macos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLockTarget {
    pub runtime_mode: ProjectPackageLockRuntimeMode,
    pub platform: ProjectPackageLockPlatform,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLockProject {
    pub project_guid: ProjectGuid,
    pub manifest_digest: ProjectManifestDigest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLockEntry {
    pub plugin_id: String,
    pub package_id: String,
    pub version: String,
    pub release_revision: String,
    pub artifact_digest: String,
    pub capabilities: Vec<String>,
}

/// Producer provenance captured from the authenticated native policy loader and
/// host package-store generation. The runtime producer obtains these values
/// from `LoadedNativePluginPolicy`; cloud consumers validate shape and digest
/// linkage only. This witness is not a server-verifiable authorization token,
/// so Hub must never grant authority from client-supplied fields alone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLockAuthority {
    pub index_sha256: String,
    pub policy_sha256: String,
    pub build_set_id: String,
    pub capability_digest: String,
    pub provider_revision_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLock {
    pub schema_version: u32,
    pub project: ProjectPackageLockProject,
    pub target: ProjectPackageLockTarget,
    pub authority: ProjectPackageLockAuthority,
    pub entries: Vec<ProjectPackageLockEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageLockUnavailableReason {
    ProviderMissing,
    Incompatible,
    GenerationChanged,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectPackageLockState {
    Present {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        lock: ProjectPackageLock,
        digest: String,
    },
    Unavailable {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        reason: ProjectPackageLockUnavailableReason,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectPackageLockError {
    UnsupportedSchema,
    Capacity,
    InvalidText,
    InvalidIdentity,
    DuplicatePlugin,
    Encoding,
    NonCanonicalWire,
    InvalidVersion,
    InvalidAuthority,
}

impl ProjectPackageLock {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProjectPackageLockError> {
        self.validate()?;
        let mut normalized = self.clone();
        normalized.entries.sort_by(|left, right| {
            left.plugin_id
                .cmp(&right.plugin_id)
                .then(left.package_id.cmp(&right.package_id))
                .then(left.release_revision.cmp(&right.release_revision))
                .then(left.artifact_digest.cmp(&right.artifact_digest))
        });
        for entry in &mut normalized.entries {
            entry.capabilities.sort();
        }
        let bytes =
            serde_json::to_vec(&normalized).map_err(|_| ProjectPackageLockError::Encoding)?;
        if bytes.len() > MAX_PROJECT_PACKAGE_LOCK_BYTES {
            return Err(ProjectPackageLockError::Capacity);
        }
        Ok(bytes)
    }

    pub fn digest(&self) -> Result<String, ProjectPackageLockError> {
        // Local checkout identity authenticates the witness, but is never part of
        // the shared dependency fingerprint: teammates have different project GUIDs.
        let normalized: Self = serde_json::from_slice(&self.canonical_bytes()?)
            .map_err(|_| ProjectPackageLockError::Encoding)?;
        let bytes = serde_json::to_vec(&(
            normalized.schema_version,
            normalized.target,
            normalized.authority,
            normalized.entries,
        ))
        .map_err(|_| ProjectPackageLockError::Encoding)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    pub fn validate(&self) -> Result<(), ProjectPackageLockError> {
        if self.schema_version != PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1 {
            return Err(ProjectPackageLockError::UnsupportedSchema);
        }
        if self.entries.len() > MAX_PROJECT_PACKAGE_LOCK_ENTRIES {
            return Err(ProjectPackageLockError::Capacity);
        }
        let mut plugins = BTreeSet::new();
        for entry in &self.entries {
            if !valid_package_version(&entry.version) {
                return Err(ProjectPackageLockError::InvalidVersion);
            }
            if !valid_plugin_id(&entry.plugin_id)
                || !canonical_uuid(&entry.package_id)
                || !canonical_revision(&entry.release_revision)
                || !is_digest(&entry.artifact_digest)
                || entry.capabilities.len() > MAX_PROJECT_PACKAGE_LOCK_CAPABILITIES
            {
                return Err(ProjectPackageLockError::InvalidIdentity);
            }
            let mut capabilities = BTreeSet::new();
            for capability in &entry.capabilities {
                if !valid_text(capability) || !capabilities.insert(capability) {
                    return Err(ProjectPackageLockError::InvalidText);
                }
            }
            if !plugins.insert(&entry.plugin_id) {
                return Err(ProjectPackageLockError::DuplicatePlugin);
            }
        }
        if !valid_authority(&self.authority) {
            return Err(ProjectPackageLockError::InvalidAuthority);
        }
        Ok(())
    }

    pub fn require_canonical_wire(&self) -> Result<(), ProjectPackageLockError> {
        let canonical = self.canonical_bytes()?;
        let actual = serde_json::to_vec(self).map_err(|_| ProjectPackageLockError::Encoding)?;
        (actual == canonical)
            .then_some(())
            .ok_or(ProjectPackageLockError::NonCanonicalWire)
    }

    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, ProjectPackageLockError> {
        if bytes.len() > MAX_PROJECT_PACKAGE_LOCK_BYTES {
            return Err(ProjectPackageLockError::Capacity);
        }
        let lock: Self =
            serde_json::from_slice(bytes).map_err(|_| ProjectPackageLockError::Encoding)?;
        lock.require_canonical_wire()?;
        Ok(lock)
    }

    pub fn is_empty_selection(&self) -> bool {
        self.entries.is_empty()
    }
}

impl ProjectPackageLockState {
    pub fn present(lock: ProjectPackageLock) -> Result<Self, ProjectPackageLockError> {
        let digest = lock.digest()?;
        Ok(Self::Present {
            schema_version: PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1,
            lock,
            digest,
        })
    }

    pub fn validate(&self) -> Result<(), ProjectPackageLockError> {
        match self {
            Self::Present {
                schema_version,
                lock,
                digest,
            } => {
                if *schema_version != PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1 {
                    return Err(ProjectPackageLockError::UnsupportedSchema);
                }
                lock.require_canonical_wire()?;
                if lock.digest()? != *digest {
                    return Err(ProjectPackageLockError::NonCanonicalWire);
                }
                Ok(())
            }
            Self::Unavailable { schema_version, .. } => (*schema_version
                == PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1)
                .then_some(())
                .ok_or(ProjectPackageLockError::UnsupportedSchema),
        }
    }

    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present { .. })
    }
}

fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PROJECT_PACKAGE_LOCK_TEXT
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

/// Match the existing package validator: three ASCII decimal segments, no
/// leading zeros (except zero itself), each representable by u32. This is
/// intentionally narrower than full SemVer because the package service
/// contract has no prerelease/build grammar.
pub fn valid_package_version(value: &str) -> bool {
    let mut segments = value.split('.');
    let valid = segments.by_ref().all(|segment| {
        !segment.is_empty()
            && (segment == "0" || !segment.starts_with('0'))
            && segment.bytes().all(|byte| byte.is_ascii_digit())
            && segment.parse::<u32>().is_ok()
    });
    valid && value.split('.').count() == 3
}

fn valid_authority(value: &ProjectPackageLockAuthority) -> bool {
    is_digest(&value.index_sha256)
        && is_digest(&value.policy_sha256)
        && is_digest(&value.build_set_id)
        && is_digest(&value.capability_digest)
        && is_digest(&value.provider_revision_digest)
}

fn valid_plugin_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PROJECT_PACKAGE_LOCK_TEXT
        && value.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_lowercase)
                && !segment.ends_with('_')
                && !segment.contains("__")
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        })
}

fn canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value)
}

fn canonical_revision(value: &str) -> bool {
    value
        .parse::<u64>()
        .ok()
        .filter(|revision| *revision > 0 && revision.to_string() == value)
        .is_some()
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
#[path = "tests/package_lock.rs"]
mod tests;
