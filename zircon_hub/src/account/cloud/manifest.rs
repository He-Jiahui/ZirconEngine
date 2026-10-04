use crate::account::AccountError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use zircon_runtime_interface::project::ProjectPackageLockState;

pub(crate) const MAX_BLOB_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_PROJECT_BYTES: u64 = 512 * 1024 * 1024;
pub(crate) const MAX_MANIFEST_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Manifest {
    pub(crate) schema_version: u32,
    pub(crate) engine: String,
    pub(crate) package_lock_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) package_lock: Option<ProjectPackageLockState>,
    pub(crate) ignore_policy: String,
    pub(crate) source_revision: Option<String>,
    pub(crate) files: Vec<FileEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FileEntry {
    pub(crate) path: String,
    pub(crate) digest: String,
    pub(crate) bytes: u64,
}

impl Manifest {
    // The service hashes this field order after sorting files by path.
    pub(crate) fn normalized(mut self) -> Result<Self, AccountError> {
        self.files.sort_by(|left, right| left.path.cmp(&right.path));
        self.canonical_digest()?;
        Ok(self)
    }

    pub(crate) fn canonical_digest(&self) -> Result<String, AccountError> {
        if self.schema_version != 1
            || self.engine.trim().is_empty()
            || self.engine.len() > 256
            || self.engine.chars().any(char::is_control)
            || self.ignore_policy != "zircon-project-v1"
            || self.files.len() > 10_000
            || !digest(&self.package_lock_digest)
            || self.source_revision.as_ref().is_some_and(|source| {
                source.is_empty() || source.len() > 256 || source.chars().any(char::is_control)
            })
            || self
                .files
                .windows(2)
                .any(|pair| pair[0].path.as_str() >= pair[1].path.as_str())
        {
            return Err(AccountError::ServiceFailure);
        }
        if let Some(state) = &self.package_lock {
            state.validate().map_err(|_| AccountError::ServiceFailure)?;
            if let ProjectPackageLockState::Present { digest, .. } = state {
                if digest != &self.package_lock_digest {
                    return Err(AccountError::ServiceFailure);
                }
            }
        }
        let mut paths = BTreeSet::new();
        let mut total = 0_u64;
        for file in &self.files {
            if !digest(&file.digest)
                || file.bytes > MAX_BLOB_BYTES as u64
                || !valid_path(&file.path)
            {
                return Err(AccountError::ServiceFailure);
            }
            total = total
                .checked_add(file.bytes)
                .ok_or(AccountError::ServiceFailure)?;
            if total > MAX_PROJECT_BYTES || !paths.insert(file.path.to_lowercase()) {
                return Err(AccountError::ServiceFailure);
            }
        }
        for path in &paths {
            for (index, _) in path.match_indices('/') {
                if paths.contains(&path[..index]) {
                    return Err(AccountError::ServiceFailure);
                }
            }
        }
        let bytes = serde_json::to_vec(self).map_err(|_| AccountError::ServiceFailure)?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(AccountError::ServiceFailure);
        }
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    pub(crate) fn require_present_package_lock(&self) -> Result<(), AccountError> {
        match self.package_lock.as_ref() {
            Some(ProjectPackageLockState::Present { .. }) => Ok(()),
            Some(ProjectPackageLockState::Unavailable { .. }) | None => {
                Err(AccountError::ServiceFailure)
            }
        }
    }
}

pub(crate) fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn valid_path(path: &str) -> bool {
    if path.is_empty()
        || path.len() > 512
        || path.contains(['\\', ':', '<', '>', '"', '|', '?', '*'])
        || path.chars().any(char::is_control)
    {
        return false;
    }
    path.split('/').all(|component| {
        let lower = component.to_ascii_lowercase();
        let stem = lower.split('.').next().unwrap_or("");
        !component.is_empty()
            && !matches!(component, "." | "..")
            && !component.ends_with(['.', ' '])
            && !matches!(
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
            && !lower.starts_with(".env.")
            && ![".pem", ".key", ".pfx", ".p12", ".keystore"]
                .iter()
                .any(|suffix| lower.ends_with(suffix))
            && !matches!(
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
    })
}
