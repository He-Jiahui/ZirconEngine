use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::asset::project::ProjectPaths;
use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::{AssetImportContext, AssetImportError};

const MAX_AUXILIARY_READ_CAPACITY_HINT: usize = 8 * 1024 * 1024;

#[path = "auxiliary_source/opened_snapshot.rs"]
mod opened_snapshot;
pub(crate) use opened_snapshot::open_admitted_file;

pub(crate) struct AuxiliarySourceResolver {
    asset_root: PathBuf,
    base_dir: PathBuf,
}

impl AuxiliarySourceResolver {
    pub(crate) const MAX_SNAPSHOT_FILES: usize = 65_536;

    pub(super) fn new(
        context: &AssetImportContext,
        base_dir: &Path,
    ) -> Result<Self, AssetImportError> {
        Self::for_source(&context.source_path, &context.uri, base_dir)
    }

    pub(crate) fn for_source(
        source_path: &Path,
        uri: &crate::asset::AssetUri,
        base_dir: &Path,
    ) -> Result<Self, AssetImportError> {
        let asset_root = source_asset_root(source_path, uri)?;
        Self::for_resolved_source_root(&asset_root, base_dir)
    }

    pub(crate) fn for_asset_root(
        asset_root: &Path,
        base_dir: &Path,
    ) -> Result<Self, AssetImportError> {
        if !asset_root.is_absolute() || !base_dir.starts_with(asset_root) {
            return Err(auxiliary_error(
                "source base is outside its admitted asset root",
            ));
        }
        Ok(Self {
            asset_root: asset_root.to_path_buf(),
            base_dir: base_dir.to_path_buf(),
        })
    }

    fn for_resolved_source_root(
        asset_root: &Path,
        base_dir: &Path,
    ) -> Result<Self, AssetImportError> {
        let asset_root = ProjectPaths::resolve_path(&asset_root).map_err(|source| {
            auxiliary_error(format!(
                "resolve asset root {}: {source}",
                ProjectPaths::display_path(&asset_root).display()
            ))
        })?;
        let base_dir = ProjectPaths::resolve_path(base_dir).map_err(|source| {
            auxiliary_error(format!(
                "resolve auxiliary base directory {}: {source}",
                ProjectPaths::display_path(base_dir).display()
            ))
        })?;
        if !base_dir
            .operation_path()
            .starts_with(asset_root.operation_path())
        {
            return Err(auxiliary_error(format!(
                "auxiliary base directory {} escapes asset root {}",
                base_dir.display_path().display(),
                asset_root.display_path().display()
            )));
        }
        Ok(Self {
            asset_root: asset_root.into_operation_path(),
            base_dir: base_dir.into_operation_path(),
        })
    }

    pub(super) fn resolve_path(&self, reference: &Path) -> Result<PathBuf, AssetImportError> {
        let candidate = self.resolve_lexical_path(reference)?;
        self.admit_existing_file(&candidate)
    }

