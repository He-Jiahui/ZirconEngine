use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer};

use crate::repository_identity::{
    identity_paths_equal, RepositoryIdentity, REPOSITORY_IDENTITY_VERSION,
};
use crate::TrayError;

pub const RUNTIME_DESCRIPTOR_VERSION: u32 = 2;
pub const SUPERVISION_API_VERSION: u32 = 1;

pub struct SecretString(String);

impl SecretString {
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretString([redacted])")
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDescriptor {
    pub descriptor_version: u32,
    pub host: String,
    pub port: u16,
    pub token: SecretString,
    pub pid: u32,
    pub process_creation_time: String,
    pub executable: PathBuf,
    pub command_line: Vec<String>,
    pub repo_root: PathBuf,
    pub repository_identity_version: u32,
    pub repository_key: String,
    pub instance_id: String,
    pub started_at: String,
    pub schema_version: u32,
    pub control_api_versions: Vec<u32>,
    pub supervision_api_versions: Vec<u32>,
}

impl RuntimeDescriptor {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, TrayError> {
        let bytes = fs::read(path)?;
        let descriptor: Self = serde_json::from_slice(&bytes)?;
        Ok(descriptor)
    }

    pub fn validate(&self, repository: &RepositoryIdentity) -> Result<(), TrayError> {
        if self.descriptor_version != RUNTIME_DESCRIPTOR_VERSION {
            return Err(TrayError::InvalidDescriptor(
                "unsupported descriptor version",
            ));
        }
        if self.host != "127.0.0.1" || self.port == 0 {
            return Err(TrayError::InvalidDescriptor(
                "endpoint is not exact IPv4 loopback",
            ));
        }
        if self.repository_identity_version != REPOSITORY_IDENTITY_VERSION
            || self.repository_key != repository.key
        {
            return Err(TrayError::IdentityMismatch("repository key differs"));
        }
        let descriptor_repo = self.repo_root.canonicalize()?;
        if !identity_paths_equal(&descriptor_repo, &repository.canonical_path) {
            return Err(TrayError::IdentityMismatch("repository path differs"));
        }
        if self.instance_id.is_empty()
            || self.process_creation_time.is_empty()
            || self.command_line.is_empty()
            || !self.control_api_versions.contains(&1)
            || !self
                .supervision_api_versions
                .contains(&SUPERVISION_API_VERSION)
        {
            return Err(TrayError::InvalidDescriptor(
                "required identity field is missing",
            ));
        }
        if !self
            .command_line
            .iter()
            .any(|part| part.to_lowercase().contains("session_coordinator"))
        {
            return Err(TrayError::IdentityMismatch(
                "command line is not the coordinator",
            ));
        }
        Ok(())
    }

    pub fn endpoint(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_debug_never_exposes_value() {
        let secret = SecretString("do-not-log".into());
        assert_eq!("SecretString([redacted])", format!("{secret:?}"));
        assert!(!format!("{secret:?}").contains("do-not-log"));
    }
}
