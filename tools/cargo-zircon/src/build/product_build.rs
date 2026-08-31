use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod batch;
mod build_set;
mod capture;
mod cargo_protocol;
mod environment;

#[cfg(test)]
mod performance_tests;
#[cfg(test)]
mod tests;

pub use batch::{
    build_product_receipt_draft_batch, ProductBuildBatchRequest, ProductBuildDraftBatch,
    VerifiedProductBuildDraftBatchHandoff,
};

use capture::{
    capture_declared_artifact, capture_declared_artifacts, capture_symbol_artifacts,
    open_cargo_product, open_cargo_runtime_dependency, open_declared_artifact,
    PreparedProductBuildToolchain,
};

use super::receipt::{
    ArtifactKind, BuildAction, ProducerIdentity, ProductReceiptDraft, ProductReceiptError,
    ReceiptArtifact, ReceiptArtifactSource, TargetProfile,
};

const PRODUCT_BUILD_REQUEST_SCHEMA_VERSION: u32 = 1;
const CARGO_METADATA_OUTPUT_LIMIT: usize = 64 * 1024 * 1024;
const CARGO_OUTPUT_INITIAL_CAPACITY: usize = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBuildToolchain {
    pub cargo_path: PathBuf,
    pub rustc_path: PathBuf,
    pub linker_path: Option<PathBuf>,
    pub sdk_files: Vec<ProductBuildSdkSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBuildSdkSource {
    pub logical_name: String,
    pub source_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBuildTarget {
    pub target_triple: String,
    pub cargo_profile: String,
    pub rustflags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductArtifactDeclaration {
    pub logical_name: String,
    pub relative_path: String,
    pub symbol_relative_directory: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBuildProducer {
    pub worker_id: String,
    pub operation_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CargoRuntimeDependencyDeclaration {
    pub logical_name: String,
    pub relative_path: String,
    pub package: String,
    pub target: String,
    pub artifact_file_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBuildRequest {
    pub schema_version: u32,
    pub build_set_manifest_path: PathBuf,
    pub manifest_path: String,
    pub target_directory: PathBuf,
    pub toolchain: ProductBuildToolchain,
    pub target: ProductBuildTarget,
    pub action: BuildAction,
    pub producer: ProductBuildProducer,
    pub product: ProductArtifactDeclaration,
    pub environment_policy: String,
    pub runtime_dependencies: Vec<CargoRuntimeDependencyDeclaration>,
    pub sbom: Option<ReceiptArtifactSource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoProductArtifact {
    pub executable: PathBuf,
    pub symbol_files: Vec<PathBuf>,
}

pub(super) struct CargoRuntimeArtifact<'a> {
    pub(super) declaration: &'a CargoRuntimeDependencyDeclaration,
    pub(super) source_path: PathBuf,
}

pub fn build_product_receipt_draft(
    mut request: ProductBuildRequest,
) -> Result<ProductReceiptDraft, ProductReceiptError> {
    validate_build_request(&mut request)?;
    let build_set = build_set::ValidatedBuildSet::open(&request.build_set_manifest_path)?;
    let mut prepared_toolchain = PreparedProductBuildToolchain::open(&mut request.toolchain)?;
    build_product_receipt_draft_in_build_set(request, &build_set, &mut prepared_toolchain)
}

pub(super) fn build_product_receipt_draft_in_build_set(
    mut request: ProductBuildRequest,
    build_set: &build_set::ValidatedBuildSet,
    prepared_toolchain: &mut PreparedProductBuildToolchain,
) -> Result<ProductReceiptDraft, ProductReceiptError> {
    let snapshot_root = build_set.snapshot_root.as_path();
    let manifest_path =
        resolve_snapshot_file(&snapshot_root, &request.manifest_path, "Cargo manifest")?;
    let (target_directory, _target_directory_lease) =
        create_owned_target_directory(&request.target_directory, &snapshot_root)?;

    let (build_environment, environment_digest) = environment::effective_build_environment(
        &request.environment_policy,
        prepared_toolchain.rustc_path(),
        prepared_toolchain.linker_path(),
        &target_directory,
        &request.target,
    )?;
    let metadata_arguments = metadata_arguments(&request, &manifest_path);
    let metadata_bytes = run_bounded_cargo_output(
        prepared_toolchain.cargo_path(),
        &snapshot_root,
        &metadata_arguments,
        &build_environment,
        CARGO_METADATA_OUTPUT_LIMIT,
        "Cargo metadata",
    )?;
    let binary = request.action.bin.as_deref().ok_or_else(|| {
        ProductReceiptError::new("product build action must select one binary target")
    })?;
    let resolution = cargo_protocol::resolve_build(
        &metadata_bytes,
        &snapshot_root,
        &request.action.package,
        binary,
        std::mem::take(&mut request.runtime_dependencies),
    )?;

    let build_arguments = build_arguments(&request, &manifest_path, &target_directory, binary);
    let mut child = spawn_cargo(
        prepared_toolchain.cargo_path(),
        &snapshot_root,
        &build_arguments,
        &build_environment,
        "Cargo product build",
    )?;
    let stdout = child.stdout.take().ok_or_else(|| {
        ProductReceiptError::new("Cargo product build stdout pipe was not available")
    })?;
    let opened_artifacts_result = cargo_protocol::select_build_artifacts(
        BufReader::new(stdout),
        &resolution,
        |artifact| open_cargo_product(artifact, &target_directory),
        |artifact| open_cargo_runtime_dependency(artifact, &target_directory),
    );
    let (opened_product, opened_runtime_dependencies) = match opened_artifacts_result {
        Ok(artifacts) => artifacts,
        Err(error) => {
            terminate_child(&mut child);
            return Err(error);
        }
    };
    let cargo_graph_digest = resolution.cargo_graph_digest;
    let status = child.wait().map_err(|error| {
        ProductReceiptError::new(format!("could not wait for Cargo product build: {error}"))
    })?;
    if !status.success() {
        return Err(ProductReceiptError::new(format!(
            "Cargo product build exited with status {status}"
        )));
    }
    build_set.verify_inventory()?;

    let opened_sbom = request.sbom.map(open_declared_artifact).transpose()?;
    let toolchain = prepared_toolchain.receipt_toolchain(environment_digest)?;
    let digest_buffer = prepared_toolchain.digest_buffer();
    let codegen_flags_digest = sha256_serialized(&request.target.rustflags)?;
    let symbols = capture_symbol_artifacts(
        opened_product.symbols,
        &request.product.logical_name,
        &request.product.symbol_relative_directory,
        digest_buffer,
    )?;
    let build_product = ReceiptArtifact::capture_from_file_with_buffer(
        request.product.logical_name,
        request.product.relative_path,
        ArtifactKind::Executable,
        opened_product.executable,
        digest_buffer,
    )?;

    Ok(ProductReceiptDraft {
        build_set_id: build_set.build_set_id.clone(),
        toolchain,
        target_profile: TargetProfile {
            target_triple: request.target.target_triple,
            cargo_profile: request.target.cargo_profile,
            codegen_flags_digest,
            cargo_graph_digest,
        },
        action: request.action,
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            worker_id: request.producer.worker_id,
            operation_id: request.producer.operation_id,
        },
        build_products: vec![build_product],
        runtime_dependencies: capture_declared_artifacts(
            opened_runtime_dependencies,
            digest_buffer,
        )?,
        symbols,
        sbom: opened_sbom
            .map(|source| capture_declared_artifact(source, digest_buffer))
            .transpose()?,
    })
}

pub fn select_cargo_product_artifact(
    messages: impl BufRead,
    expected_package_id: &str,
    expected_binary: &str,
) -> Result<CargoProductArtifact, ProductReceiptError> {
    cargo_protocol::select_product_artifact(messages, expected_package_id, expected_binary)
}

pub(super) fn validate_build_request(
    request: &mut ProductBuildRequest,
) -> Result<(), ProductReceiptError> {
    if request.schema_version != PRODUCT_BUILD_REQUEST_SCHEMA_VERSION {
        return Err(ProductReceiptError::new(format!(
            "unsupported product build request schema version {}",
            request.schema_version
        )));
    }
    validate_required("build package", &request.action.package)?;
    validate_required("build target triple", &request.target.target_triple)?;
    validate_required("Cargo profile", &request.target.cargo_profile)?;
    validate_required("product logical name", &request.product.logical_name)?;
    validate_required("producer worker id", &request.producer.worker_id)?;
    validate_required("producer operation id", &request.producer.operation_id)?;
    validate_relative_path(
        "product receipt path",
        &request.product.relative_path,
        false,
    )?;
    validate_relative_path(
        "product symbol directory",
        &request.product.symbol_relative_directory,
        true,
    )?;
    let binary = request.action.bin.as_ref().ok_or_else(|| {
        ProductReceiptError::new("product build action must select one binary target")
    })?;
    validate_required("build binary", binary)?;

    if request.toolchain.sdk_files.is_empty() {
        return Err(ProductReceiptError::new(
            "product build toolchain must declare at least one SDK or CRT file",
        ));
    }
    request
        .toolchain
        .sdk_files
        .sort_unstable_by(|left, right| left.logical_name.cmp(&right.logical_name));
    let mut previous: Option<&str> = None;
    for sdk_file in &request.toolchain.sdk_files {
        validate_stable_name("SDK file logical name", &sdk_file.logical_name)?;
        if previous == Some(sdk_file.logical_name.as_str()) {
            return Err(ProductReceiptError::new(format!(
                "product build toolchain contains duplicate SDK file `{}`",
                sdk_file.logical_name
            )));
        }
        previous = Some(sdk_file.logical_name.as_str());
    }

    request
        .runtime_dependencies
        .sort_unstable_by(|left, right| left.logical_name.cmp(&right.logical_name));
    let mut previous: Option<&str> = None;
    for dependency in &request.runtime_dependencies {
        validate_stable_name("runtime dependency logical name", &dependency.logical_name)?;
        validate_relative_path(
            "runtime dependency receipt path",
            &dependency.relative_path,
            false,
        )?;
        validate_required("runtime dependency package", &dependency.package)?;
        validate_required("runtime dependency target", &dependency.target)?;
        validate_artifact_file_name(&dependency.artifact_file_name)?;
        if previous == Some(dependency.logical_name.as_str()) {
            return Err(ProductReceiptError::new(format!(
                "product build request contains duplicate runtime dependency `{}`",
                dependency.logical_name
            )));
        }
        previous = Some(dependency.logical_name.as_str());
    }

    request.action.features.sort_unstable();
    let mut previous: Option<&str> = None;
    for feature in &request.action.features {
        validate_required("build feature", feature)?;
        if previous == Some(feature.as_str()) {
            return Err(ProductReceiptError::new(format!(
                "product build request contains duplicate feature `{feature}`"
            )));
        }
        previous = Some(feature.as_str());
    }
    Ok(())
}

fn validate_required(label: &str, value: &str) -> Result<(), ProductReceiptError> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(ProductReceiptError::new(format!(
            "product build {label} must be non-empty text without control characters"
        )));
    }
    Ok(())
}

fn validate_stable_name(label: &str, value: &str) -> Result<(), ProductReceiptError> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_.".contains(&byte)
        })
        || !value.as_bytes()[0].is_ascii_lowercase()
    {
        return Err(ProductReceiptError::new(format!(
            "product build {label} must be a stable lowercase identifier"
        )));
    }
    Ok(())
}

