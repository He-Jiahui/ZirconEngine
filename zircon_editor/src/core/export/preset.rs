use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use zircon_runtime_interface::export::{
    load_export_preset, ExportPreset, ExportPresetLoadError, ExportPresetValidationError,
};
use zircon_runtime_interface::serialization::{write_versioned_text, WriteError};

const EXPORT_PRESET_DIRECTORY: &str = "export";
const EXPORT_PRESET_EXTENSION: &str = "zpreset";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportPresetStore {
    project_root: PathBuf,
}

impl ExportPresetStore {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn preset_path(&self, name: &str) -> Result<PathBuf, ExportPresetStoreError> {
        validate_preset_name(name)?;
        Ok(self
            .project_root
            .join(EXPORT_PRESET_DIRECTORY)
            .join(format!("{name}.{EXPORT_PRESET_EXTENSION}")))
    }

    pub fn load(&self, name: &str) -> Result<ExportPreset, ExportPresetStoreError> {
        let path = self.preset_path(name)?;
        let bytes = fs::read(&path).map_err(|source| ExportPresetStoreError::Read {
            path: path.clone(),
            source,
        })?;
        let preset =
            load_export_preset(&bytes).map_err(|source| ExportPresetStoreError::Decode {
                path: path.clone(),
                source,
            })?;
        Ok(preset)
    }

    pub fn save(
        &self,
        name: &str,
        preset: &ExportPreset,
    ) -> Result<PathBuf, ExportPresetStoreError> {
        let path = self.preset_path(name)?;
        preset
            .validate()
            .map_err(|source| ExportPresetStoreError::Validation {
                path: path.clone(),
                source,
            })?;
        let encoded = write_versioned_text(preset).map_err(ExportPresetStoreError::Encode)?;
        let directory = path
            .parent()
            .expect("validated export preset path always has a parent");
        fs::create_dir_all(directory).map_err(|source| {
            ExportPresetStoreError::CreateDirectory {
                path: directory.to_path_buf(),
                source,
            }
        })?;

        let transaction = PresetWriteTransaction::new(path.clone());
        transaction.write_and_commit(encoded.as_bytes())?;
        Ok(path)
    }
}

#[derive(Debug)]
pub enum ExportPresetStoreError {
    InvalidName {
        name: String,
    },
    Read {
        path: PathBuf,
        source: io::Error,
    },
    Decode {
        path: PathBuf,
        source: ExportPresetLoadError,
    },
    Validation {
        path: PathBuf,
        source: ExportPresetValidationError,
    },
    Encode(WriteError),
    CreateDirectory {
        path: PathBuf,
        source: io::Error,
    },
    CreateStaging {
        path: PathBuf,
        source: io::Error,
    },
    WriteStaging {
        path: PathBuf,
        source: io::Error,
    },
    SyncStaging {
        path: PathBuf,
        source: io::Error,
    },
    Commit {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for ExportPresetStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { name } => write!(formatter, "invalid export preset name `{name}`"),
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "failed to read export preset {}: {source}",
                    path.display()
                )
            }
            Self::Decode { path, source } => write!(
                formatter,
                "failed to decode export preset {}: {source}",
                path.display()
            ),
            Self::Validation { path, source } => write!(
                formatter,
                "export preset {} is invalid: {source}",
                path.display()
            ),
            Self::Encode(source) => write!(formatter, "failed to encode export preset: {source}"),
            Self::CreateDirectory { path, source } => write!(
                formatter,
                "failed to create export preset directory {}: {source}",
                path.display()
            ),
            Self::CreateStaging { path, source } => write!(
                formatter,
                "failed to create export preset staging file {}: {source}",
                path.display()
            ),
            Self::WriteStaging { path, source } => write!(
                formatter,
                "failed to write export preset staging file {}: {source}",
                path.display()
            ),
            Self::SyncStaging { path, source } => write!(
                formatter,
                "failed to sync export preset staging file {}: {source}",
                path.display()
            ),
            Self::Commit { path, source } => write!(
                formatter,
                "failed to commit export preset {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for ExportPresetStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidName { .. } => None,
            Self::Read { source, .. }
            | Self::CreateDirectory { source, .. }
            | Self::CreateStaging { source, .. }
            | Self::WriteStaging { source, .. }
            | Self::SyncStaging { source, .. }
            | Self::Commit { source, .. } => Some(source),
            Self::Decode { source, .. } => Some(source),
            Self::Validation { source, .. } => Some(source),
            Self::Encode(source) => Some(source),
        }
    }
}

struct PresetWriteTransaction {
    destination: PathBuf,
    staging: PathBuf,
}

impl PresetWriteTransaction {
    fn new(destination: PathBuf) -> Self {
        let nonce = format!("{}-{}", std::process::id(), thread_nonce());
        let file_name = destination
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("preset.zpreset")
            .to_owned();
        let directory = destination
            .parent()
            .expect("export preset destination always has a parent")
            .to_owned();
        Self {
            destination,
            staging: directory.join(format!(".{file_name}.{nonce}.staging")),
        }
    }

    fn write_and_commit(mut self, bytes: &[u8]) -> Result<(), ExportPresetStoreError> {
        let mut staging = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.staging)
            .map_err(|source| ExportPresetStoreError::CreateStaging {
                path: self.staging.clone(),
                source,
            })?;
        staging
            .write_all(bytes)
            .map_err(|source| ExportPresetStoreError::WriteStaging {
                path: self.staging.clone(),
                source,
            })?;
        staging
            .sync_all()
            .map_err(|source| ExportPresetStoreError::SyncStaging {
                path: self.staging.clone(),
                source,
            })?;
        drop(staging);

        atomic_replace(&self.staging, &self.destination).map_err(|source| {
            ExportPresetStoreError::Commit {
                path: self.destination.clone(),
                source,
            }
        })?;

        sync_parent_directory(self.destination.parent());
        Ok(())
    }
}

impl Drop for PresetWriteTransaction {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.staging);
    }
}

#[cfg(windows)]
fn atomic_replace(staging: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;

    if !destination.exists() {
        return fs::rename(staging, destination);
    }
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let staging = staging
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // SAFETY: both buffers are NUL-terminated and remain alive for the call;
    // null backup/exclude/reserved pointers are explicitly allowed by Win32.
    let replaced = unsafe {
        ReplaceFileW(
            destination.as_ptr(),
            staging.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn atomic_replace(staging: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(staging, destination)
}

fn validate_preset_name(name: &str) -> Result<(), ExportPresetStoreError> {
    let valid = !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err(ExportPresetStoreError::InvalidName {
            name: name.to_string(),
        })
    }
}

fn thread_nonce() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}

#[cfg(unix)]
fn sync_parent_directory(parent: Option<&Path>) {
    use std::fs::File;

    if let Some(parent) = parent {
        let _ = File::open(parent).and_then(|directory| directory.sync_all());
    }
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: Option<&Path>) {}
