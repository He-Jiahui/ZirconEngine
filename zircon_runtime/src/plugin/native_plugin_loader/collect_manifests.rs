use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::PLUGIN_MANIFEST_FILE;

pub(super) type NativePluginManifestCollectionResult<T> =
    std::result::Result<T, NativePluginManifestCollectionError>;

#[derive(Debug)]
pub(super) enum NativePluginManifestCollectionError {
    EnumerateRoot { root: PathBuf, source: io::Error },
    InspectEntry { root: PathBuf, source: io::Error },
}

impl std::fmt::Display for NativePluginManifestCollectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnumerateRoot { root, source } => write!(
                formatter,
                "failed to enumerate native plugin root {}: {source}",
                root.display()
            ),
            Self::InspectEntry { root, source } => write!(
                formatter,
                "failed to inspect native plugin entry under {}: {source}",
                root.display()
            ),
        }
    }
}

impl std::error::Error for NativePluginManifestCollectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EnumerateRoot { source, .. } | Self::InspectEntry { source, .. } => Some(source),
        }
    }
}

pub(super) fn collect_plugin_manifests(
    root: &Path,
    manifest_paths: &mut Vec<PathBuf>,
) -> NativePluginManifestCollectionResult<()> {
    let entries = fs::read_dir(root).map_err(|source| {
        NativePluginManifestCollectionError::EnumerateRoot {
            root: root.to_path_buf(),
            source,
        }
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| NativePluginManifestCollectionError::InspectEntry {
            root: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_plugin_manifests(&path, manifest_paths)?;
        } else if path.file_name().and_then(|value| value.to_str()) == Some(PLUGIN_MANIFEST_FILE) {
            manifest_paths.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_plugin_manifests_reports_enumerate_root_with_typed_error() {
        let missing_root = std::env::temp_dir().join(format!(
            "zircon-missing-native-plugin-root-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        let mut manifest_paths = Vec::new();
        let error = collect_plugin_manifests(&missing_root, &mut manifest_paths)
            .expect_err("missing root should report typed manifest collection error");

        match error {
            NativePluginManifestCollectionError::EnumerateRoot { root, .. } => {
                assert_eq!(root, missing_root);
            }
            NativePluginManifestCollectionError::InspectEntry { .. } => {
                panic!("missing root should fail while enumerating root")
            }
        }
    }

    #[test]
    fn manifest_collection_typed_error_preserves_inspect_entry_message() {
        let root = PathBuf::from("native-plugin-root");
        let error = NativePluginManifestCollectionError::InspectEntry {
            root: root.clone(),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "entry unavailable"),
        };

        assert_eq!(
            error.to_string(),
            "failed to inspect native plugin entry under native-plugin-root: entry unavailable"
        );
        assert!(
            std::error::Error::source(&error).is_some(),
            "inspect-entry error should preserve io::Error source"
        );
    }
}