fn validate_artifact_file_name(value: &str) -> Result<(), ProductReceiptError> {
    validate_required("runtime dependency artifact file name", value)?;
    if value.contains('/')
        || value.contains('\\')
        || Path::new(value).file_name() != Some(value.as_ref())
    {
        return Err(ProductReceiptError::new(
            "product build runtime dependency artifact file name must be one file name",
        ));
    }
    Ok(())
}

fn validate_relative_path(
    label: &str,
    value: &str,
    directory: bool,
) -> Result<(), ProductReceiptError> {
    if value.is_empty()
        || value.contains('\\')
        || value.starts_with('/')
        || value.ends_with('/')
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || (!directory && value.ends_with("/."))
    {
        return Err(ProductReceiptError::new(format!(
            "product build {label} must be a normalized relative path"
        )));
    }
    Ok(())
}

fn canonical_file(path: &Path, label: &str) -> Result<PathBuf, ProductReceiptError> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not resolve {label} `{}`: {error}",
            path.display()
        ))
    })?;
    if !canonical.is_file() {
        return Err(ProductReceiptError::new(format!(
            "{label} `{}` is not a regular file",
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn create_owned_target_directory(
    requested: &Path,
    snapshot_root: &Path,
) -> Result<(PathBuf, File), ProductReceiptError> {
    let target_name = requested.file_name().ok_or_else(|| {
        ProductReceiptError::new("Cargo target directory must have a final path component")
    })?;
    if !requested.is_absolute()
        || requested
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(ProductReceiptError::new(
            "Cargo target directory must be an absolute normalized path",
        ));
    }
    let parent = requested.parent().ok_or_else(|| {
        ProductReceiptError::new("Cargo target directory must have an existing parent")
    })?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not resolve Cargo target parent `{}`: {error}",
            parent.display()
        ))
    })?;
    if !canonical_parent.is_dir() {
        return Err(ProductReceiptError::new(
            "Cargo target parent must be a directory",
        ));
    }
    let target = canonical_parent.join(target_name);
    if target.starts_with(snapshot_root) || snapshot_root.starts_with(&target) {
        return Err(ProductReceiptError::new(
            "Cargo target directory must not overlap the immutable BuildSet snapshot",
        ));
    }
    match fs::symlink_metadata(&target) {
        Ok(_) => {
            return Err(ProductReceiptError::new(format!(
                "Cargo target directory must not already exist: {}",
                target.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(ProductReceiptError::new(format!(
                "could not inspect Cargo target directory `{}`: {error}",
                target.display()
            )));
        }
    }
    fs::create_dir(&target).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not create owned Cargo target directory `{}`: {error}",
            target.display()
        ))
    })?;
    let lease = match open_directory_lease(&target) {
        Ok(lease) => lease,
        Err(error) => {
            let _ = fs::remove_dir(&target);
            return Err(error);
        }
    };
    let canonical = fs::canonicalize(&target).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not resolve owned Cargo target directory `{}`: {error}",
            target.display()
        ))
    })?;
    Ok((canonical, lease))
}

