use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::contract::hash_file;

pub(super) const PROJECT_MANIFEST_SOURCE_PATH: &str =
    zircon_runtime::asset::project::PROJECT_MANIFEST_FILE;
pub(super) const MAIN_SCENE_SOURCE_PATH: &str = "assets/scenes/main.scene.toml";
pub(super) const EMPTY_SCENE_SOURCE_PATH: &str = "assets/scenes/workbench-review-empty.scene.toml";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ManagedSceneFingerprint {
    pub(super) project_root: String,
    pub(super) sources: Vec<ManagedInputFingerprint>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ManagedInputFingerprint {
    pub(super) source_path: String,
    pub(super) sha256: String,
}

pub(super) fn capture(project_root: &Path) -> Result<ManagedSceneFingerprint, String> {
    let canonical_root = canonical_root(project_root)?;
    let fingerprint = ManagedSceneFingerprint {
        project_root: canonical_root.to_string_lossy().into_owned(),
        sources: fixed_sources(&canonical_root)?,
    };
    validate_fingerprint(&fingerprint, &canonical_root)?;
    Ok(fingerprint)
}

pub(super) fn validate_value(
    value: &Value,
    expected_project_root: &Path,
) -> Result<ManagedSceneFingerprint, String> {
    let fingerprint: ManagedSceneFingerprint = serde_json::from_value(value.clone())
        .map_err(|error| format!("managedSceneFingerprint has an unsupported shape: {error}"))?;
    let canonical_root = canonical_root(expected_project_root)?;
    validate_fingerprint(&fingerprint, &canonical_root)?;
    Ok(fingerprint)
}

fn validate_fingerprint(
    fingerprint: &ManagedSceneFingerprint,
    canonical_root: &Path,
) -> Result<(), String> {
    if fingerprint.project_root.as_str() != canonical_root.to_string_lossy().as_ref() {
        return Err("managedSceneFingerprint projectRoot does not match the bound project".into());
    }
    if fingerprint.sources.len() != 3 {
        return Err("managedSceneFingerprint must contain exactly three sources".into());
    }
    let expected_paths = [
        PROJECT_MANIFEST_SOURCE_PATH,
        MAIN_SCENE_SOURCE_PATH,
        EMPTY_SCENE_SOURCE_PATH,
    ];
    for (index, expected_path) in expected_paths.into_iter().enumerate() {
        let source = &fingerprint.sources[index];
        if source.source_path != expected_path {
            return Err(format!(
                "managedSceneFingerprint source {} must be {}",
                index, expected_path
            ));
        }
        if !is_sha256(&source.sha256) {
            return Err(format!(
                "managedSceneFingerprint source {} has an invalid SHA-256",
                index
            ));
        }
        let source_path = resolve_fixed_source(canonical_root, expected_path)?;
        let actual = hash_file(&source_path)?;
        if actual != source.sha256 {
            return Err(format!(
                "managedSceneFingerprint source is stale: {}",
                expected_path
            ));
        }
    }
    Ok(())
}

fn fixed_sources(canonical_root: &Path) -> Result<Vec<ManagedInputFingerprint>, String> {
    [
        PROJECT_MANIFEST_SOURCE_PATH,
        MAIN_SCENE_SOURCE_PATH,
        EMPTY_SCENE_SOURCE_PATH,
    ]
    .into_iter()
    .map(|source_path| {
        Ok(ManagedInputFingerprint {
            source_path: source_path.to_owned(),
            sha256: hash_file(&resolve_fixed_source(canonical_root, source_path)?)?,
        })
    })
    .collect()
}

fn resolve_fixed_source(root: &Path, relative_path: &str) -> Result<std::path::PathBuf, String> {
    let lexical_path = root.join(relative_path);
    let canonical_path = std::fs::canonicalize(&lexical_path).map_err(|error| {
        format!(
            "cannot resolve managed project input {}: {error}",
            relative_path
        )
    })?;
    if !canonical_path.starts_with(root) {
        return Err(format!(
            "managed project input escapes its bound project: {relative_path}"
        ));
    }
    if !same_physical_path(&canonical_path, &lexical_path) {
        return Err(format!(
            "managed project input resolves through a path alias: {relative_path}"
        ));
    }
    if !canonical_path.is_file() {
        return Err(format!(
            "managed project input is not a file: {relative_path}"
        ));
    }
    Ok(canonical_path)
}

#[cfg(windows)]
fn normalize_physical(path: &Path) -> String {
    let normalized = path
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    normalized
        .strip_prefix("\\\\?\\")
        .unwrap_or(&normalized)
        .to_owned()
}

#[cfg(not(windows))]
fn normalize_physical(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
}

#[cfg(windows)]
fn same_physical_path(left: &Path, right: &Path) -> bool {
    normalize_physical(left) == normalize_physical(right)
}

#[cfg(not(windows))]
fn same_physical_path(left: &Path, right: &Path) -> bool {
    left == right
}

fn canonical_root(path: &Path) -> Result<std::path::PathBuf, String> {
    if !path.is_absolute() {
        return Err("managed review project root must be absolute".into());
    }
    let canonical = std::fs::canonicalize(path).map_err(|error| {
        format!(
            "cannot resolve managed review project root {}: {error}",
            path.display()
        )
    })?;
    if !canonical.is_dir() {
        return Err(format!(
            "managed review project root is not a directory: {}",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[cfg(test)]
#[path = "tests/managed_inputs.rs"]
mod tests;
