use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::TrayError;

pub const REPOSITORY_IDENTITY_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryIdentity {
    pub version: u32,
    pub canonical_path: PathBuf,
    pub key: String,
}

impl RepositoryIdentity {
    pub fn for_path(path: impl AsRef<Path>) -> Result<Self, TrayError> {
        let canonical_path = path.as_ref().canonicalize()?;
        let normalized = canonical_path.to_string_lossy().to_lowercase();
        let key = hex::encode(Sha256::digest(normalized.as_bytes()));
        Ok(Self {
            version: REPOSITORY_IDENTITY_VERSION,
            canonical_path,
            key,
        })
    }

    pub fn short_key(&self) -> String {
        self.key[..10].to_ascii_uppercase()
    }

    pub fn mutex_name(&self) -> String {
        format!("Local\\ZirconSessionTray-{}", self.short_key())
    }
}

#[cfg(windows)]
pub struct RepositoryMutex(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl RepositoryMutex {
    pub fn acquire(identity: &RepositoryIdentity) -> Result<Self, TrayError> {
        use windows::core::HSTRING;
        use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
        use windows::Win32::System::Threading::CreateMutexW;

        let name = HSTRING::from(identity.mutex_name());
        let handle = unsafe { CreateMutexW(None, false, &name)? };
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { windows::Win32::Foundation::CloseHandle(handle)? };
            return Err(TrayError::IdentityMismatch(
                "another tray already owns this repository identity",
            ));
        }
        Ok(Self(handle))
    }
}

#[cfg(windows)]
impl Drop for RepositoryMutex {
    fn drop(&mut self) {
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(self.0) };
    }
}

#[cfg(not(windows))]
pub struct RepositoryMutex;

#[cfg(not(windows))]
impl RepositoryMutex {
    pub fn acquire(_identity: &RepositoryIdentity) -> Result<Self, TrayError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_key_is_stable_and_matches_task_suffix_shape() {
        let identity = RepositoryIdentity::for_path(".").expect("repo identity");
        let second = RepositoryIdentity::for_path(&identity.canonical_path).expect("repo identity");
        assert_eq!(identity, second);
        assert_eq!(64, identity.key.len());
        assert_eq!(10, identity.short_key().len());
        assert!(identity.mutex_name().ends_with(&identity.short_key()));
    }
}