fn open_directory_lease(path: &Path) -> Result<File, ProductReceiptError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const FILE_SHARE_WRITE: u32 = 0x0000_0002;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

        options
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let lease = options.open(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not lease owned Cargo target directory `{}`: {error}",
            path.display()
        ))
    })?;
    let metadata = lease.metadata().map_err(|error| {
        ProductReceiptError::new(format!(
            "could not inspect owned Cargo target directory `{}`: {error}",
            path.display()
        ))
    })?;
    if !metadata.is_dir() {
        return Err(ProductReceiptError::new(
            "owned Cargo target path is not a directory",
        ));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        if metadata.file_attributes() & 0x0000_0400 != 0 {
            return Err(ProductReceiptError::new(
                "owned Cargo target directory must not be a reparse point",
            ));
        }
    }
    Ok(lease)
}

fn resolve_snapshot_file(
    snapshot_root: &Path,
    relative_path: &str,
    label: &str,
) -> Result<PathBuf, ProductReceiptError> {
    let relative = Path::new(relative_path);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ProductReceiptError::new(format!(
            "{label} path must be normalized and relative to the BuildSet snapshot"
        )));
    }
    let canonical = canonical_file(&snapshot_root.join(relative), label)?;
    if !canonical.starts_with(snapshot_root) {
        return Err(ProductReceiptError::new(format!(
            "{label} resolved outside the BuildSet snapshot"
        )));
    }
    Ok(canonical)
}

