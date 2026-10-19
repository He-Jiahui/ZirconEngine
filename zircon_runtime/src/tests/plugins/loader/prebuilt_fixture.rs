//! Read-only admission of Root-built DLL fixtures; no subprocess or source preparation.
use std::collections::BTreeSet;
use std::fs::File;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::asset::project::{ProjectPaths, ResolvedProjectPathIdentity};
use crate::plugin::native::NativePluginArtifactDigest;

const SOURCE_INPUTS: &[&str] = &[
    "zircon_plugins/Cargo.toml",
    "zircon_plugins/Cargo.lock",
    "zircon_plugins/native_dynamic_fixture/native/Cargo.toml",
    "zircon_plugins/native_dynamic_fixture/native/src/lib.rs",
    "zircon_plugins/native_dynamic_fixture/plugin.toml",
    "zircon_plugins/plugin_sdk/Cargo.toml",
    "zircon_plugins/plugin_sdk/src/lib.rs",
    "zircon_plugins/plugin_sdk/src/declaration.rs",
    "zircon_plugins/plugin_sdk/src/declaration/macros.rs",
    "zircon_plugins/plugin_sdk/src/native.rs",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileFact {
    byte_length: u64,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceInput {
    path: String,
    fact: FileFact,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    variant: String,
    features: Vec<String>,
    file: PathBuf,
    fact: FileFact,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    task_root: PathBuf,
    target_root: PathBuf,
    build_root: PathBuf,
    source_root: PathBuf,
    expected_variants: Vec<String>,
    source_inputs: Vec<SourceInput>,
    artifacts: Vec<Artifact>,
}

fn ensure(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn admitted(path: &Path, directory: bool) -> Result<ResolvedProjectPathIdentity, String> {
    ensure(path.is_absolute(), "native fixture path must be absolute")?;
    ensure(
        !path.components().any(|part| {
            matches!(
                part,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )
        }),
        "native fixture path must not contain dot components",
    )?;
    for ancestor in path.ancestors() {
        let metadata = std::fs::symlink_metadata(ancestor).map_err(|error| error.to_string())?;
        ensure(
            !metadata.file_type().is_symlink(),
            "native fixture symlink is forbidden",
        )?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            ensure(
                metadata.file_attributes() & 0x400 == 0,
                "native fixture reparse point is forbidden",
            )?;
        }
    }
    let resolved = ProjectPaths::resolve_existing(path).map_err(|error| error.to_string())?;
    let metadata =
        std::fs::metadata(resolved.operation_path()).map_err(|error| error.to_string())?;
    ensure(
        if directory {
            metadata.is_dir()
        } else {
            metadata.is_file()
        },
        "native fixture has the wrong file kind",
    )?;
    let declared = path
        .to_str()
        .ok_or("native fixture path is not Unicode")?
        .replace('\\', "/");
    let physical = resolved
        .display_path()
        .to_str()
        .ok_or("native fixture physical path is not Unicode")?
        .replace('\\', "/");
    ensure(
        declared.eq_ignore_ascii_case(&physical),
        "native fixture path must spell its physical location without aliases",
    )?;
    ProjectPaths::resolve_identity(resolved.operation_path()).map_err(|error| error.to_string())
}

fn env_path(name: &str) -> Result<PathBuf, String> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing {name}"))
}

fn validate_file(path: &Path, fact: &FileFact) -> Result<PathBuf, String> {
    ensure(
        fact.byte_length != 0
            && fact.sha256.len() == 64
            && fact.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "native fixture file fact is malformed",
    )?;
    let identity = admitted(path, false)?;
    let file = File::open(identity.operation_path()).map_err(|error| error.to_string())?;
    let actual = NativePluginArtifactDigest::capture_file(&file, identity.operation_path())
        .map_err(|error| error.to_string())?;
    ensure(
        actual.byte_length == fact.byte_length && actual.sha256 == fact.sha256,
        "native fixture bytes or SHA256 drifted",
    )?;
    ensure(
        admitted(path, false)? == identity,
        "native fixture identity changed during validation",
    )?;
    Ok(identity.operation_path().to_owned())
}

pub(super) fn variant_for(features: &[&str]) -> Result<&'static str, String> {
    match features {
        [] => Ok("dist"),
        ["abi_unknown_version"] => Ok("abi_unknown_version"),
        ["descriptor_export_missing"] => Ok("descriptor_export_missing"),
        ["runtime_entry_export_missing"] => Ok("runtime_entry_export_missing"),
        ["required_capability_missing"] => Ok("required_capability_missing"),
        _ => Err("unknown or combined native fixture feature variant".to_owned()),
    }
}