    pub(super) fn resolve_lexical_path(
        &self,
        reference: &Path,
    ) -> Result<PathBuf, AssetImportError> {
        reject_root_or_scheme(reference)?;
        let mut candidate = self.base_dir.clone();
        for component in reference.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    if candidate == self.asset_root || !candidate.pop() {
                        return Err(self.escape_error(reference));
                    }
                }
                Component::Normal(name) => candidate.push(name),
                Component::Prefix(_) | Component::RootDir => {
                    return Err(auxiliary_error(format!(
                        "auxiliary source {} must not use a drive, UNC path, or filesystem root",
                        ProjectPaths::display_path(reference).display()
                    )));
                }
            }
            if !candidate.starts_with(&self.asset_root) {
                return Err(self.escape_error(reference));
            }
        }
        Ok(candidate)
    }

    pub(super) fn resolve_gltf_uri(&self, uri: &str) -> Result<PathBuf, AssetImportError> {
        if uri
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
        {
            return Err(auxiliary_error(
                "embedded glTF data URIs do not name auxiliary files",
            ));
        }
        if uri.contains(':') {
            return Err(auxiliary_error(format!(
                "glTF auxiliary source `{uri}` must not be scheme-qualified"
            )));
        }
        let decoded = percent_decode_uri_path(uri)?;
        self.resolve_path(Path::new(&decoded))
    }

    pub(crate) fn resolve_gltf_uri_lexical(&self, uri: &str) -> Result<PathBuf, AssetImportError> {
        if uri
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
        {
            return Err(auxiliary_error(
                "embedded glTF data URIs do not name auxiliary files",
            ));
        }
        if uri.contains(':') {
            return Err(auxiliary_error(format!(
                "glTF auxiliary source `{uri}` must not be scheme-qualified"
            )));
        }
        let decoded = percent_decode_uri_path(uri)?;
        self.resolve_lexical_path(Path::new(&decoded))
    }

    pub(crate) fn read_gltf_uri_snapshot(
        &self,
        uri: &str,
        limit: u64,
    ) -> Result<(PathBuf, Vec<u8>, u64), AssetImportError> {
        if uri
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
        {
            return Err(auxiliary_error(
                "embedded glTF data URIs do not name auxiliary files",
            ));
        }
        if uri.contains(':') {
            return Err(auxiliary_error(format!(
                "glTF auxiliary source `{uri}` must not be scheme-qualified"
            )));
        }
        let decoded = percent_decode_uri_path(uri)?;
        self.read_path_snapshot(Path::new(&decoded), limit)
    }

    pub(crate) fn read_path_snapshot(
        &self,
        reference: &Path,
        limit: u64,
    ) -> Result<(PathBuf, Vec<u8>, u64), AssetImportError> {
        self.read_path_snapshot_with_hooks(reference, limit, || {}, || {})
    }

    fn read_path_snapshot_with_hooks(
        &self,
        reference: &Path,
        limit: u64,
        after_admission: impl FnOnce(),
        after_open: impl FnOnce(),
    ) -> Result<(PathBuf, Vec<u8>, u64), AssetImportError> {
        let admitted = self.resolve_path(reference)?;
        after_admission();
        let file =
            opened_snapshot::open_admitted_file(&admitted, &self.asset_root).map_err(|source| {
                auxiliary_error(format!(
                    "open auxiliary source {}: {source}",
                    ProjectPaths::display_path(&admitted).display()
                ))
            })?;
        after_open();
        let metadata = file.metadata().map_err(|source| {
            auxiliary_error(format!(
                "inspect opened auxiliary source {}: {source}",
                ProjectPaths::display_path(&admitted).display()
            ))
        })?;
        let metadata_len = metadata.len();
        let metadata_modified = metadata.modified().ok();
        let mtime_unix_ms = metadata_modified
            .and_then(|mtime| mtime.duration_since(std::time::UNIX_EPOCH).ok())
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
            .unwrap_or_default();
        if metadata_len > limit {
            return Err(auxiliary_error(format!(
                "auxiliary source {} is {metadata_len} bytes, exceeding the {limit}-byte limit",
                ProjectPaths::display_path(&admitted).display()
            )));
        }
        let mut bytes = Vec::with_capacity(
            usize::try_from(metadata_len)
                .unwrap_or(usize::MAX)
                .min(MAX_AUXILIARY_READ_CAPACITY_HINT),
        );
        let mut reader = file.take(limit.saturating_add(1));
        reader.read_to_end(&mut bytes).map_err(|source| {
            auxiliary_error(format!(
                "read auxiliary source {}: {source}",
                ProjectPaths::display_path(&admitted).display()
            ))
        })?;
        if bytes.len() as u64 > limit {
            return Err(auxiliary_error(format!(
                "auxiliary source {} grew beyond the {limit}-byte limit while it was read",
                ProjectPaths::display_path(&admitted).display()
            )));
        }
        let file = reader.into_inner();
        let final_metadata = file.metadata().map_err(|source| {
            auxiliary_error(format!(
                "reinspect opened auxiliary source {}: {source}",
                ProjectPaths::display_path(&admitted).display()
            ))
        })?;
        if final_metadata.len() != metadata_len
            || final_metadata.modified().ok() != metadata_modified
        {
            return Err(auxiliary_error(format!(
                "auxiliary source {} changed while it was read",
                ProjectPaths::display_path(&admitted).display()
            )));
        }
        Ok((admitted, bytes, mtime_unix_ms))
    }

    pub(super) fn root_relative_path<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<&'a Path, AssetImportError> {
        path.strip_prefix(&self.asset_root)
            .map_err(|_| self.escape_error(path))
    }

    fn admit_existing_file(&self, candidate: &Path) -> Result<PathBuf, AssetImportError> {
        self.validate_existing_components(candidate)?;
        let relative = candidate
            .strip_prefix(&self.asset_root)
            .map_err(|_| self.escape_error(candidate))?;
        let mut inspected = self.asset_root.clone();
        for component in relative.components() {
            let Component::Normal(name) = component else {
                return Err(self.escape_error(candidate));
            };
            inspected.push(name);
            let metadata = fs::symlink_metadata(&inspected).map_err(|source| {
                auxiliary_error(format!(
                    "inspect auxiliary source {}: {source}",
                    ProjectPaths::display_path(&inspected).display()
                ))
            })?;
            if is_link_or_reparse(&metadata) {
                return Err(auxiliary_error(format!(
                    "auxiliary source {} traverses a symbolic link or Windows reparse point",
                    ProjectPaths::display_path(&inspected).display()
                )));
            }
        }
        let metadata = fs::symlink_metadata(candidate).map_err(|source| {
            auxiliary_error(format!(
                "inspect auxiliary source {}: {source}",
                ProjectPaths::display_path(candidate).display()
            ))
        })?;
        if !metadata.is_file() {
            return Err(auxiliary_error(format!(
                "auxiliary source {} is not a regular file",
                ProjectPaths::display_path(candidate).display()
            )));
        }
        let physical = ProjectPaths::resolve_existing_path(candidate).map_err(|source| {
            auxiliary_error(format!(
                "resolve auxiliary source {}: {source}",
                ProjectPaths::display_path(candidate).display()
            ))
        })?;
        if !opened_snapshot::path_is_within(&physical, &self.asset_root) {
            return Err(self.escape_error(candidate));
        }
        Ok(physical)
    }

    fn validate_existing_components(&self, candidate: &Path) -> Result<(), AssetImportError> {
        let relative = candidate
            .strip_prefix(&self.asset_root)
            .map_err(|_| self.escape_error(candidate))?;
        let mut inspected = self.asset_root.clone();
        for component in relative.components() {
            let Component::Normal(name) = component else {
                return Err(self.escape_error(candidate));
            };
            inspected.push(name);
            let Ok(metadata) = fs::symlink_metadata(&inspected) else {
                continue;
            };
            if is_link_or_reparse(&metadata) {
                return Err(auxiliary_error(format!(
                    "auxiliary source {} traverses a symbolic link or Windows reparse point",
                    ProjectPaths::display_path(&inspected).display()
                )));
            }
        }
        Ok(())
    }

    fn escape_error(&self, reference: &Path) -> AssetImportError {
        auxiliary_error(format!(
            "auxiliary source {} escapes asset root {}",
            ProjectPaths::display_path(reference).display(),
            ProjectPaths::display_path(&self.asset_root).display()
        ))
    }
}