fn metadata_arguments<'a>(
    request: &'a ProductBuildRequest,
    manifest_path: &'a Path,
) -> Vec<Cow<'a, OsStr>> {
    let base_arguments = [
        Cow::Borrowed(OsStr::new("metadata")),
        Cow::Borrowed(OsStr::new("--format-version")),
        Cow::Borrowed(OsStr::new("1")),
        Cow::Borrowed(OsStr::new("--manifest-path")),
        Cow::Borrowed(manifest_path.as_os_str()),
        Cow::Borrowed(OsStr::new("--frozen")),
        Cow::Borrowed(OsStr::new("--filter-platform")),
        Cow::Borrowed(OsStr::new(request.target.target_triple.as_str())),
    ];
    let mut arguments =
        Vec::with_capacity(base_arguments.len() + feature_argument_count(&request.action.features));
    arguments.extend(base_arguments);
    push_features(&mut arguments, &request.action.features);
    arguments
}

fn build_arguments<'a>(
    request: &'a ProductBuildRequest,
    manifest_path: &'a Path,
    target_directory: &'a Path,
    binary: &'a str,
) -> Vec<Cow<'a, OsStr>> {
    let base_arguments = [
        Cow::Borrowed(OsStr::new("build")),
        Cow::Borrowed(OsStr::new("--manifest-path")),
        Cow::Borrowed(manifest_path.as_os_str()),
        Cow::Borrowed(OsStr::new("--package")),
        Cow::Borrowed(OsStr::new(request.action.package.as_str())),
        Cow::Borrowed(OsStr::new("--bin")),
        Cow::Borrowed(OsStr::new(binary)),
        Cow::Borrowed(OsStr::new("--frozen")),
        Cow::Borrowed(OsStr::new("--target")),
        Cow::Borrowed(OsStr::new(request.target.target_triple.as_str())),
        Cow::Borrowed(OsStr::new("--profile")),
        Cow::Borrowed(OsStr::new(request.target.cargo_profile.as_str())),
        Cow::Borrowed(OsStr::new("--target-dir")),
        Cow::Borrowed(target_directory.as_os_str()),
        Cow::Borrowed(OsStr::new("--message-format=json-render-diagnostics")),
    ];
    let mut arguments =
        Vec::with_capacity(base_arguments.len() + feature_argument_count(&request.action.features));
    arguments.extend(base_arguments);
    push_features(&mut arguments, &request.action.features);
    arguments
}

