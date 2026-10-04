use crate::service::error::ServiceError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use zircon_runtime_interface::project::ProjectPackageLockState;

pub const MAX_BLOB_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PROJECT_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: usize = 8 * 1024 * 1024;
pub const IGNORE_POLICY: &str = "zircon-project-v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub engine: String,
    pub package_lock_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_lock: Option<ProjectPackageLockState>,
    pub ignore_policy: String,
    pub source_revision: Option<String>,
    pub files: Vec<FileEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileEntry {
    pub path: String,
    pub digest: String,
    pub bytes: u64,
}

pub fn digest(value: &str) -> Result<(), ServiceError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ServiceError::InvalidRequest);
    }
    Ok(())
}

impl Manifest {
    pub fn validate(&self) -> Result<(), ServiceError> {
        if self.schema_version != 1
            || self.engine.trim().is_empty()
            || self.engine.len() > 256
            || self.engine.chars().any(char::is_control)
            || self.files.len() > 10000
            || self.ignore_policy != IGNORE_POLICY
            || self.source_revision.as_ref().is_some_and(|value| {
                value.is_empty() || value.len() > 256 || value.chars().any(char::is_control)
            })
        {
            return Err(ServiceError::InvalidRequest);
        }
        digest(&self.package_lock_digest)?;
        if let Some(state) = &self.package_lock {
            state.validate().map_err(|_| ServiceError::InvalidRequest)?;
            if let ProjectPackageLockState::Present { digest, .. } = state {
                if digest != &self.package_lock_digest {
                    return Err(ServiceError::InvalidRequest);
                }
            }
        }
        let mut paths = BTreeSet::new();
        let mut total = 0u64;
        for entry in &self.files {
            digest(&entry.digest)?;
            if entry.path.is_empty()
                || entry.path.len() > 512
                || entry
                    .path
                    .contains(['\\', ':', '<', '>', '"', '|', '?', '*'])
                || entry.bytes > MAX_BLOB_BYTES as u64
                || entry.path.chars().any(char::is_control)
            {
                return Err(ServiceError::InvalidRequest);
            }
            for component in entry.path.split('/') {
                let lower = component.to_ascii_lowercase();
                let stem = lower.split('.').next().unwrap_or("");
                if component.is_empty()
                    || matches!(component, "." | "..")
                    || component.ends_with(['.', ' '])
                    || matches!(
                        lower.as_str(),
                        ".git"
                            | ".hg"
                            | ".svn"
                            | ".ssh"
                            | ".aws"
                            | ".azure"
                            | ".gnupg"
                            | ".codex"
                            | ".env"
                            | "target"
                            | "build"
                            | "binaries"
                            | "generated"
                            | "node_modules"
                            | "cache"
                            | "deriveddatacache"
                            | "intermediate"
                            | "saved"
                            | "crashdumps"
                            | "credentials"
                            | "secrets"
                    )
                    || lower.starts_with(".env.")
                    || [".pem", ".key", ".pfx", ".p12", ".keystore"]
                        .iter()
                        .any(|suffix| lower.ends_with(suffix))
                    || matches!(
                        stem,
                        "con"
                            | "prn"
                            | "aux"
                            | "nul"
                            | "com1"
                            | "com2"
                            | "com3"
                            | "com4"
                            | "com5"
                            | "com6"
                            | "com7"
                            | "com8"
                            | "com9"
                            | "lpt1"
                            | "lpt2"
                            | "lpt3"
                            | "lpt4"
                            | "lpt5"
                            | "lpt6"
                            | "lpt7"
                            | "lpt8"
                            | "lpt9"
                    )
                {
                    return Err(ServiceError::InvalidRequest);
                }
            }
            if !paths.insert(entry.path.to_lowercase()) {
                return Err(ServiceError::InvalidRequest);
            }
            total = total
                .checked_add(entry.bytes)
                .ok_or(ServiceError::Capacity)?;
            if total > MAX_PROJECT_BYTES {
                return Err(ServiceError::Capacity);
            }
        }
        for path in &paths {
            for (index, _) in path.match_indices('/') {
                if paths.contains(&path[..index]) {
                    return Err(ServiceError::InvalidRequest);
                }
            }
        }
        Ok(())
    }

    pub(super) fn canonicalize(&mut self) -> Result<String, ServiceError> {
        // Canonicalization authenticates the manifest's shape and lock digest
        // linkage only. Authority fields remain producer provenance; this
        // server path must not bless a client assertion as authorization.
        self.validate()?;
        if !matches!(
            self.package_lock.as_ref(),
            Some(ProjectPackageLockState::Present { .. })
        ) {
            return Err(ServiceError::InvalidRequest);
        }
        self.files.sort_by(|left, right| left.path.cmp(&right.path));
        let bytes = serde_json::to_vec(self).map_err(|_| ServiceError::InvalidRequest)?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(ServiceError::Capacity);
        }
        use sha2::{Digest, Sha256};
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }
}
