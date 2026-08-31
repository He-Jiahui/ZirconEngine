use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::serialization::{
    load_versioned, write_versioned_text, Format, LoadError, MigrateError, MigrationChain,
    MigrationStep, SchemaId, VersionedSchema, WriteError,
};

use super::{
    SettingValue, SettingsAuthority, SettingsError, SettingsKey, SettingsRegistry, SettingsScope,
};

/// User settings root override. This is a directory, never a settings file path.
pub const SETTINGS_USER_ROOT_ENV: &str = "ZIRCON_EDITOR_APPEARANCE_PREFERENCES";
const SETTINGS_FILE_NAME: &str = "settings.toml";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsPaths {
    user: PathBuf,
    project: Option<PathBuf>,
}

impl SettingsPaths {
    pub fn from_roots(user_root: impl Into<PathBuf>, project_root: Option<&Path>) -> Self {
        Self {
            user: user_root.into().join(SETTINGS_FILE_NAME),
            project: project_root.map(project_settings_path),
        }
    }

    pub fn user(&self) -> &Path {
        &self.user
    }

    pub fn project(&self) -> Option<&Path> {
        self.project.as_deref()
    }

    pub fn user_root_from_environment() -> Result<PathBuf, SettingsStoreError> {
        Self::user_root_from_env_value(std::env::var_os(SETTINGS_USER_ROOT_ENV))
    }

    pub(crate) fn user_root_from_env_value(
        value: Option<OsString>,
    ) -> Result<PathBuf, SettingsStoreError> {
        if let Some(root) = value.filter(|value| !value.is_empty()) {
            let root = PathBuf::from(root);
            if root.is_file() {
                return Err(SettingsStoreError::UserRootIsFile { path: root });
            }
            return Ok(root);
        }
        std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .filter(|home| !home.is_empty())
            .map(PathBuf::from)
            .map(|home| home.join(".zircon").join("editor"))
            .ok_or(SettingsStoreError::MissingUserHome)
    }
}

fn project_settings_path(project_root: &Path) -> PathBuf {
    ProjectPaths::resolve_path(project_root)
        .and_then(|root| {
            ProjectPaths::resolve_path_from(&root, Path::new(".zircon").join(SETTINGS_FILE_NAME))
        })
        .map(|path| path.into_operation_path())
        .unwrap_or_else(|_| project_root.join(".zircon").join(SETTINGS_FILE_NAME))
}