fn feature_argument_count(features: &[String]) -> usize {
    if features.is_empty() {
        0
    } else {
        2
    }
}

fn push_features<'a>(arguments: &mut Vec<Cow<'a, OsStr>>, features: &[String]) {
    if !features.is_empty() {
        arguments.push(Cow::Borrowed(OsStr::new("--features")));
        arguments.push(Cow::Owned(features.join(",").into()));
    }
}

fn run_bounded_cargo_output(
    cargo_path: &Path,
    current_directory: &Path,
    arguments: &[Cow<'_, OsStr>],
    environment: &[(String, String)],
    limit: usize,
    label: &str,
) -> Result<Vec<u8>, ProductReceiptError> {
    let mut child = spawn_cargo(cargo_path, current_directory, arguments, environment, label)?;
    let stdout = child.stdout.take().ok_or_else(|| {
        ProductReceiptError::new(format!("{label} stdout pipe was not available"))
    })?;
    let mut output = bounded_output_buffer(limit);
    let read_result = stdout
        .take(limit as u64 + 1)
        .read_to_end(&mut output)
        .map_err(|error| {
            ProductReceiptError::new(format!("could not read {label} output: {error}"))
        });
    if let Err(error) = read_result {
        terminate_child(&mut child);
        return Err(error);
    }
    if output.len() > limit {
        terminate_child(&mut child);
        return Err(ProductReceiptError::new(format!(
            "{label} output exceeded the {limit}-byte limit"
        )));
    }
    let status = child.wait().map_err(|error| {
        ProductReceiptError::new(format!("could not wait for {label}: {error}"))
    })?;
    if !status.success() {
        return Err(ProductReceiptError::new(format!(
            "{label} exited with status {status}"
        )));
    }
    Ok(output)
}

fn bounded_output_buffer(limit: usize) -> Vec<u8> {
    Vec::with_capacity(limit.min(CARGO_OUTPUT_INITIAL_CAPACITY))
}

fn spawn_cargo(
    cargo_path: &Path,
    current_directory: &Path,
    arguments: &[Cow<'_, OsStr>],
    environment: &[(String, String)],
    label: &str,
) -> Result<Child, ProductReceiptError> {
    let mut command = Command::new(cargo_path);
    command
        .current_dir(current_directory)
        .args(arguments)
        .env_clear()
        .envs(environment.iter().map(|(name, value)| (name, value)))
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    command
        .spawn()
        .map_err(|error| ProductReceiptError::new(format!("could not start {label}: {error}")))
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn open_locked_source(path: &Path, label: &str) -> Result<File, ProductReceiptError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        options.share_mode(0x0000_0001);
    }
    options.open(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not open product build {label} `{}`: {error}",
            path.display()
        ))
    })
}

pub(super) fn sha256_serialized(payload: &impl Serialize) -> Result<String, ProductReceiptError> {
    let mut hasher = Sha256::new();
    serde_json::to_writer(&mut hasher, payload).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not serialize product build identity: {error}"
        ))
    })?;
    Ok(hex_digest(hasher.finalize().as_slice()))
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[usize::from(*byte >> 4)] as char);
        encoded.push(HEX[usize::from(*byte & 0x0f)] as char);
    }
    encoded
}
