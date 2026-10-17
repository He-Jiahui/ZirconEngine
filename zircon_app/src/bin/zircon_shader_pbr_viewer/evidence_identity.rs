use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};

pub(crate) const READY_FRAME_EVIDENCE_IDENTITY_SCHEMA: &str =
    "zircon_shader_pbr_viewer_evidence_identity_v1";
pub(crate) const READY_FRAME_EVIDENCE_VALIDATION_POLICY: &str =
    "zircon_shader_pbr_viewer_ready_frame_v19";
const INDEPENDENT_COMMAND_RECEIPT_SCHEMA: &str = "zircon_local_cargo_command_evidence_v1";
const INDEPENDENT_COMMAND_ORIGIN: &str = "independent-local-command";
const INDEPENDENT_COMMAND_STATUS: &str = "completed-selected-input-command";
const SELECTED_SOURCE_PROTOCOL: &str = "zircon-local-selected-source-map-v1";
const SELECTED_SOURCE_SCOPE: &str = "requested-selected-files";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EvidenceFileFingerprint {
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) byte_length: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReadyFrameEvidenceIdentity {
    pub(crate) identity_manifest: EvidenceFileFingerprint,
    pub(crate) run_id: String,
    pub(crate) validation_policy: String,
    pub(crate) source_manifest_sha256: String,
    pub(crate) viewer_binary: EvidenceFileFingerprint,
    pub(crate) hdri: EvidenceFileFingerprint,
    pub(crate) build_provenance: EvidenceFileFingerprint,
}

#[derive(Deserialize)]
struct EvidenceIdentityManifest {
    schema: String,
    run_id: String,
    validation_policy: String,
    source_manifest_sha256: String,
    viewer_binary: ManifestFileFingerprint,
    hdri: ManifestFileFingerprint,
    build_provenance: ManifestFileFingerprint,
}

#[derive(Deserialize)]
struct ManifestFileFingerprint {
    path: PathBuf,
    sha256: String,
    byte_length: u64,
}

#[derive(Debug)]
enum UniqueJson {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Self>),
    Object(BTreeMap<String, Self>),
}

impl UniqueJson {
    fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(value),
            Self::Number(value) => Value::Number(value),
            Self::String(value) => Value::String(value),
            Self::Array(values) => Value::Array(values.into_iter().map(Self::into_value).collect()),
            Self::Object(values) => Value::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, value.into_value()))
                    .collect::<Map<_, _>>(),
            ),
        }
    }
}

impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct JsonVisitor;

        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = UniqueJson;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON value with unique object keys")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(UniqueJson::Null)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(UniqueJson::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(UniqueJson::Number(Number::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(UniqueJson::Number(Number::from(value)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Number::from_f64(value)
                    .map(UniqueJson::Number)
                    .ok_or_else(|| E::custom("JSON number is not finite"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(UniqueJson::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(UniqueJson::String(value))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(UniqueJson::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(UniqueJson::Array(values))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!(
                            "duplicate JSON object field: {}",
                            key
                        )));
                    }
                    values.insert(key, map.next_value()?);
                }
                Ok(UniqueJson::Object(values))
            }
        }

        deserializer.deserialize_any(JsonVisitor)
    }
}

fn parse_unique_json<T: DeserializeOwned>(contents: &str, description: &str) -> Result<T, String> {
    let mut deserializer =
        serde_json::Deserializer::from_str(contents.trim_start_matches('\u{feff}'));
    let value = UniqueJson::deserialize(&mut deserializer)
        .map_err(|error| format!("parse {description}: {error}"))?
        .into_value();
    deserializer
        .end()
        .map_err(|error| format!("parse trailing data in {description}: {error}"))?;
    serde_json::from_value(value).map_err(|error| format!("decode {description}: {error}"))
}

