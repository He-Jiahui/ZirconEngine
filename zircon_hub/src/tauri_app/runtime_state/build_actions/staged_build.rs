use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Take};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::de::DeserializeOwned;
use serde::Deserialize;
use zircon_runtime_interface::runtime_build_set::{
    ZrRuntimeArtifactIdentityV1, ZrRuntimeArtifactManifestV1, ZrRuntimeBuildModeV1,
    ZrRuntimeBuildSetExpectationV1, ZrRuntimeDigestV1, ZrRuntimeTargetModelV1,
};

use crate::error::HubError;

mod platform;

use platform::{
    editor_executable_name as platform_editor_executable_name,
    runtime_executable_name as platform_runtime_executable_name,
    runtime_library_name as platform_runtime_library_name,
};

const STAGING_MANIFEST_FILE_NAME: &str = "staging_manifest.json";
const STAGING_MANIFEST_SCHEMA_V1: u32 = 1;
const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
const MAX_STAGING_ARTIFACTS: usize = 100_000;
const STAGING_DIRECTORY_CREATE_ATTEMPTS: usize = 32;
static NEXT_STAGING_DIRECTORY_ID: AtomicU64 = AtomicU64::new(1);

pub(in super::super) struct QualifiedStagedBuild {
    build_set_id: String,
}

pub(super) fn qualified_build_log(qualified: &QualifiedStagedBuild, build_log: &str) -> String {
    let qualification = format!("qualified BuildSet {}", qualified.build_set_id);
    if build_log.is_empty() {
        qualification
    } else {
        format!("{qualification}\n{build_log}")
    }
}

pub(super) fn allocate_build_output_dir(output_dir: &Path) -> Result<PathBuf, HubError> {
    let parent = output_dir.join(".zircon-hub").join("build-staging");
    fs::create_dir_all(&parent).map_err(|error| {
        qualification_error(format!(
            "cannot create build staging root {}: {error}",
            parent.display()
        ))
    })?;
    for _ in 0..STAGING_DIRECTORY_CREATE_ATTEMPTS {
        let id = NEXT_STAGING_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!("{}-{id}", std::process::id()));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(qualification_error(format!(
                    "cannot create build staging directory {}: {error}",
                    candidate.display()
                )))
            }
        }
    }
    Err(qualification_error(format!(
        "cannot allocate a unique build staging directory below {}",
        parent.display()
    )))
}

pub(super) fn discard_build_output(build_output_dir: &Path) -> Result<(), HubError> {
    match fs::remove_dir_all(build_output_dir) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(qualification_error(format!(
            "cannot remove failed build staging directory {}: {error}",
            build_output_dir.display()
        ))),
    }
}

