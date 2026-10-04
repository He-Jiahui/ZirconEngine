//! Authenticated package preparation and inventory publication, without activation.

mod admission;
mod store;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
mod verify;
#[cfg(windows)]
mod windows;

use super::native::{NativePackageKeyPolicy, NativePluginArtifactTarget};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf};

pub use admission::{
    capture_project_package_lock, load_host_policy_for_target, load_host_policy_index_for_target,
    resolve_project_native_plugin_admission, LoadedNativePluginPolicy, NativePluginAdmission,
    NativePluginInstalledSelection, NativePluginPolicyError, NativePluginPolicyStatus,
    NativePluginSelectionOutcome, NativePluginSelectionStatus, ProjectPackageLockProviderError,
};
pub use store::{read_regular, PackageStore};

pub const MAX_PACKAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_FILES: usize = 1024;
const MAX_CONTROL_BYTES: usize = 65536;
pub const INSTALL_REQUEST_SCHEMA_V2: u8 = 2;
pub const INSTALL_RECEIPT_SCHEMA_V2: u32 = 2;

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("package_input_invalid")]
    Invalid,
    #[error("package_trust_denied")]
    Trust,
    #[error("package_policy_unconfigured")]
    PolicyUnconfigured,
    #[error("package_target_unconfigured")]
    TargetUnconfigured,
    #[error("package_capacity_exceeded")]
    Capacity,
    #[error("package_policy_conflict")]
    Conflict,
    #[error("package_store_unavailable")]
    Storage,
    #[error("package_operation_busy")]
    Busy,
    #[error("package_outcome_unknown")]
    OutcomeUnknown,
}

type Result<T> = std::result::Result<T, PackageError>;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageHostPolicy {
    pub root: PathBuf,
    pub trust_registry: serde_json::Value,
    pub key_policies: Vec<NativePackageKeyPolicy>,
    pub trust_valid_until: DateTime<Utc>,
    pub target: NativePluginArtifactTarget,
    pub target_triple: String,
    pub sdk_api_version: String,
    pub build_set_id: String,
    pub allowed_capabilities: Vec<String>,
    pub max_receipt_age_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallRequest {
    pub schema_version: u8,
    pub operation_id: String,
    pub identity_digest: String,
    pub package_id: String,
    pub version: String,
    pub release_revision: String,
    pub artifact_digest: String,
    pub artifact_size: u64,
    pub expected_inventory_revision: String,
    pub target: NativePluginArtifactTarget,
}

impl InstallRequest {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != INSTALL_REQUEST_SCHEMA_V2
            || self.target.runtime_mode
                == crate::core::framework::platform::RuntimeTargetMode::ServerRuntime
            || !canonical_uuid(&self.operation_id)
            || !canonical_uuid(&self.package_id)
            || !is_digest(&self.identity_digest)
            || !is_digest(&self.artifact_digest)
            || self.version.is_empty()
            || self.version.len() > 64
            || self.version.chars().any(char::is_control)
            || revision(&self.release_revision)? == 0
        {
            return Err(PackageError::Invalid);
        }
        revision(&self.expected_inventory_revision)?;
        if self.artifact_size == 0 || self.artifact_size > MAX_PACKAGE_BYTES as u64 {
            return Err(PackageError::Capacity);
        }
        Ok(())
    }

    fn fingerprint(&self) -> Result<String> {
        Ok(digest(
            &serde_json::to_vec(self).map_err(|_| PackageError::Invalid)?,
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstalledPackage {
    pub operation_id: String,
    pub package_id: String,
    pub version: String,
    pub release_revision: String,
    pub artifact_digest: String,
    pub slot: String,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageInventory {
    pub schema_version: u32,
    pub revision: String,
    pub packages: Vec<InstalledPackage>,
}

impl Default for PackageInventory {
    fn default() -> Self {
        Self {
            schema_version: 1,
            revision: "0".into(),
            packages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallReceipt {
    pub schema_version: u32,
    pub operation_id: String,
    pub request_digest: String,
    pub inventory_revision: String,
    pub package: InstalledPackage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<NativePluginArtifactTarget>,
}

impl InstallReceipt {
    pub fn matches_request(&self, request: &InstallRequest) -> Result<bool> {
        Ok(self.operation_id == request.operation_id
            && self.request_digest == request.fingerprint()?)
    }
}

pub struct PreparedPackage {
    request: InstallRequest,
    plugin_id: String,
    files: BTreeMap<String, Vec<u8>>,
    valid_until: DateTime<Utc>,
}

impl PreparedPackage {
    pub fn verify(
        request: InstallRequest,
        bytes: Vec<u8>,
        policy: &PackageHostPolicy,
    ) -> Result<Self> {
        verify::prepare(request, bytes, policy)
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value)
}

fn revision(value: &str) -> Result<u64> {
    value
        .parse::<u64>()
        .ok()
        .filter(|number| number.to_string() == value)
        .ok_or(PackageError::Invalid)
}

fn valid_plugin_id(value: &str) -> bool {
    value.len() <= 128
        && value.trim() == value
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

fn valid_member(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 240
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && !part.ends_with('.')
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                && !matches!(
                    part.split('.')
                        .next()
                        .unwrap()
                        .to_ascii_uppercase()
                        .as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
}

fn distinct_members<'a>(members: impl IntoIterator<Item = &'a str>) -> bool {
    let mut paths = std::collections::BTreeSet::new();
    for path in members {
        if !valid_member(path) || !paths.insert(path.to_ascii_lowercase()) {
            return false;
        }
    }
    paths.iter().all(|path| {
        path.match_indices('/')
            .all(|(index, _)| !paths.contains(&path[..index]))
    })
}