pub(crate) fn load_ready_frame_evidence_identity(
    identity_path: &Path,
    expected_hdri_path: &Path,
) -> Result<ReadyFrameEvidenceIdentity, String> {
    let identity_manifest = fingerprint_file(identity_path, "Ready-frame evidence identity")?;
    let contents = fs::read_to_string(&identity_manifest.path).map_err(|error| {
        format!(
            "read Ready-frame evidence identity {}: {error}",
            identity_manifest.path.display()
        )
    })?;
    let manifest: EvidenceIdentityManifest = parse_unique_json(
        &contents,
        &format!(
            "Ready-frame evidence identity {}",
            identity_manifest.path.display()
        ),
    )?;

    if manifest.schema != READY_FRAME_EVIDENCE_IDENTITY_SCHEMA {
        return Err(format!(
            "Ready-frame evidence identity has unsupported schema {}: {}",
            manifest.schema,
            identity_manifest.path.display()
        ));
    }
    if manifest.validation_policy != READY_FRAME_EVIDENCE_VALIDATION_POLICY {
        return Err(format!(
            "Ready-frame evidence identity has unsupported validation policy {}: {}",
            manifest.validation_policy,
            identity_manifest.path.display()
        ));
    }
    if !is_safe_run_id(&manifest.run_id) {
        return Err(format!(
            "Ready-frame evidence identity run_id must use lowercase ASCII letters, digits, or hyphens: {}",
            identity_manifest.path.display()
        ));
    }
    if !is_sha256(&manifest.source_manifest_sha256) {
        return Err(format!(
            "Ready-frame evidence identity source_manifest_sha256 is invalid: {}",
            identity_manifest.path.display()
        ));
    }

    let identity_directory = identity_manifest.path.parent().ok_or_else(|| {
        format!(
            "Ready-frame evidence identity has no parent directory: {}",
            identity_manifest.path.display()
        )
    })?;
    let viewer_binary = verify_manifest_file(
        &manifest.viewer_binary,
        &std::env::current_exe().map_err(|error| {
            format!("resolve current viewer executable for evidence identity: {error}")
        })?,
        identity_directory,
        "viewer binary",
    )?;
    let hdri = verify_manifest_file(
        &manifest.hdri,
        expected_hdri_path,
        identity_directory,
        "HDRI input",
    )?;
    let build_provenance = verify_manifest_file(
        &manifest.build_provenance,
        &resolve_manifest_path(identity_directory, &manifest.build_provenance.path),
        identity_directory,
        "managed build provenance",
    )?;
    validate_independent_build_provenance(
        &build_provenance.path,
        &viewer_binary,
        &manifest.source_manifest_sha256,
    )?;

    Ok(ReadyFrameEvidenceIdentity {
        identity_manifest,
        run_id: manifest.run_id,
        validation_policy: manifest.validation_policy,
        source_manifest_sha256: manifest.source_manifest_sha256,
        viewer_binary,
        hdri,
        build_provenance,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectedSourceBinding {
    manifest: BTreeMap<String, String>,
    sha256: String,
    protocol: String,
    scope: String,
}

/// Parse and bind the independent local-command provenance carried by the
/// v1 identity wrapper. This is deliberately private: it proves transport
/// identity and preserves the false admission flags, but it never authorizes a
/// viewer capture.
fn validate_independent_build_provenance(
    provenance_path: &Path,
    expected_binary: &EvidenceFileFingerprint,
    expected_source_manifest_sha256: &str,
) -> Result<(), String> {
    let contents = fs::read_to_string(provenance_path).map_err(|error| {
        format!(
            "read independent Shader PBR build provenance {}: {error}",
            provenance_path.display()
        )
    })?;
    let document: Value = parse_unique_json(
        &contents,
        &format!(
            "independent Shader PBR build provenance {}",
            provenance_path.display()
        ),
    )?;
    let object = json_object(&document, "build provenance")?;
    let schema_version = json_u64(object, "schema_version", "build provenance")?;
    if schema_version == 2 {
        return Err(format!(
            "schema2 Shader PBR build provenance is historical and cannot enter the current identity loader: {}",
            provenance_path.display()
        ));
    }
    if schema_version != 3
        || json_string(object, "provenance_kind", "build provenance")?
            != "zircon_independent_command_artifact_provenance"
    {
        return Err(format!(
            "independent Shader PBR build provenance has unsupported schema or kind: {}",
            provenance_path.display()
        ));
    }
    if json_string(object, "provenance_scope", "build provenance")? != "identity-transport"
        || json_bool(
            object,
            "compiler_input_closure_verified",
            "build provenance",
        )?
        || json_bool(object, "capture_admission", "build provenance")?
    {
        return Err(format!(
            "independent Shader PBR build provenance is not a non-admitting identity transport: {}",
            provenance_path.display()
        ));
    }
    let source_manifest_sha256 = json_string(object, "source_manifest_sha256", "build provenance")?;
    if !is_sha256(source_manifest_sha256)
        || source_manifest_sha256 != expected_source_manifest_sha256
    {
        return Err(format!(
            "independent Shader PBR build provenance source manifest hash does not match the identity wrapper: {}",
            provenance_path.display()
        ));
    }
    let source_manifest_protocol =
        json_string(object, "source_manifest_protocol", "build provenance")?;
    let source_scope = json_string(object, "source_scope", "build provenance")?;
    if source_manifest_protocol != SELECTED_SOURCE_PROTOCOL || source_scope != SELECTED_SOURCE_SCOPE
    {
        return Err(format!(
            "independent Shader PBR build provenance has unsupported source protocol or scope: {}",
            provenance_path.display()
        ));
    }

    let binary_value = json_field(object, "binary", "build provenance")?;
    let _binary = json_object(binary_value, "binary")?;
    verify_json_file_fingerprint(binary_value, expected_binary, "build provenance binary")?;

    let repository = json_object(
        json_field(object, "repository", "build provenance")?,
        "repository",
    )?;
    let repository_root = canonical_existing_path(
        json_string(repository, "root", "repository")?,
        "build provenance repository root",
    )?;
    let source_manifest = parse_source_manifest(
        json_field(repository, "source_manifest", "repository")?,
        &repository_root,
    )?;

    let execution = json_object(
        json_field(object, "execution_evidence", "build provenance")?,
        "execution_evidence",
    )?;
    if json_string(execution, "origin", "execution evidence")? != INDEPENDENT_COMMAND_ORIGIN {
        return Err(format!(
            "independent Shader PBR execution evidence has an unsupported origin: {}",
            provenance_path.display()
        ));
    }
    let receipt_reference = json_field(execution, "receipt", "execution evidence")?;
    let receipt_path = verify_reference_file_fact(
        receipt_reference,
        provenance_path.parent().unwrap_or_else(|| Path::new(".")),
        None,
        None,
        "independent command receipt",
    )?;
    let receipt_contents = fs::read_to_string(&receipt_path).map_err(|error| {
        format!(
            "read independent command receipt {}: {error}",
            receipt_path.display()
        )
    })?;
    let receipt: Value = parse_unique_json(
        &receipt_contents,
        &format!("independent command receipt {}", receipt_path.display()),
    )?;
    let receipt_binding =
        validate_command_receipt(&receipt_path, &receipt, expected_binary, &repository_root)?;
    if receipt_binding.sha256 != source_manifest_sha256
        || receipt_binding.protocol != source_manifest_protocol
        || receipt_binding.scope != source_scope
        || source_manifest
            .iter()
            .any(|(name, hash)| receipt_binding.manifest.get(name) != Some(hash))
    {
        return Err(format!(
            "independent Shader PBR provenance and command receipt source identity differ: {}",
            provenance_path.display()
        ));
    }
    let receipt_after = fingerprint_file(&receipt_path, "independent command receipt")?;
    let declared_receipt = parse_file_fingerprint(receipt_reference, "execution evidence receipt")?;
    if receipt_after.sha256 != declared_receipt.sha256
        || receipt_after.byte_length != declared_receipt.byte_length
    {
        return Err(format!(
            "independent command receipt changed while binding provenance: {}",
            receipt_path.display()
        ));
    }
    Ok(())
}

fn matches_canonical_path(declared: &str, expected: &Path) -> bool {
    let declared = Path::new(declared);
    declared.is_absolute() && evidence_transport_path(declared) == evidence_transport_path(expected)
}

fn validate_command_receipt(
    receipt_path: &Path,
    receipt: &Value,
    expected_binary: &EvidenceFileFingerprint,
    repository_root: &Path,
) -> Result<SelectedSourceBinding, String> {
    let object = json_object(receipt, "independent command receipt")?;
    if json_string(object, "schema", "independent command receipt")?
        != INDEPENDENT_COMMAND_RECEIPT_SCHEMA
        || json_string(object, "origin", "independent command receipt")?
            != INDEPENDENT_COMMAND_ORIGIN
        || json_string(object, "status", "independent command receipt")?
            != INDEPENDENT_COMMAND_STATUS
        || json_u64(object, "rootExitCode", "independent command receipt")? != 0
    {
        return Err(format!(
            "independent command receipt is not a completed selected-input command: {}",
            receipt_path.display()
        ));
    }
    for field in [
        "compilerInputClosureVerified",
        "captureAdmission",
        "immutableInputSeal",
        "JenkinsAcceptance",
        "milestoneAcceptance",
        "formalAcceptance",
    ] {
        if json_bool(object, field, "independent command receipt")? {
            return Err(format!(
                "independent command receipt attempts to upgrade {field}: {}",
                receipt_path.display()
            ));
        }
    }
    for field in [
        "selectedInputsUnchanged",
        "compilerUnchanged",
        "driversUnchanged",
    ] {
        if !json_bool(object, field, "independent command receipt")? {
            return Err(format!(
                "independent command receipt does not retain {field}: {}",
                receipt_path.display()
            ));
        }
    }
    let source_scope = json_string(object, "sourceScope", "independent command receipt")?;
    let source_manifest_protocol = json_string(
        object,
        "sourceManifestProtocol",
        "independent command receipt",
    )?;
    if source_scope != SELECTED_SOURCE_SCOPE || source_manifest_protocol != SELECTED_SOURCE_PROTOCOL
    {
        return Err(format!(
            "independent command receipt source protocol or scope is unsupported: {}",
            receipt_path.display()
        ));
    }
    let receipt_directory = receipt_path.parent().ok_or_else(|| {
        format!(
            "independent command receipt has no parent: {}",
            receipt_path.display()
        )
    })?;
    let request = read_reference_json(
        receipt_directory,
        json_field(object, "request", "independent command receipt")?,
        "request.json",
    )?;
    let native = read_reference_json(
        receipt_directory,
        json_field(object, "nativeTerminal", "independent command receipt")?,
        "native-terminal.json",
    )?;
    let inputs_after = read_reference_json(
        receipt_directory,
        json_field(object, "inputsAfter", "independent command receipt")?,
        "inputs-after.json",
    )?;
    let cargo_artifacts = read_reference_json(
        receipt_directory,
        json_field(object, "cargoArtifacts", "independent command receipt")?,
        "cargo-artifacts.json",
    )?;
    for name in ["stdout.bin", "stderr.bin"] {
        let field = json_field(
            object,
            name.trim_end_matches(".bin"),
            "independent command receipt",
        )?;
        verify_reference_file_fact(
            field,
            receipt_directory,
            Some(receipt_directory),
            Some(name),
            "independent command output",
        )?;
    }

    let request_object = json_object(&request, "independent command request")?;
    if json_u64(
        request_object,
        "schemaVersion",
        "independent command request",
    )? != 1
        || json_string(request_object, "origin", "independent command request")?
            != INDEPENDENT_COMMAND_ORIGIN
        || !matches_canonical_path(
            json_string(request_object, "cwd", "independent command request")?,
            repository_root,
        )
        || json_bool(
            request_object,
            "immutableInputSeal",
            "independent command request",
        )?
        || json_bool(
            request_object,
            "formalAcceptance",
            "independent command request",
        )?
    {
        return Err(format!(
            "independent command request does not bind its repository or scope: {}",
            receipt_path.display()
        ));
    }
    let command = json_string_array(
        json_field(request_object, "command", "independent command request")?,
        "independent command argv",
    )?;
    if command.len() < 2
        || !matches!(command[1].as_str(), "build" | "test")
        || !command.iter().any(|value| value == "--locked")
        || command
            .iter()
            .filter(|value| *value == "--message-format=json")
            .count()
            != 1
        || command.iter().any(|value| value == "--")
        || (command[1] == "test" && !command.iter().any(|value| value == "--no-run"))
    {
        return Err(format!(
            "independent command request argv is not the admitted Cargo artifact command: {}",
            receipt_path.display()
        ));
    }
    let prepared = json_string_array(
        json_field(
            request_object,
            "literalPreparedCommand",
            "independent command request",
        )?,
        "independent prepared command",
    )?;
    if prepared
        != std::iter::once("cargo".to_owned())
            .chain(command.iter().skip(1).cloned())
            .collect::<Vec<_>>()
    {
        return Err(format!(
            "independent command request prepared argv differs from its literal argv: {}",
            receipt_path.display()
        ));
    }
    let target_directory = canonical_existing_path(
        json_string(
            request_object,
            "targetDirectory",
            "independent command request",
        )?,
        "independent Cargo target directory",
    )?;
    let build_directory = canonical_existing_path(
        json_string(
            request_object,
            "buildDirectory",
            "independent command request",
        )?,
        "independent Cargo build directory",
    )?;
    if !is_allowed_target_directory(&target_directory)
        || build_directory
            != target_directory
                .parent()
                .unwrap_or(Path::new(""))
                .join("build")
    {
        return Err(format!(
            "independent command request uses an unapproved target/build root: {}",
            receipt_path.display()
        ));
    }
    let artifact_output_directories = json_string_array(
        json_field(
            request_object,
            "artifactOutputDirectories",
            "independent command request",
        )?,
        "artifact output directories",
    )?;
    if artifact_output_directories.len() != 2
        || !matches_canonical_path(&artifact_output_directories[0], &target_directory)
        || !matches_canonical_path(&artifact_output_directories[1], &build_directory)
    {
        return Err(format!(
            "independent command request output roots differ from target/build: {}",
            receipt_path.display()
        ));
    }
    let environment = json_object(
        json_field(
            request_object,
            "buildEnvironment",
            "independent command request",
        )?,
        "build environment",
    )?;
    if !matches_canonical_path(
        json_string(environment, "CARGO_TARGET_DIR", "build environment")?,
        &target_directory,
    ) || !matches_canonical_path(
        json_string(environment, "CARGO_BUILD_BUILD_DIR", "build environment")?,
        &build_directory,
    ) {
        return Err(format!(
            "independent command request environment differs from target/build: {}",
            receipt_path.display()
        ));
    }
    let target_flag = command
        .iter()
        .position(|value| value == "--target-dir")
        .ok_or_else(|| {
            format!(
                "independent command request has no --target-dir: {}",
                receipt_path.display()
            )
        })?;
    if !command
        .get(target_flag + 1)
        .is_some_and(|declared| matches_canonical_path(declared, &target_directory))
    {
        return Err(format!(
            "independent command request --target-dir differs from targetDirectory: {}",
            receipt_path.display()
        ));
    }
    validate_compiler_facts(request_object, repository_root)?;
    let selected_input_request_path = verify_reference_file_fact(
        json_field(
            request_object,
            "selectedInputRequest",
            "independent command request",
        )?,
        receipt_directory,
        None,
        None,
        "selected input request",
    )?;
    let selected_input_request_contents = fs::read_to_string(&selected_input_request_path)
        .map_err(|error| {
            format!(
                "read selected input request {}: {error}",
                selected_input_request_path.display()
            )
        })?;
    let selected_input_request: Value = parse_unique_json(
        &selected_input_request_contents,
        &format!(
            "selected input request {}",
            selected_input_request_path.display()
        ),
    )?;
    let source_before = read_reference_json(
        receipt_directory,
        json_field(
            request_object,
            "inputsBefore",
            "independent command request",
        )?,
        "inputs-before.json",
    )?;
    let source_before_binding = json_object(&source_before, "inputs-before")?;
    let source_after_binding = json_object(&inputs_after, "inputs-after")?;
    let source_before_manifest = parse_source_manifest(
        json_field(source_before_binding, "manifest", "inputs-before")?,
        repository_root,
    )?;
    let source_after_manifest = parse_source_manifest(
        json_field(source_after_binding, "manifest", "inputs-after")?,
        repository_root,
    )?;
    if source_before_manifest != source_after_manifest
        || sha256_json_object(&source_before_manifest)?
            != json_string(source_before_binding, "manifestSha256", "inputs-before")?
        || json_string(source_before_binding, "scope", "inputs-before")? != source_scope
        || json_string(source_before_binding, "protocol", "inputs-before")?
            != source_manifest_protocol
        || json_string(
            object,
            "sourceManifestSha256",
            "independent command receipt",
        )? != json_string(source_before_binding, "manifestSha256", "inputs-before")?
    {
        return Err(format!(
            "independent command selected source map changed or has mismatched protocol: {}",
            receipt_path.display()
        ));
    }
    let selected_input_object = json_object(&selected_input_request, "selected input request")?;
    if json_u64(
        selected_input_object,
        "schemaVersion",
        "selected input request",
    )? != 1
        || !matches_canonical_path(
            json_string(
                selected_input_object,
                "repositoryRoot",
                "selected input request",
            )?,
            repository_root,
        )
    {
        return Err(format!(
            "independent selected-input request does not bind its repository: {}",
            receipt_path.display()
        ));
    }
    let selected_inputs = parse_source_manifest(
        json_field(selected_input_object, "inputs", "selected input request")?,
        repository_root,
    )?;
    if selected_inputs != source_before_manifest {
        return Err(format!(
            "independent selected-input request differs from inputs-before: {}",
            receipt_path.display()
        ));
    }
    let artifacts_object = json_object(&cargo_artifacts, "cargo artifacts")?;
    let build_finished = json_array(
        json_field(artifacts_object, "buildFinished", "cargo artifacts")?,
        "cargo build-finished records",
    )?;
    if build_finished.len() != 1
        || !json_bool(
            json_object(&build_finished[0], "cargo build-finished record")?,
            "success",
            "cargo build-finished record",
        )?
    {
        return Err(format!(
            "independent command cargo artifact record is incomplete: {}",
            receipt_path.display()
        ));
    }
    let executable_artifacts = json_array(
        json_field(artifacts_object, "executableArtifacts", "cargo artifacts")?,
        "cargo executable artifacts",
    )?;
    let matching_artifacts = executable_artifacts
        .iter()
        .filter(|artifact| {
            json_object(artifact, "executable artifact")
                .and_then(|value| json_field(value, "binary", "executable artifact"))
                .and_then(|value| {
                    verify_json_file_fingerprint(
                        value,
                        expected_binary,
                        "executable artifact binary",
                    )
                })
                .is_ok()
        })
        .count();
    if matching_artifacts != 1 {
        return Err(format!(
            "independent command cargo artifacts do not uniquely bind current_exe: {}",
            receipt_path.display()
        ));
    }
    let native_object = json_object(&native, "native terminal")?;
    if json_u64(native_object, "schemaVersion", "native terminal")? != 1
        || json_u64(native_object, "rootExitCode", "native terminal")? != 0
        || json_string_array(
            json_field(native_object, "command", "native terminal")?,
            "native command",
        )? != command
        || !matches_canonical_path(
            json_string(native_object, "cwd", "native terminal")?,
            repository_root,
        )
        || json_u64(native_object, "pid", "native terminal")? == 0
        || json_string(native_object, "creationTime", "native terminal")?
            .bytes()
            .any(|byte| !byte.is_ascii_digit())
    {
        return Err(format!(
            "independent native terminal does not bind the original command: {}",
            receipt_path.display()
        ));
    }
    validate_native_terminal(native_object, receipt_path)?;
    Ok(SelectedSourceBinding {
        manifest: source_before_manifest,
        sha256: json_string(source_before_binding, "manifestSha256", "inputs-before")?.to_owned(),
        protocol: source_manifest_protocol.to_owned(),
        scope: source_scope.to_owned(),
    })
}

fn json_object<'a>(value: &'a Value, description: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{description} must be a JSON object"))
}

fn json_field<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    description: &str,
) -> Result<&'a Value, String> {
    object
        .get(field)
        .ok_or_else(|| format!("{description} is missing field {field}"))
}