pub(super) fn activate_staged_engine(
    staged_engine_dir: &Path,
    active_engine_dir: &Path,
) -> Result<Option<String>, HubError> {
    let staged_metadata = fs::symlink_metadata(staged_engine_dir).map_err(|error| {
        qualification_error(format!(
            "cannot inspect qualified staging directory {}: {error}",
            staged_engine_dir.display()
        ))
    })?;
    if !staged_metadata.is_dir() || staged_metadata.file_type().is_symlink() {
        return Err(qualification_error(format!(
            "qualified staging directory {} is not a regular directory",
            staged_engine_dir.display()
        )));
    }
    let active_metadata = match fs::symlink_metadata(active_engine_dir) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(qualification_error(format!(
                "cannot inspect active BuildSet directory {}: {error}",
                active_engine_dir.display()
            )))
        }
    };
    if let Some(metadata) = active_metadata.as_ref() {
        if metadata.file_type().is_symlink() {
            return Err(qualification_error(format!(
                "active BuildSet directory {} must not be a symlink",
                active_engine_dir.display()
            )));
        }
        if !metadata.is_dir() {
            return Err(qualification_error(format!(
                "active BuildSet path {} must be a directory",
                active_engine_dir.display()
            )));
        }
    }
    let parent = active_engine_dir.parent().ok_or_else(|| {
        qualification_error(format!(
            "active BuildSet directory {} has no parent",
            active_engine_dir.display()
        ))
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        qualification_error(format!(
            "cannot create active BuildSet parent {}: {error}",
            parent.display()
        ))
    })?;
    let backup = parent.join(format!(
        ".{}.previous-{}-{}",
        active_engine_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("zircon-engine"),
        std::process::id(),
        NEXT_STAGING_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let had_active = active_metadata.is_some();
    if had_active {
        fs::rename(active_engine_dir, &backup).map_err(|error| {
            qualification_error(format!(
                "cannot move the previous active BuildSet {} aside: {error}",
                active_engine_dir.display()
            ))
        })?;
    }
    match fs::rename(staged_engine_dir, active_engine_dir) {
        Ok(()) => {
            let cleanup_warning = if had_active {
                fs::remove_dir_all(&backup).err().map(|error| {
                    format!(
                        "previous BuildSet backup cleanup pending at {}: {error}",
                        backup.display()
                    )
                })
            } else {
                None
            };
            Ok(cleanup_warning)
        }
        Err(error) => {
            let restore = if had_active {
                fs::rename(&backup, active_engine_dir).err()
            } else {
                None
            };
            let restore_detail = restore
                .map(|restore| format!("; restore of previous active BuildSet failed: {restore}"))
                .unwrap_or_default();
            Err(qualification_error(format!(
                "cannot atomically activate BuildSet {}: {error}{restore_detail}",
                staged_engine_dir.display()
            )))
        }
    }
}

pub(in super::super) fn validate_staged_editor_runtime_build(
    staged_engine_dir: &Path,
    expected_source_dir: &Path,
    expected_profile: &str,
) -> Result<QualifiedStagedBuild, HubError> {
    let staged_engine_dir = canonical_directory(staged_engine_dir, "staged engine directory")?;
    let expected_source_dir = canonical_directory(expected_source_dir, "Source Engine directory")?;
    let staging_manifest_path = staged_engine_dir.join(STAGING_MANIFEST_FILE_NAME);
    let staging: StagingManifestV1 = read_bounded_json(&staging_manifest_path)?;
    staging.validate(&expected_source_dir, expected_profile)?;

    let runtime_library_name = platform_runtime_library_name();
    let runtime_manifest_name = format!("{runtime_library_name}.manifest.json");
    let runtime_manifest_path = staged_engine_dir.join(&runtime_manifest_name);
    let runtime_manifest: ZrRuntimeArtifactManifestV1 = read_bounded_json(&runtime_manifest_path)?;
    validate_runtime_manifest(
        &runtime_manifest,
        expected_profile,
        runtime_library_name,
        &staging.build.runtime_features,
    )?;

    let artifacts = staging.artifacts_by_target_path()?;
    validate_required_artifact(
        &staged_engine_dir,
        &artifacts,
        runtime_library_name,
        "runtime.library",
        &runtime_manifest.artifact.sha256,
    )?;
    validate_required_artifact(
        &staged_engine_dir,
        &artifacts,
        &runtime_manifest_name,
        "runtime.library.manifest",
        artifacts
            .get(&runtime_manifest_name)
            .map(|artifact| &artifact.sha256)
            .ok_or_else(|| {
                qualification_error(format!(
                    "{runtime_manifest_name} is absent from {STAGING_MANIFEST_FILE_NAME}"
                ))
            })?,
    )?;

    for (file_name, logical_artifact) in [
        (platform_editor_executable_name(), "editor.executable"),
        (platform_runtime_executable_name(), "runtime.executable"),
    ] {
        let identity = required_host_artifact(&runtime_manifest, file_name)?;
        validate_required_artifact(
            &staged_engine_dir,
            &artifacts,
            file_name,
            logical_artifact,
            &identity.sha256,
        )?;
    }

    Ok(QualifiedStagedBuild {
        build_set_id: runtime_manifest.build_set_id.as_str().to_string(),
    })
}

fn validate_runtime_manifest(
    manifest: &ZrRuntimeArtifactManifestV1,
    expected_profile: &str,
    expected_runtime_library: &str,
    staging_runtime_features: &[String],
) -> Result<(), HubError> {
    let expected_mode = build_mode(expected_profile)?;
    if manifest.build_mode != expected_mode {
        return Err(qualification_error(format!(
            "Runtime manifest mode {:?} does not match requested profile {expected_profile}",
            manifest.build_mode
        )));
    }
    if manifest.artifact.file_name != expected_runtime_library {
        return Err(qualification_error(format!(
            "Runtime manifest selects {}, expected {expected_runtime_library}",
            manifest.artifact.file_name
        )));
    }
    let staging_runtime_features = staging_runtime_features
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if staging_runtime_features.len() != 1
        || !staging_runtime_features.contains("target-client")
        || &staging_runtime_features != &manifest.runtime_features
    {
        return Err(qualification_error(format!(
            "Runtime features {:?} do not match the editor/runtime staging contract {:?}",
            manifest.runtime_features, staging_runtime_features
        )));
    }
    let editor = required_host_artifact(manifest, platform_editor_executable_name())?.clone();
    required_host_artifact(manifest, platform_runtime_executable_name())?;
    let expectation = ZrRuntimeBuildSetExpectationV1::new(
        manifest.build_set_id.clone(),
        ZrRuntimeTargetModelV1::current(),
        std::iter::empty::<String>(),
    )
    .map_err(|error| qualification_error(error.to_string()))?
    .with_host_artifact(editor);
    manifest
        .validate_against(&expectation)
        .map_err(|error| qualification_error(error.to_string()))
}

fn required_host_artifact<'a>(
    manifest: &'a ZrRuntimeArtifactManifestV1,
    file_name: &str,
) -> Result<&'a ZrRuntimeArtifactIdentityV1, HubError> {
    manifest
        .host_artifacts
        .iter()
        .find(|artifact| artifact.file_name == file_name)
        .ok_or_else(|| {
            qualification_error(format!(
                "Runtime manifest does not include required host artifact {file_name}"
            ))
        })
}