#[cfg(test)]
mod settings_paths_tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::project::ProjectPaths;

    use super::SettingsPaths;

    #[cfg(any(unix, windows))]
    #[test]
    fn project_settings_path_keeps_the_physical_project_identity() {
        let parent = unique_settings_project_root("physical-identity");
        let physical_project = parent.join("physical-project");
        fs::create_dir_all(&physical_project).unwrap();
        let project_alias = parent.join("project-alias");
        create_directory_link(&physical_project, &project_alias);

        let paths = SettingsPaths::from_roots(parent.join("user"), Some(&project_alias));
        let expected = ProjectPaths::resolve_existing_path(&physical_project)
            .unwrap()
            .join(".zircon/settings.toml");

        fs::remove_dir_all(&parent).unwrap();
        assert_eq!(paths.project(), Some(expected.as_path()));
    }

    fn unique_settings_project_root(case_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon-editor-settings-{case_name}-{}-{timestamp}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    #[cfg(unix)]
    fn create_directory_link(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("create settings project alias");
    }

    #[cfg(windows)]
    fn create_directory_link(target: &Path, link: &Path) {
        let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
        let output = std::process::Command::new("cmd")
            .args(["/D", "/S", "/C"])
            .arg(command)
            .output()
            .expect("start mklink for settings project alias");
        assert!(
            output.status.success(),
            "create settings project junction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsStore {
    paths: SettingsPaths,
}

/// A versioned settings document encoded under the authority lock and ready for
/// the worker-owned atomic writer.
pub(crate) struct PreparedSettingsWrite {
    path: PathBuf,
    source: String,
}

impl SettingsStore {
    pub fn from_roots(user_root: impl Into<PathBuf>, project_root: Option<&Path>) -> Self {
        Self {
            paths: SettingsPaths::from_roots(user_root, project_root),
        }
    }

    pub fn from_user_environment() -> Result<Self, SettingsStoreError> {
        Ok(Self {
            paths: SettingsPaths::from_roots(SettingsPaths::user_root_from_environment()?, None),
        })
    }

    pub fn paths(&self) -> &SettingsPaths {
        &self.paths
    }

    pub(crate) fn with_project_root(&self, project_root: &Path) -> Self {
        Self {
            paths: SettingsPaths {
                user: self.paths.user.clone(),
                project: Some(project_settings_path(project_root)),
            },
        }
    }

    pub fn load_into(
        &self,
        scope: SettingsScope,
        registry: &mut SettingsRegistry,
    ) -> Result<SettingsLoad, SettingsStoreError> {
        let path = self.path_for(scope)?.to_path_buf();
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Ok(SettingsLoad::Missing { path });
            }
            Err(source) => return Err(SettingsStoreError::Read { path, source }),
        };
        let document =
            decode_current_document(&source).map_err(|source| SettingsStoreError::Decode {
                path: path.clone(),
                source,
            })?;
        let changes = registry
            .replace_persistent_layer(scope, document.values)
            .map_err(|source| SettingsStoreError::Apply {
                path: path.clone(),
                source,
            })?;
        Ok(SettingsLoad::Loaded {
            path,
            schema_version: SettingsDocument::VERSION,
            changes,
        })
    }

    pub(crate) fn load_authority_layer(
        &self,
        scope: SettingsScope,
        authority: &SettingsAuthority,
    ) -> Result<SettingsLoad, SettingsStoreError> {
        let path = self.path_for(scope)?.to_path_buf();
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Ok(SettingsLoad::Missing { path });
            }
            Err(source) => return Err(SettingsStoreError::Read { path, source }),
        };
        let document =
            decode_current_document(&source).map_err(|source| SettingsStoreError::Decode {
                path: path.clone(),
                source,
            })?;
        let changes = authority
            .replace_persistent_layer(scope, document.values)
            .map_err(|source| SettingsStoreError::Apply {
                path: path.clone(),
                source,
            })?;
        Ok(SettingsLoad::Loaded {
            path,
            schema_version: SettingsDocument::VERSION,
            changes,
        })
    }

    pub fn save_from(
        &self,
        scope: SettingsScope,
        registry: &SettingsRegistry,
    ) -> Result<(), SettingsStoreError> {
        Self::write_prepared(self.prepare_registry_layer(scope, registry)?)
    }

    pub(crate) fn save_authority_layer(
        &self,
        scope: SettingsScope,
        authority: &SettingsAuthority,
    ) -> Result<(), SettingsStoreError> {
        let Some(write) = authority.prepare_persistent_layer_for_write(scope, self)? else {
            return Ok(());
        };
        Self::write_prepared(write)
    }

    pub(crate) fn prepare_registry_layer(
        &self,
        scope: SettingsScope,
        registry: &SettingsRegistry,
    ) -> Result<PreparedSettingsWrite, SettingsStoreError> {
        let path = self.path_for(scope)?.to_path_buf();
        let values =
            registry
                .persistent_values(scope)
                .map_err(|source| SettingsStoreError::Apply {
                    path: path.clone(),
                    source,
                })?;
        let document = SettingsDocumentView { values };
        let source = write_versioned_text(&document).map_err(SettingsStoreError::Encode)?;
        Ok(PreparedSettingsWrite { path, source })
    }

    pub(crate) fn write_prepared(write: PreparedSettingsWrite) -> Result<(), SettingsStoreError> {
        let path = write.path.clone();
        write_atomically(&write.path, write.source.as_bytes())
            .map_err(|source| SettingsStoreError::Write { path, source })
    }

    fn path_for(&self, scope: SettingsScope) -> Result<&Path, SettingsStoreError> {
        match scope {
            SettingsScope::User => Ok(self.paths.user()),
            SettingsScope::Project => self
                .paths
                .project()
                .ok_or(SettingsStoreError::ProjectRootRequired),
            SettingsScope::Session => Err(SettingsStoreError::NonPersistentScope(scope)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SettingsLoad {
    Missing {
        path: PathBuf,
    },
    Loaded {
        path: PathBuf,
        schema_version: u32,
        changes: Vec<super::SettingChange>,
    },
}

#[derive(Debug, Error)]
pub enum SettingsStoreError {
    #[error("editor settings require USERPROFILE or HOME when {SETTINGS_USER_ROOT_ENV} is unset")]
    MissingUserHome,
    #[error(
        "editor settings root `{path}` is a file; {SETTINGS_USER_ROOT_ENV} must name a directory"
    )]
    UserRootIsFile { path: PathBuf },
    #[error("project settings require a project root")]
    ProjectRootRequired,
    #[error("{0:?} settings are session-only and cannot be persisted")]
    NonPersistentScope(SettingsScope),
    #[error("failed to read settings `{path}`: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to decode settings `{path}`: {source}")]
    Decode {
        path: PathBuf,
        #[source]
        source: SettingsDecodeError,
    },
    #[error("settings `{path}` contain entries invalid for the registered schema: {source}")]
    Apply {
        path: PathBuf,
        #[source]
        source: SettingsError,
    },
    #[error("failed to encode settings: {0}")]
    Encode(#[source] WriteError),
    #[error("failed to write settings `{path}`: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum SettingsDecodeError {
    #[error("settings must use the current Zircon versioned text envelope")]
    LegacyPayload,
    #[error(transparent)]
    Versioned(#[from] LoadError),
}

/// The physical `settings.toml` location is specified by Editor17; its content is
/// the canonical Plan11 versioned text envelope, not an independently parsed TOML dialect.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SettingsDocument {
    pub(super) values: BTreeMap<SettingsKey, SettingValue>,
}

#[derive(Serialize)]
struct SettingsDocumentView<'a> {
    values: &'a BTreeMap<SettingsKey, SettingValue>,
}

impl VersionedSchema for SettingsDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.editor.settings");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<SettingsDocument> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_settings_document)]);
        &MIGRATIONS
    }
}