fn variant_features(variant: &str) -> Result<Vec<String>, String> {
    let features = match variant {
        "dist" => vec!["dist"],
        "abi_unknown_version" => vec!["dist", "abi_unknown_version"],
        "descriptor_export_missing" => vec!["dist", "descriptor_export_missing"],
        "runtime_entry_export_missing" => vec!["dist", "runtime_entry_export_missing"],
        "required_capability_missing" => vec!["dist", "required_capability_missing"],
        _ => return Err("unknown native fixture manifest variant".to_owned()),
    };
    Ok(features.into_iter().map(str::to_owned).collect())
}

fn read_manifest() -> Result<(Manifest, PathBuf), String> {
    use std::io::Read;
    let target_path = env_path("CARGO_TARGET_DIR")?;
    let build_path = env_path("CARGO_BUILD_BUILD_DIR")?;
    let target = admitted(&target_path, true)?;
    let build = admitted(&build_path, true)?;
    ensure(
        target_path.file_name() == Some(std::ffi::OsStr::new("target"))
            && build_path.file_name() == Some(std::ffi::OsStr::new("build")),
        "native fixture requires registered target/build leaf names",
    )?;
    let task_path = target_path.parent().ok_or("target has no task parent")?;
    let task = admitted(task_path, true)?;
    ensure(
        admitted(build_path.parent().ok_or("build has no task parent")?, true)? == task,
        "native fixture build and target must share the same task",
    )?;
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        let mut components = task_path.components();
        let drive = match components.next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(letter) => letter.to_ascii_uppercase(),
                _ => return Err("native fixture requires an ordinary drive-root path".to_owned()),
            },
            _ => return Err("native fixture requires a physical drive root".to_owned()),
        };
        ensure(
            matches!(drive, b'D' | b'E' | b'F'),
            "native fixture compiler outputs require drive D, E or F",
        )?;
        ensure(
            matches!(components.next(), Some(Component::RootDir)),
            "native fixture drive-relative output is forbidden",
        )?;
        let managed_root = match components.next() {
            Some(Component::Normal(name)) => name.to_str(),
            _ => None,
        };
        ensure(
            managed_root.is_some_and(|name| name.eq_ignore_ascii_case("cargo-targets"))
                && components.next().is_some(),
            "native fixture task must be below the physical drive-root cargo-targets directory",
        )?;
    }
    let root_path = env_path("ZIRCON_NATIVE_FIXTURE_ROOT")?;
    let root = admitted(&root_path, true)?;
    let fixtures = admitted(&build_path.join("fixtures"), true)?;
    ensure(
        admitted(
            root_path.parent().ok_or("fixture root has no parent")?,
            true,
        )? == fixtures
            && root.is_within(&build),
        "native fixture root must be an immediate build/fixtures child",
    )?;
    let manifest_path = env_path("ZIRCON_NATIVE_FIXTURE_MANIFEST")?;
    ensure(
        manifest_path == root_path.join("native-fixtures.json"),
        "native fixture manifest must be the explicitly registered root file",
    )?;
    let identity = admitted(&manifest_path, false)?;
    let mut bytes = Vec::new();
    File::open(identity.operation_path())
        .map_err(|error| error.to_string())?
        .take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    ensure(
        !bytes.is_empty() && bytes.len() <= 65_536,
        "native fixture manifest size is invalid",
    )?;
    let expected = std::env::var("ZIRCON_NATIVE_FIXTURE_MANIFEST_SHA256")
        .map_err(|_| "missing registered native fixture manifest SHA256")?;
    ensure(
        format!("{:x}", Sha256::digest(&bytes)) == expected,
        "native fixture manifest differs from the registered SHA256",
    )?;
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    ensure(
        manifest.schema == 1,
        "unsupported native fixture manifest schema",
    )?;
    ensure(
        admitted(&manifest.task_root, true)? == task
            && admitted(&manifest.target_root, true)? == target
            && admitted(&manifest.build_root, true)? == build,
        "native fixture manifest belongs to another task/output root",
    )?;
    let source = admitted(&super::repo_root(), true)?;
    ensure(
        !target.is_within(&source) && !build.is_within(&source),
        "native fixture compiler outputs must stay outside the checkout",
    )?;
    ensure(
        admitted(&manifest.source_root, true)? == source,
        "native fixture was not built from this actual checkout",
    )?;
    let mut inputs = BTreeSet::new();
    for input in &manifest.source_inputs {
        ensure(
            SOURCE_INPUTS.contains(&input.path.as_str()) && inputs.insert(input.path.as_str()),
            "native fixture has an unknown or repeated source input",
        )?;
        validate_file(&manifest.source_root.join(&input.path), &input.fact)?;
    }
    ensure(
        inputs.len() == SOURCE_INPUTS.len(),
        "native fixture source input inventory is incomplete",
    )?;
    let expected: BTreeSet<_> = manifest
        .expected_variants
        .iter()
        .map(String::as_str)
        .collect();
    ensure(
        !expected.is_empty() && expected.len() == manifest.expected_variants.len(),
        "native fixture expected variants are empty or repeated",
    )?;
    let library_name =
        super::platform_library_file_name("zircon_plugin_native_dynamic_fixture_native");
    let mut actual = BTreeSet::new();
    for artifact in &manifest.artifacts {
        ensure(
            expected.contains(artifact.variant.as_str())
                && actual.insert(artifact.variant.as_str()),
            "native fixture artifact variant is unexpected or repeated",
        )?;
        ensure(
            artifact.features == variant_features(&artifact.variant)?,
            "native fixture artifact features do not match its variant",
        )?;
        let relative = PathBuf::from(&artifact.variant).join(&library_name);
        ensure(
            artifact.file == relative,
            "native fixture artifact has an unexpected relative path",
        )?;
        let directory_path = root_path.join(&artifact.variant);
        let directory = admitted(&directory_path, true)?;
        ensure(
            directory.is_within(&root),
            "native fixture variant escaped its root",
        )?;
        validate_file(&root_path.join(&artifact.file), &artifact.fact)?;
        let files = std::fs::read_dir(directory.operation_path())
            .map_err(|error| error.to_string())?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        ensure(
            files == [std::ffi::OsString::from(library_name.as_str())],
            "native fixture variant must contain exactly its registered DLL, no source tree",
        )?;
    }
    ensure(
        actual == expected,
        "native fixture manifest omits an expected artifact",
    )?;
    let mut entries = BTreeSet::new();
    for entry in std::fs::read_dir(root.operation_path()).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        entries.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| "non-Unicode fixture entry")?,
        );
    }
    let mut allowed: BTreeSet<_> = expected.into_iter().map(str::to_owned).collect();
    allowed.insert("native-fixtures.json".to_owned());
    ensure(
        entries == allowed,
        "native fixture root contains unregistered files or source copies",
    )?;
    drop(inputs);
    drop(actual);
    Ok((manifest, root_path))
}