fn source_asset_root(
    source_path: &Path,
    uri: &crate::asset::AssetUri,
) -> Result<PathBuf, AssetImportError> {
    let mut root = source_path.to_path_buf();
    let logical_path = uri.package_path().unwrap_or_else(|| uri.path());
    let logical_components = Path::new(logical_path)
        .components()
        .filter(|component| matches!(component, Component::Normal(_)))
        .count();
    if logical_components == 0 {
        return Err(auxiliary_error("asset URI has no source path components"));
    }
    for _ in 0..logical_components {
        if !root.pop() {
            return Err(auxiliary_error(format!(
                "asset URI {} has more components than source path {}",
                uri,
                ProjectPaths::display_path(source_path).display()
            )));
        }
    }
    Ok(root)
}

fn reject_root_or_scheme(reference: &Path) -> Result<(), AssetImportError> {
    if reference.as_os_str().is_empty() {
        return Err(auxiliary_error("auxiliary source path is empty"));
    }
    if reference.is_absolute()
        || reference.has_root()
        || reference
            .components()
            .any(|component| matches!(component, Component::Prefix(_) | Component::RootDir))
    {
        return Err(auxiliary_error(format!(
            "auxiliary source {} must not use a drive, UNC path, or filesystem root",
            ProjectPaths::display_path(reference).display()
        )));
    }
    if os_str_contains_colon(reference.as_os_str()) {
        return Err(auxiliary_error(format!(
            "auxiliary source {} must not be scheme-qualified",
            ProjectPaths::display_path(reference).display()
        )));
    }
    Ok(())
}

fn os_str_contains_colon(value: &OsStr) -> bool {
    value.to_string_lossy().contains(':')
}

fn percent_decode_uri_path(uri: &str) -> Result<String, AssetImportError> {
    let bytes = uri.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let (Some(high), Some(low)) = (
                bytes.get(index + 1).copied().and_then(hex_value),
                bytes.get(index + 2).copied().and_then(hex_value),
            ) else {
                return Err(auxiliary_error(format!(
                    "glTF auxiliary source `{uri}` contains a malformed percent escape"
                )));
            };
            decoded.push((high << 4) | low);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(decoded)
        .map_err(|_| auxiliary_error(format!("glTF auxiliary source `{uri}` is not UTF-8")))
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn auxiliary_error(message: impl Into<String>) -> AssetImportError {
    AssetImportError::Parse(format!(
        "auxiliary source path rejected: {}",
        message.into()
    ))
}

#[cfg(test)]
#[path = "tests/auxiliary_source_opened_snapshot_tests.rs"]
mod opened_snapshot_tests;