impl<'a> VersionedSchema for SettingsDocumentView<'a> {
    const SCHEMA: SchemaId = SettingsDocument::SCHEMA;
    const VERSION: u32 = SettingsDocument::VERSION;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<SettingsDocumentView<'static>> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_settings_document)]);
        &MIGRATIONS
    }
}

fn reject_legacy_settings_document(
    _value: serde_json::Value,
) -> Result<serde_json::Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "settings version zero is retired; create a current versioned settings document",
    ))
}

fn decode_current_document(source: &str) -> Result<SettingsDocument, SettingsDecodeError> {
    let root: serde_json::Value = match serde_json::from_str(source) {
        Ok(root) => root,
        Err(error) if source.trim_start().starts_with('{') => {
            return Err(SettingsDecodeError::Versioned(LoadError::MalformedText {
                source: error,
            }));
        }
        Err(_) => return Err(SettingsDecodeError::LegacyPayload),
    };
    if root
        .as_object()
        .and_then(|object| object.get("$zircon"))
        .is_none()
    {
        return Err(SettingsDecodeError::LegacyPayload);
    }
    let loaded = load_versioned::<SettingsDocument>(source.as_bytes(), Format::Text)?;
    if loaded.migrated_from.is_some() {
        return Err(SettingsDecodeError::LegacyPayload);
    }
    Ok(loaded.value)
}

fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let (temporary, mut file) = create_temporary_file(path)?;
    let write_result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        // std::fs::rename uses replacement semantics on Windows. The temp and target
        // share a parent directory, so the replacement is a single filesystem operation.
        fs::rename(&temporary, path)?;
        sync_parent_directory(path)?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

fn create_temporary_file(path: &Path) -> io::Result<(PathBuf, fs::File)> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("settings");
    for attempt in 0_u8..32 {
        let temporary = path.with_file_name(format!(
            ".{file_name}.{}.{}.{}.tmp",
            std::process::id(),
            unique,
            attempt
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => return Ok((temporary, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a unique settings temporary file",
    ))
}

#[cfg(not(windows))]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(windows)]
fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    // Windows cannot open a directory with std::fs::File. The flushed temp and
    // replace-capable MoveFileEx-backed rename provide the supported durability boundary.
    Ok(())
}