fn json_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    description: &str,
) -> Result<&'a str, String> {
    json_field(object, field, description)?
        .as_str()
        .ok_or_else(|| format!("{description}.{field} must be a string"))
}

fn json_bool(object: &Map<String, Value>, field: &str, description: &str) -> Result<bool, String> {
    json_field(object, field, description)?
        .as_bool()
        .ok_or_else(|| format!("{description}.{field} must be a boolean"))
}

fn json_u64(object: &Map<String, Value>, field: &str, description: &str) -> Result<u64, String> {
    json_field(object, field, description)?
        .as_u64()
        .ok_or_else(|| format!("{description}.{field} must be a non-negative integer"))
}

fn json_array<'a>(value: &'a Value, description: &str) -> Result<&'a Vec<Value>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{description} must be a JSON array"))
}

fn json_string_array(value: &Value, description: &str) -> Result<Vec<String>, String> {
    Ok(json_array(value, description)?
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("{description} must contain non-empty strings"))
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn parse_file_fingerprint(
    value: &Value,
    description: &str,
) -> Result<ManifestFileFingerprint, String> {
    let object = json_object(value, description)?;
    let path = PathBuf::from(json_string(object, "path", description)?);
    let sha256 = json_string(object, "sha256", description)?.to_owned();
    if !is_sha256(&sha256) {
        return Err(format!("{description}.sha256 is not a lowercase SHA-256"));
    }
    let byte_length = object
        .get("byte_length")
        .or_else(|| object.get("byteLength"))
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{description} is missing byte_length/byteLength"))?;
    Ok(ManifestFileFingerprint {
        path,
        sha256,
        byte_length,
    })
}

fn verify_json_file_fingerprint(
    value: &Value,
    expected: &EvidenceFileFingerprint,
    description: &str,
) -> Result<(), String> {
    let declared = parse_file_fingerprint(value, description)?;
    let declared_path = fs::canonicalize(&declared.path)
        .map_err(|error| format!("resolve {description} {}: {error}", declared.path.display()))?;
    if declared_path != expected.path
        || declared.sha256 != expected.sha256
        || declared.byte_length != expected.byte_length
    {
        return Err(format!(
            "{description} does not bind the active file: expected={} actual={}",
            expected.path.display(),
            declared_path.display()
        ));
    }
    Ok(())
}

fn verify_reference_file_fact(
    value: &Value,
    base_directory: &Path,
    expected_directory: Option<&Path>,
    expected_leaf: Option<&str>,
    description: &str,
) -> Result<PathBuf, String> {
    let declared = parse_file_fingerprint(value, description)?;
    let path = resolve_manifest_path(base_directory, &declared.path);
    let canonical = fs::canonicalize(&path)
        .map_err(|error| format!("resolve {description} {}: {error}", path.display()))?;
    if !is_allowed_evidence_path(&canonical) {
        return Err(format!(
            "{description} is outside an allowed cargo-targets evidence root: {}",
            canonical.display()
        ));
    }
    if let Some(directory) = expected_directory {
        let expected = fs::canonicalize(directory).map_err(|error| {
            format!(
                "resolve expected {description} directory {}: {error}",
                directory.display()
            )
        })?;
        if canonical.parent() != Some(expected.as_path()) {
            return Err(format!(
                "{description} escaped its receipt directory: {}",
                canonical.display()
            ));
        }
    }
    if let Some(leaf) = expected_leaf {
        if canonical.file_name().and_then(|name| name.to_str()) != Some(leaf) {
            return Err(format!(
                "{description} has an unexpected leaf name: {}",
                canonical.display()
            ));
        }
    }
    let actual = fingerprint_file(&canonical, description)?;
    if actual.sha256 != declared.sha256 || actual.byte_length != declared.byte_length {
        return Err(format!(
            "{description} changed after it was recorded: {}",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn read_reference_json(
    base_directory: &Path,
    value: &Value,
    expected_leaf: &str,
) -> Result<Value, String> {
    let path = verify_reference_file_fact(
        value,
        base_directory,
        Some(base_directory),
        Some(expected_leaf),
        expected_leaf,
    )?;
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("read {} {}: {error}", expected_leaf, path.display()))?;
    parse_unique_json(&contents, &format!("{} {}", expected_leaf, path.display()))
}

fn canonical_existing_path(path: &str, description: &str) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|error| format!("resolve {description} {path}: {error}"))
}

fn parse_source_manifest(
    value: &Value,
    repository_root: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let object = json_object(value, "source manifest")?;
    if object.is_empty() {
        return Err("source manifest must not be empty".to_owned());
    }
    let mut manifest = BTreeMap::new();
    for (relative_path, value) in object {
        let path = Path::new(relative_path);
        if path.is_absolute()
            || relative_path.contains('\\')
            || relative_path.is_empty()
            || path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::CurDir | std::path::Component::ParentDir
                )
            })
        {
            return Err(format!(
                "source manifest has an unsafe path: {relative_path}"
            ));
        }
        let expected = repository_root.join(path);
        let expected = fs::canonicalize(&expected)
            .map_err(|error| format!("resolve source manifest file {relative_path}: {error}"))?;
        if !expected.starts_with(repository_root) {
            return Err(format!(
                "source manifest escaped repository root: {relative_path}"
            ));
        }
        let declared_hash = value
            .as_str()
            .ok_or_else(|| format!("source manifest hash for {relative_path} must be a string"))?;
        if !is_sha256(declared_hash) {
            return Err(format!(
                "source manifest hash for {relative_path} is invalid"
            ));
        }
        let actual = fingerprint_file(&expected, "source manifest input")?;
        if actual.sha256 != declared_hash {
            return Err(format!("source manifest input changed: {relative_path}"));
        }
        manifest.insert(relative_path.clone(), declared_hash.to_owned());
    }
    Ok(manifest)
}

