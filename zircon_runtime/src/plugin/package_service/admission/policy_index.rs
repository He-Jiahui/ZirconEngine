use super::super::{
    valid_plugin_id, InstallReceipt, PackageError, PackageHostPolicy, Result,
    INSTALL_RECEIPT_SCHEMA_V2,
};
use crate::{
    core::framework::{platform::RuntimeTargetMode, project::ExportTargetPlatform},
    plugin::native::{NativePackageReceiptTrust, NativePluginArtifactTarget},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

const MAX_POLICY_INDEX_BYTES: usize = 65_536;
const MAX_POLICY_ENTRIES: usize = 24;
const POLICY_INDEX_KIND: &str = "zircon_native_plugin_policy_index";
const POLICY_INDEX_ENV: &str = "ZIRCON_NATIVE_PLUGIN_POLICY_INDEX";
const INSTALLED_SELECTION_INDEX_KIND: &str = "zircon_native_plugin_installed_selections";
const INSTALLED_SELECTION_INDEX_SCHEMA_VERSION: u32 = 1;
const INSTALLED_SELECTION_INDEX_FILE: &str = "zircon_native_plugin_installed_selections.json";
const MAX_INSTALLED_SELECTIONS: usize = 512;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativePluginPolicyIndexDocument {
    schema_version: u32,
    kind: String,
    entries: Vec<NativePluginPolicyIndexEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativePluginPolicyIndexEntry {
    target: NativePluginArtifactTarget,
    policy_path: PathBuf,
    policy_sha256: String,
}

#[derive(Debug)]
struct NativePluginPolicyIndex {
    entries: Vec<NativePluginPolicyIndexEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativePluginInstalledSelectionIndexDocument {
    schema_version: u32,
    kind: String,
    entries: Vec<NativePluginInstalledSelection>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePluginInstalledSelection {
    pub target: NativePluginArtifactTarget,
    pub plugin_id: String,
    pub identity_digest: String,
    pub package_id: String,
    pub release_revision: String,
    pub artifact_digest: String,
}

#[derive(Debug)]
struct NativePluginInstalledSelectionIndex {
    entries: Vec<NativePluginInstalledSelection>,
}

impl NativePluginInstalledSelectionIndex {
    fn entry_for(
        &self,
        target: &NativePluginArtifactTarget,
        plugin_id: &str,
    ) -> Option<&NativePluginInstalledSelection> {
        self.entries
            .iter()
            .find(|entry| &entry.target == target && entry.plugin_id == plugin_id)
    }
}

impl NativePluginPolicyIndex {
    fn entry_for(
        &self,
        target: &NativePluginArtifactTarget,
    ) -> Option<&NativePluginPolicyIndexEntry> {
        self.entries.iter().find(|entry| &entry.target == target)
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum NativePluginPolicyError {
    #[error(
        "native plugin host policy is not configured; set ZIRCON_NATIVE_PLUGIN_POLICY_INDEX to a private host policy index"
    )]
    Unconfigured,
    #[error("native plugin host policy does not define this target context")]
    TargetUnconfigured,
    #[error("native plugin host policy failed its private trust checks")]
    Rejected,
    #[error("native plugin target store is unavailable or is not private")]
    StoreUnavailable,
}

/// A policy loaded from a private, operator-selected host policy index.
#[derive(Clone)]
pub struct LoadedNativePluginPolicy {
    policy: PackageHostPolicy,
    index_path: PathBuf,
    policy_path: PathBuf,
    index_sha256: String,
    policy_sha256: String,
}

impl std::fmt::Debug for LoadedNativePluginPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LoadedNativePluginPolicy")
            .field("index_path", &self.index_path)
            .field("policy_path", &self.policy_path)
            .field("index_sha256", &self.index_sha256)
            .field("policy_sha256", &self.policy_sha256)
            .finish_non_exhaustive()
    }
}

impl LoadedNativePluginPolicy {
    pub fn policy(&self) -> &PackageHostPolicy {
        &self.policy
    }

    pub fn index_path(&self) -> &Path {
        &self.index_path
    }

    pub fn policy_path(&self) -> &Path {
        &self.policy_path
    }

    pub fn index_sha256(&self) -> &str {
        &self.index_sha256
    }

    pub fn policy_sha256(&self) -> &str {
        &self.policy_sha256
    }

    /// Returns the host-owned exact install selection for this policy target.
    ///
    /// An absent file means no package has been selected yet. A malformed or untrusted file is
    /// rejected so the caller cannot fall back to whichever account happens to contain a match.
    pub fn installed_selections(
        &self,
    ) -> std::result::Result<Vec<NativePluginInstalledSelection>, NativePluginPolicyError> {
        let path = self.installed_selection_path();
        if !path
            .try_exists()
            .map_err(|_| NativePluginPolicyError::Rejected)?
        {
            return Ok(Vec::new());
        }
        let bytes = read_private(&path)?;
        let index = parse_installed_selection_index(&bytes)
            .map_err(|_| NativePluginPolicyError::Rejected)?;
        Ok(index
            .entries
            .into_iter()
            .filter(|entry| entry.target == self.policy.target)
            .collect())
    }

    /// Location of the host-owned selection sidecar derived from the selected policy index.
    pub fn installed_selection_path(&self) -> PathBuf {
        self.index_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(INSTALLED_SELECTION_INDEX_FILE)
    }

    /// Persists one explicit host install selection without allowing the project or package to
    /// choose the storage root. The selection file is derived from the already authenticated
    /// policy index and published through the same private-directory checks as host policy data.
    ///
    /// Receipt publication and selection publication are intentionally separate durable steps:
    /// callers must retry this operation from the receipt before reporting a committed install if
    /// the first write is interrupted.
    pub fn persist_installed_selection(
        &self,
        selection: NativePluginInstalledSelection,
    ) -> std::result::Result<(), NativePluginPolicyError> {
        if selection.target != self.policy.target || !valid_installed_selection(&selection) {
            return Err(NativePluginPolicyError::Rejected);
        }
        let path = self.installed_selection_path();
        #[cfg(windows)]
        {
            let lock_path = path.with_file_name(format!("{INSTALLED_SELECTION_INDEX_FILE}.lock"));
            let _lock = super::super::windows::lock_private_file(&lock_path)
                .map_err(|_| NativePluginPolicyError::Rejected)?;
            let mut index = if path
                .try_exists()
                .map_err(|_| NativePluginPolicyError::Rejected)?
            {
                parse_installed_selection_index(&read_private(&path)?)
                    .map_err(|_| NativePluginPolicyError::Rejected)?
            } else {
                NativePluginInstalledSelectionIndex {
                    entries: Vec::new(),
                }
            };
            upsert_installed_selection(&mut index.entries, selection);
            if index.entries.len() > MAX_INSTALLED_SELECTIONS
                || index
                    .entries
                    .iter()
                    .any(|entry| !valid_installed_selection(entry))
            {
                return Err(NativePluginPolicyError::Rejected);
            }
            let document = NativePluginInstalledSelectionIndexDocument {
                schema_version: INSTALLED_SELECTION_INDEX_SCHEMA_VERSION,
                kind: INSTALLED_SELECTION_INDEX_KIND.to_owned(),
                entries: index.entries,
            };
            let bytes =
                serde_json::to_vec(&document).map_err(|_| NativePluginPolicyError::Rejected)?;
            if bytes.len() > MAX_POLICY_INDEX_BYTES {
                return Err(NativePluginPolicyError::Rejected);
            }
            super::super::windows::write_private_regular_atomic(
                &path,
                &bytes,
                MAX_POLICY_INDEX_BYTES,
            )
            .map_err(|_| NativePluginPolicyError::Rejected)
        }
        #[cfg(not(windows))]
        {
            let _ = (path, selection);
            Err(NativePluginPolicyError::Rejected)
        }
    }

    /// Converts a verified v2 package-store receipt into the exact host selection to persist.
    /// Legacy receipts have no independently recoverable plugin or target identity and therefore
    /// cannot create or repair a selection entry.
    pub fn persist_installed_receipt_selection(
        &self,
        identity_digest: &str,
        receipt: &InstallReceipt,
    ) -> std::result::Result<(), NativePluginPolicyError> {
        let plugin_id = receipt
            .plugin_id
            .as_deref()
            .filter(|plugin_id| valid_plugin_id(plugin_id))
            .ok_or(NativePluginPolicyError::Rejected)?;
        let target = receipt
            .target
            .as_ref()
            .filter(|target| **target == self.policy.target)
            .cloned()
            .ok_or(NativePluginPolicyError::Rejected)?;
        if receipt.schema_version != INSTALL_RECEIPT_SCHEMA_V2 || !is_policy_digest(identity_digest)
        {
            return Err(NativePluginPolicyError::Rejected);
        }
        self.persist_installed_selection(NativePluginInstalledSelection {
            target,
            plugin_id: plugin_id.to_owned(),
            identity_digest: identity_digest.to_owned(),
            package_id: receipt.package.package_id.clone(),
            release_revision: receipt.package.release_revision.clone(),
            artifact_digest: receipt.package.artifact_digest.clone(),
        })
    }

    /// Keeps host configuration and installed code outside the active project tree.
    pub fn is_disjoint_from_project(&self, project_root: &Path) -> bool {
        let Ok(project_root) = std::fs::canonicalize(project_root) else {
            return false;
        };
        !paths_overlap(&self.index_path, &project_root)
            && !paths_overlap(&self.policy_path, &project_root)
            && !paths_overlap(&self.policy.root, &project_root)
    }
}

/// Resolves the current process' host-selected private native-plugin policy index.
pub fn load_host_policy_for_target(
    target: &NativePluginArtifactTarget,
) -> std::result::Result<LoadedNativePluginPolicy, NativePluginPolicyError> {
    let path = std::env::var_os(POLICY_INDEX_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or(NativePluginPolicyError::Unconfigured)?;
    load_host_policy_index_for_target(&path, target)
}

/// Resolves a target policy from a host-selected index path after checking its Windows ACLs.
///
/// Callers must obtain `index_path` from host-owned configuration, never from a project,
/// package, or install request. The file and every target policy are reopened through the
/// existing handle-pinned private-file checks.
pub fn load_host_policy_index_for_target(
    index_path: &Path,
    target: &NativePluginArtifactTarget,
) -> std::result::Result<LoadedNativePluginPolicy, NativePluginPolicyError> {
    let index_bytes = read_private(index_path)?;
    let index_sha256 = digest(&index_bytes);
    let index = parse_policy_index(&index_bytes).map_err(|_| NativePluginPolicyError::Rejected)?;
    let selected = index
        .entry_for(target)
        .ok_or(NativePluginPolicyError::TargetUnconfigured)?;
    let index_parent = canonical_existing_directory(
        index_path
            .parent()
            .ok_or(NativePluginPolicyError::Rejected)?,
    )?;

    let mut roots = Vec::<PathBuf>::with_capacity(index.entries.len());
    let mut selected_policy = None;
    let mut selected_policy_path = None;
    for entry in &index.entries {
        let bytes = read_private(&entry.policy_path)?;
        verify_policy_digest(&entry.policy_sha256, &bytes)
            .map_err(|_| NativePluginPolicyError::Rejected)?;
        let mut policy: PackageHostPolicy =
            serde_json::from_slice(&bytes).map_err(|_| NativePluginPolicyError::Rejected)?;
        if !validate_policy_entry(entry, &policy) || !matches_running_binary_target(&policy) {
            return Err(NativePluginPolicyError::Rejected);
        }
        let root = canonical_private_directory(&policy.root)?;
        let policy_parent = canonical_existing_directory(
            entry
                .policy_path
                .parent()
                .ok_or(NativePluginPolicyError::Rejected)?,
        )?;
        if paths_overlap(&index_parent, &root)
            || paths_overlap(&policy_parent, &root)
            || roots.iter().any(|existing| paths_overlap(existing, &root))
        {
            return Err(NativePluginPolicyError::Rejected);
        }
        roots.push(root);
        if std::ptr::eq(entry, selected) {
            policy.root = roots
                .last()
                .cloned()
                .ok_or(NativePluginPolicyError::Rejected)?;
            selected_policy = Some(policy);
            selected_policy_path = Some(
                std::fs::canonicalize(&entry.policy_path)
                    .map_err(|_| NativePluginPolicyError::Rejected)?,
            );
        }
    }

    let policy = selected_policy.ok_or(NativePluginPolicyError::TargetUnconfigured)?;
    Ok(LoadedNativePluginPolicy {
        policy,
        index_path: std::fs::canonicalize(index_path)
            .map_err(|_| NativePluginPolicyError::Rejected)?,
        policy_path: selected_policy_path.ok_or(NativePluginPolicyError::Rejected)?,
        index_sha256,
        policy_sha256: selected.policy_sha256.clone(),
    })
}

fn read_private(path: &Path) -> std::result::Result<Vec<u8>, NativePluginPolicyError> {
    #[cfg(windows)]
    {
        super::super::windows::read_private_regular(path, MAX_POLICY_INDEX_BYTES)
            .map_err(|_| NativePluginPolicyError::Rejected)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(NativePluginPolicyError::Rejected)
    }
}

fn canonical_existing_directory(
    path: &Path,
) -> std::result::Result<PathBuf, NativePluginPolicyError> {
    #[cfg(windows)]
    {
        let pins = super::super::windows::DirectoryPins::open(path, false)
            .map_err(|_| NativePluginPolicyError::Rejected)?;
        pins.require_private_owner()
            .map_err(|_| NativePluginPolicyError::Rejected)?;
        Ok(pins.path().to_owned())
    }
    #[cfg(not(windows))]
    {
        std::fs::canonicalize(path).map_err(|_| NativePluginPolicyError::Rejected)
    }
}

fn canonical_private_directory(
    path: &Path,
) -> std::result::Result<PathBuf, NativePluginPolicyError> {
    #[cfg(windows)]
    {
        let pins = super::super::windows::DirectoryPins::open(path, false)
            .map_err(|_| NativePluginPolicyError::StoreUnavailable)?;
        pins.require_private_owner()
            .map_err(|_| NativePluginPolicyError::StoreUnavailable)?;
        Ok(pins.path().to_owned())
    }
    #[cfg(not(windows))]
    {
        std::fs::canonicalize(path).map_err(|_| NativePluginPolicyError::StoreUnavailable)
    }
}

fn valid_host_policy(policy: &PackageHostPolicy) -> bool {
    use chrono::Utc;

    if policy.target_triple.trim().is_empty()
        || policy.target_triple.chars().any(char::is_control)
        || policy.sdk_api_version.trim().is_empty()
        || policy.sdk_api_version.chars().any(char::is_control)
        || !is_policy_digest(&policy.build_set_id)
        || policy.max_receipt_age_seconds == 0
        || policy
            .allowed_capabilities
            .iter()
            .any(|value| value.trim().is_empty() || value.chars().any(char::is_control))
        || policy.trust_valid_until <= Utc::now()
        || !policy.target.platform.supports_native_dynamic()
        || !valid_native_target_triple(&policy.target, &policy.target_triple)
    {
        return false;
    }
    let Ok(registry) = serde_json::to_vec(&policy.trust_registry) else {
        return false;
    };
    NativePackageReceiptTrust::from_registry_json(
        &registry,
        policy.key_policies.clone(),
        policy.trust_valid_until,
        MAX_POLICY_INDEX_BYTES,
    )
    .is_ok()
}

fn valid_native_target_triple(target: &NativePluginArtifactTarget, triple: &str) -> bool {
    if !target.platform.supports_native_dynamic() {
        return false;
    }
    let parts = triple.split('-').collect::<Vec<_>>();
    parts.len() >= 3
        && matches!(parts[0], "x86_64" | "aarch64" | "i686")
        && parts
            .iter()
            .any(|part| part.eq_ignore_ascii_case("windows"))
        && parts
            .last()
            .is_some_and(|part| matches!(*part, "msvc" | "gnu" | "gnullvm"))
}

fn matches_running_binary_target(policy: &PackageHostPolicy) -> bool {
    #[cfg(windows)]
    {
        policy.target.platform == ExportTargetPlatform::Windows
            && policy.target_triple.split('-').next() == Some(std::env::consts::ARCH)
    }
    #[cfg(not(windows))]
    {
        let _ = policy;
        false
    }
}

fn validate_policy_entry(entry: &NativePluginPolicyIndexEntry, policy: &PackageHostPolicy) -> bool {
    policy.target == entry.target && valid_host_policy(policy)
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    let left = normalized_path_key(left);
    let right = normalized_path_key(right);
    path_key_contains(&left, &right) || path_key_contains(&right, &left)
}

fn path_key_contains(parent: &str, child: &str) -> bool {
    child == parent
        || child
            .strip_prefix(parent)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn parse_policy_index(bytes: &[u8]) -> Result<NativePluginPolicyIndex> {
    if bytes.len() > MAX_POLICY_INDEX_BYTES {
        return Err(PackageError::Capacity);
    }
    let document: NativePluginPolicyIndexDocument =
        serde_json::from_slice(bytes).map_err(|_| PackageError::Trust)?;
    if document.schema_version != 1
        || document.kind != POLICY_INDEX_KIND
        || document.entries.is_empty()
        || document.entries.len() > MAX_POLICY_ENTRIES
    {
        return Err(PackageError::Trust);
    }

    let mut targets = Vec::with_capacity(document.entries.len());
    let mut policy_paths = HashSet::with_capacity(document.entries.len());
    for entry in &document.entries {
        if !entry.policy_path.is_absolute()
            || entry.policy_path.components().any(|part| {
                matches!(
                    part,
                    std::path::Component::CurDir | std::path::Component::ParentDir
                )
            })
            || !is_policy_digest(&entry.policy_sha256)
            || targets.iter().any(|target| target == &entry.target)
        {
            return Err(PackageError::Trust);
        }
        let path_key = normalized_path_key(&entry.policy_path);
        if !policy_paths.insert(path_key) {
            return Err(PackageError::Trust);
        }
        targets.push(entry.target.clone());
    }
    Ok(NativePluginPolicyIndex {
        entries: document.entries,
    })
}

fn parse_installed_selection_index(bytes: &[u8]) -> Result<NativePluginInstalledSelectionIndex> {
    if bytes.len() > MAX_POLICY_INDEX_BYTES {
        return Err(PackageError::Capacity);
    }
    let document: NativePluginInstalledSelectionIndexDocument =
        serde_json::from_slice(bytes).map_err(|_| PackageError::Trust)?;
    if document.schema_version != INSTALLED_SELECTION_INDEX_SCHEMA_VERSION
        || document.kind != INSTALLED_SELECTION_INDEX_KIND
        || document.entries.len() > MAX_INSTALLED_SELECTIONS
    {
        return Err(PackageError::Trust);
    }

    for (index, entry) in document.entries.iter().enumerate() {
        if !valid_installed_selection(entry)
            || document.entries[..index].iter().any(|existing| {
                existing.target == entry.target && existing.plugin_id == entry.plugin_id
            })
        {
            return Err(PackageError::Trust);
        }
    }
    Ok(NativePluginInstalledSelectionIndex {
        entries: document.entries,
    })
}

fn upsert_installed_selection(
    entries: &mut Vec<NativePluginInstalledSelection>,
    selection: NativePluginInstalledSelection,
) {
    if let Some(existing) = entries.iter_mut().find(|existing| {
        existing.target == selection.target && existing.plugin_id == selection.plugin_id
    }) {
        *existing = selection;
    } else {
        entries.push(selection);
    }
}

fn valid_installed_selection(entry: &NativePluginInstalledSelection) -> bool {
    let canonical_revision = entry
        .release_revision
        .parse::<u64>()
        .is_ok_and(|revision| revision > 0 && revision.to_string() == entry.release_revision);
    matches!(
        entry.target.runtime_mode,
        RuntimeTargetMode::EditorHost | RuntimeTargetMode::ClientRuntime
    ) && matches!(
        entry.target.platform,
        ExportTargetPlatform::Windows | ExportTargetPlatform::Linux | ExportTargetPlatform::Macos
    ) && valid_plugin_id(&entry.plugin_id)
        && is_policy_digest(&entry.identity_digest)
        && uuid::Uuid::parse_str(&entry.package_id)
            .is_ok_and(|package_id| package_id.to_string() == entry.package_id)
        && canonical_revision
        && is_policy_digest(&entry.artifact_digest)
}

fn normalized_path_key(path: &std::path::Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(bytes))
}

fn is_policy_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn verify_policy_digest(expected: &str, bytes: &[u8]) -> Result<()> {
    use sha2::{Digest, Sha256};

    if !is_policy_digest(expected) || format!("{:x}", Sha256::digest(bytes)) != expected {
        return Err(PackageError::Trust);
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/policy_index.rs"]
mod tests;