fn validate_required_artifact(
    staged_engine_dir: &Path,
    artifacts: &BTreeMap<String, &StagingArtifactV1>,
    target_path: &str,
    expected_logical_artifact: &str,
    expected_digest: &ZrRuntimeDigestV1,
) -> Result<(), HubError> {
    let entry = artifacts.get(target_path).ok_or_else(|| {
        qualification_error(format!(
            "required artifact {target_path} is absent from {STAGING_MANIFEST_FILE_NAME}"
        ))
    })?;
    if entry.logical_artifact != expected_logical_artifact {
        return Err(qualification_error(format!(
            "artifact {target_path} is classified as {}, expected {expected_logical_artifact}",
            entry.logical_artifact
        )));
    }
    if &entry.sha256 != expected_digest {
        return Err(qualification_error(format!(
            "artifact {target_path} has inconsistent digests across staging and Runtime manifests"
        )));
    }
    let actual_digest = hash_regular_file(&staged_engine_dir.join(target_path))?;
    if &actual_digest != expected_digest {
        return Err(qualification_error(format!(
            "artifact {target_path} does not match its staged SHA-256 digest"
        )));
    }
    Ok(())
}

fn read_bounded_json<T: DeserializeOwned>(path: &Path) -> Result<T, HubError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        qualification_error(format!("cannot inspect {}: {error}", path.display()))
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(qualification_error(format!(
            "{} must be a regular non-symlink file",
            path.display()
        )));
    }
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(qualification_error(format!(
            "{} exceeds the {} byte manifest limit",
            path.display(),
            MAX_MANIFEST_BYTES
        )));
    }
    let file = File::open(path)
        .map_err(|error| qualification_error(format!("cannot open {}: {error}", path.display())))?;
    let reader: Take<File> = file.take(MAX_MANIFEST_BYTES + 1);
    serde_json::from_reader(reader)
        .map_err(|error| qualification_error(format!("cannot decode {}: {error}", path.display())))
}