pub(super) fn load(requested_root: &Path, features: &[&str]) -> Result<PathBuf, String> {
    let variant = variant_for(features)?;
    let smoke_path = env_path("ZIRCON_RENDERING_SMOKE_OUTPUT_DIR")?;
    let smoke = admitted(&smoke_path, true)?;
    let build_path = env_path("CARGO_BUILD_BUILD_DIR")?;
    let build = admitted(&build_path, true)?;
    ensure(
        admitted(smoke_path.parent().ok_or("smoke root has no parent")?, true)?
            == admitted(&build_path.join("fixtures"), true)?
            && smoke.is_within(&build),
        "native fixture smoke root must be an immediate registered build/fixtures child",
    )?;
    for name in ["TEMP", "TMP", "TMPDIR"] {
        ensure(
            admitted(&env_path(name)?, true)? == smoke,
            "native fixture temp environment differs from its registered smoke root",
        )?;
    }
    ensure(
        admitted(
            requested_root
                .parent()
                .ok_or("fixture request has no parent")?,
            true,
        )? == smoke
            && requested_root
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("zircon-native-dynamic-")),
        "native fixture request is outside the registered task temp directory",
    )?;
    let (manifest, root) = read_manifest()?;
    let artifact = manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.variant == variant)
        .ok_or_else(|| {
            format!("native fixture variant {variant} was not prebuilt and registered")
        })?;
    validate_file(&root.join(&artifact.file), &artifact.fact)
}

pub(super) fn validate_contract() -> Result<usize, String> {
    let (manifest, _) = read_manifest()?;
    Ok(manifest.artifacts.len())
}