fn sha256_json_object(manifest: &BTreeMap<String, String>) -> Result<String, String> {
    let bytes = serde_json::to_vec(manifest)
        .map_err(|error| format!("serialize source manifest for hashing: {error}"))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn is_allowed_target_directory(path: &Path) -> bool {
    let value = evidence_transport_path(path)
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    [
        "d:\\cargo-targets\\",
        "e:\\cargo-targets\\",
        "f:\\cargo-targets\\",
    ]
    .iter()
    .any(|prefix| value.starts_with(prefix))
        && !value.contains("\\targets\\")
        && !value.contains("\\zirconbuilds\\")
}

fn is_allowed_evidence_path(path: &Path) -> bool {
    let value = evidence_transport_path(path)
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    [
        "d:\\cargo-targets\\",
        "e:\\cargo-targets\\",
        "f:\\cargo-targets\\",
    ]
    .iter()
    .any(|prefix| value.starts_with(prefix))
}

fn validate_compiler_facts(
    request: &Map<String, Value>,
    repository_root: &Path,
) -> Result<(), String> {
    let compiler = json_object(
        json_field(request, "compiler", "independent command request")?,
        "compiler facts",
    )?;
    let binaries = json_object(
        json_field(compiler, "binaries", "compiler facts")?,
        "compiler binaries",
    )?;
    for name in ["cargo", "rustc", "rustdoc"] {
        let binary = parse_file_fingerprint(
            json_field(binaries, name, "compiler binaries")?,
            "compiler binary",
        )?;
        let actual = fingerprint_file(&binary.path, "compiler binary")?;
        if actual.sha256 != binary.sha256 || actual.byte_length != binary.byte_length {
            return Err(format!(
                "compiler binary changed: {}",
                binary.path.display()
            ));
        }
    }
    let drivers = json_object(
        json_field(request, "driverFiles", "independent command request")?,
        "driver files",
    )?;
    let expected = [
        "tools/dev/local_cargo.py",
        "tools/dev/local-cargo.ps1",
        "tools/jenkins/pilot/native/windows_job_process.py",
        "tools/jenkins/pilot/native/process_identity.py",
        "tools/jenkins/pilot/process_evidence.py",
        "tools/jenkins/pilot/source_manifest.py",
        "tools/jenkins/pilot/managed_cargo.py",
        "tools/jenkins/resources/paths.py",
    ];
    if drivers.len() != expected.len() {
        return Err("independent command driver set is incomplete".to_owned());
    }
    for name in expected {
        let declared =
            parse_file_fingerprint(json_field(drivers, name, "driver files")?, "driver file")?;
        let actual_path = repository_root.join(name);
        let actual = fingerprint_file(&actual_path, "driver file")?;
        if actual.sha256 != declared.sha256 || actual.byte_length != declared.byte_length {
            return Err(format!("driver file changed: {name}"));
        }
    }
    Ok(())
}

fn validate_native_terminal(
    native: &Map<String, Value>,
    receipt_path: &Path,
) -> Result<(), String> {
    let process_tree = json_object(
        json_field(native, "processTree", "native terminal")?,
        "process tree",
    )?;
    if json_string(process_tree, "scope", "process tree")? != "retained-windows-job"
        || ![
            "terminal",
            "nativeWaiterJoined",
            "readersFinished",
            "pipeEOF",
        ]
        .iter()
        .all(|field| json_bool(process_tree, field, "process tree").unwrap_or(false))
        || json_bool(
            process_tree,
            "jobHandleRetainedUntilCliExit",
            "process tree",
        )?
    {
        return Err(format!(
            "native process tree is incomplete: {}",
            receipt_path.display()
        ));
    }
    let pipes = json_object(
        json_field(process_tree, "pipes", "process tree")?,
        "process pipes",
    )?;
    for name in ["stdout", "stderr"] {
        let pipe = json_object(json_field(pipes, name, "process pipes")?, "process pipe")?;
        if !json_bool(pipe, "eof", "process pipe")? || !json_bool(pipe, "closed", "process pipe")? {
            return Err(format!(
                "native {name} pipe did not reach EOF: {}",
                receipt_path.display()
            ));
        }
    }
    if !json_array(
        json_field(native, "terminalErrors", "native terminal")?,
        "terminal errors",
    )?
    .is_empty()
        || json_bool(native, "formalAcceptance", "native terminal")?
    {
        return Err(format!(
            "native terminal contains an error or acceptance upgrade: {}",
            receipt_path.display()
        ));
    }
    let job = json_object(
        json_field(native, "nativeJobEvidence", "native terminal")?,
        "native Job evidence",
    )?;
    if json_string(job, "access", "native Job evidence")? != "JOB_OBJECT_QUERY"
        || !json_array(
            json_field(job, "errors", "native Job evidence")?,
            "native Job errors",
        )?
        .is_empty()
    {
        return Err(format!(
            "native Job evidence is incomplete: {}",
            receipt_path.display()
        ));
    }
    let records = json_object(
        json_field(job, "records", "native Job evidence")?,
        "native Job records",
    )?;
    for stage in ["afterTerminal", "afterClose"] {
        let record = json_object(
            json_field(records, stage, "native Job records")?,
            "native Job record",
        )?;
        if json_u64(record, "activeProcesses", "native Job record")? != 0 {
            return Err(format!(
                "native Job retained an active process: {}",
                receipt_path.display()
            ));
        }
    }
    let before_resume = json_object(
        json_field(records, "beforeResume", "native Job records")?,
        "native Job before-resume record",
    )?;
    if json_u64(
        before_resume,
        "activeProcesses",
        "native Job before-resume record",
    )? < 1
    {
        return Err(format!(
            "native Job did not record a started process: {}",
            receipt_path.display()
        ));
    }
    let pid = json_u64(native, "pid", "native terminal")?;
    let creation_time = json_string(native, "creationTime", "native terminal")?;
    let members = json_array(
        json_field(before_resume, "members", "native Job before-resume record")?,
        "native Job members",
    )?;
    if !members.iter().any(|member| {
        let Ok(member) = json_object(member, "native Job member") else {
            return false;
        };
        json_u64(member, "pid", "native Job member").ok() == Some(pid)
            && json_string(member, "creationTime", "native Job member").ok() == Some(creation_time)
            && json_bool(member, "member", "native Job member").ok() == Some(true)
            && json_field(member, "error", "native Job member").is_err()
    }) {
        return Err(format!(
            "native Job start inventory does not bind the root process: {}",
            receipt_path.display()
        ));
    }
    Ok(())
}

pub(crate) fn fingerprint_file(
    path: &Path,
    description: &str,
) -> Result<EvidenceFileFingerprint, String> {
    let canonical_path = fs::canonicalize(path)
        .map_err(|error| format!("resolve {description} {}: {error}", path.display()))?;
    let metadata = fs::metadata(&canonical_path).map_err(|error| {
        format!(
            "inspect {description} {}: {error}",
            canonical_path.display()
        )
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "{description} must be a file: {}",
            canonical_path.display()
        ));
    }
    let mut file = File::open(&canonical_path)
        .map_err(|error| format!("open {description} {}: {error}", canonical_path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|error| format!("read {description} {}: {error}", canonical_path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(EvidenceFileFingerprint {
        path: canonical_path,
        sha256: format!("{:x}", hasher.finalize()),
        byte_length: metadata.len(),
    })
}

pub(crate) fn evidence_transport_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let value = path.as_os_str().to_string_lossy();
        if let Some(unc_path) = value.strip_prefix("\\\\?\\UNC\\") {
            return PathBuf::from(format!("\\\\{unc_path}"));
        }
        if let Some(normal_path) = value.strip_prefix("\\\\?\\") {
            return PathBuf::from(normal_path);
        }
    }
    path.to_path_buf()
}

fn verify_manifest_file(
    declared: &ManifestFileFingerprint,
    expected_path: &Path,
    identity_directory: &Path,
    description: &str,
) -> Result<EvidenceFileFingerprint, String> {
    if !is_sha256(&declared.sha256) {
        return Err(format!(
            "Ready-frame evidence identity has an invalid {description} SHA-256"
        ));
    }
    let actual = fingerprint_file(expected_path, description)?;
    let declared_path = resolve_manifest_path(identity_directory, &declared.path);
    let declared_canonical_path = fs::canonicalize(&declared_path).map_err(|error| {
        format!(
            "resolve declared {description} {}: {error}",
            declared_path.display()
        )
    })?;
    if actual.path != declared_canonical_path {
        return Err(format!(
            "Ready-frame evidence identity {description} path does not match the active input: expected={} actual={}",
            declared_canonical_path.display(),
            actual.path.display()
        ));
    }
    if actual.sha256 != declared.sha256 || actual.byte_length != declared.byte_length {
        return Err(format!(
            "Ready-frame evidence identity {description} fingerprint does not match the active input: {}",
            actual.path.display()
        ));
    }
    Ok(actual)
}

fn resolve_manifest_path(identity_directory: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        identity_directory.join(path)
    }
}

fn is_safe_run_id(value: &str) -> bool {
    value.len() >= 3
        && value.len() <= 160
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
#[path = "tests/evidence_identity.rs"]
mod tests;