fn hash_regular_file(path: &Path) -> Result<ZrRuntimeDigestV1, HubError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        qualification_error(format!("cannot inspect {}: {error}", path.display()))
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || metadata.len() == 0 {
        return Err(qualification_error(format!(
            "{} must be a non-empty regular non-symlink file",
            path.display()
        )));
    }
    let mut file = File::open(path)
        .map_err(|error| qualification_error(format!("cannot open {}: {error}", path.display())))?;
    ZrRuntimeDigestV1::sha256_reader(&mut file)
        .map_err(|error| qualification_error(format!("cannot hash {}: {error}", path.display())))
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, HubError> {
    let canonical = path.canonicalize().map_err(|error| {
        qualification_error(format!(
            "cannot resolve {label} {}: {error}",
            path.display()
        ))
    })?;
    if !canonical.is_dir() {
        return Err(qualification_error(format!(
            "{label} {} is not a directory",
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn build_mode(profile: &str) -> Result<ZrRuntimeBuildModeV1, HubError> {
    match profile {
        "debug" => Ok(ZrRuntimeBuildModeV1::Debug),
        "release" => Ok(ZrRuntimeBuildModeV1::Release),
        "profiling" => Ok(ZrRuntimeBuildModeV1::Profiling),
        other => Err(qualification_error(format!(
            "unsupported build profile {other}"
        ))),
    }
}

fn qualification_error(message: impl Into<String>) -> HubError {
    HubError::message(format!(
        "staged BuildSet qualification failed: {}",
        message.into()
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StagingManifestV1 {
    schema_version: u32,
    source_repository: PathBuf,
    build: StagingBuildV1,
    artifacts: Vec<StagingArtifactV1>,
}

impl StagingManifestV1 {
    fn validate(&self, expected_source_dir: &Path, expected_profile: &str) -> Result<(), HubError> {
        if self.schema_version != STAGING_MANIFEST_SCHEMA_V1 {
            return Err(qualification_error(format!(
                "staging manifest schema {} is unsupported",
                self.schema_version
            )));
        }
        let source_repository =
            canonical_directory(&self.source_repository, "manifest source repository")?;
        if source_repository != expected_source_dir {
            return Err(qualification_error(format!(
                "staging manifest source {} does not match bound Source Engine {}",
                source_repository.display(),
                expected_source_dir.display()
            )));
        }
        if self.build.mode != expected_profile {
            return Err(qualification_error(format!(
                "staging manifest mode {} does not match requested profile {expected_profile}",
                self.build.mode
            )));
        }
        let targets = self
            .build
            .targets
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if self.build.targets.len() != 2 || targets != BTreeSet::from(["editor", "runtime"]) {
            return Err(qualification_error(format!(
                "staging manifest targets {:?} do not describe the editor/runtime BuildSet",
                self.build.targets
            )));
        }
        if self.artifacts.len() > MAX_STAGING_ARTIFACTS {
            return Err(qualification_error(format!(
                "staging manifest contains {} artifacts; limit is {MAX_STAGING_ARTIFACTS}",
                self.artifacts.len()
            )));
        }
        for artifact in &self.artifacts {
            artifact.validate()?;
        }
        Ok(())
    }

    fn artifacts_by_target_path(&self) -> Result<BTreeMap<String, &StagingArtifactV1>, HubError> {
        let mut indexed = BTreeMap::new();
        for artifact in &self.artifacts {
            if indexed
                .insert(artifact.target_path.clone(), artifact)
                .is_some()
            {
                return Err(qualification_error(format!(
                    "staging manifest repeats target path {}",
                    artifact.target_path
                )));
            }
        }
        Ok(indexed)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StagingBuildV1 {
    mode: String,
    targets: Vec<String>,
    runtime_features: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StagingArtifactV1 {
    logical_artifact: String,
    source: StagingSourceV1,
    target_path: String,
    sha256: ZrRuntimeDigestV1,
}

impl StagingArtifactV1 {
    fn validate(&self) -> Result<(), HubError> {
        let path = Path::new(&self.target_path);
        if self.logical_artifact.is_empty()
            || self.source.kind.is_empty()
            || self.source.path.is_empty()
            || self.target_path.is_empty()
            || path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(qualification_error(format!(
                "staging manifest contains invalid artifact entry {}",
                self.target_path
            )));
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StagingSourceV1 {
    kind: String,
    path: String,
}

#[cfg(test)]
pub(in super::super) fn write_valid_test_fixture(
    staged_engine_dir: &Path,
    source_dir: &Path,
    profile: &str,
) {
    tests::write_valid_fixture(staged_engine_dir, source_dir, profile);
}

#[cfg(test)]
#[path = "tests/staged_build.rs"]
mod tests;
