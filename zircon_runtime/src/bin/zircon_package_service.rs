use serde::Deserialize;
use std::{
    io::{self, BufRead, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
use zircon_runtime::{
    core::framework::platform::RuntimeTargetMode,
    plugin::{
        native::NativePluginArtifactTarget,
        package_service::{
            capture_project_package_lock, load_host_policy_index_for_target, InstallRequest,
            LoadedNativePluginPolicy, NativePluginPolicyError, PackageError, PackageInventory,
            PackageStore, PreparedPackage, ProjectPackageLockProviderError,
            INSTALL_REQUEST_SCHEMA_V2, MAX_PACKAGE_BYTES,
        },
    },
};
use zircon_runtime_interface::project::ProjectPackageLockProject;

const PROJECT_LOCK_REQUEST_SCHEMA_VERSION_V1: u8 = 1;

#[derive(Deserialize)]
#[serde(
    tag = "action",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Request {
    Install {
        request: InstallRequest,
    },
    Inventory {
        schema_version: u8,
        target: NativePluginArtifactTarget,
        identity_digest: String,
    },
    Receipt {
        schema_version: u8,
        target: NativePluginArtifactTarget,
        identity_digest: String,
        operation_id: String,
    },
    ProjectLock {
        schema_version: u8,
        target: NativePluginArtifactTarget,
        project_root: PathBuf,
        project: ProjectPackageLockProject,
    },
}

fn line(input: &mut impl BufRead) -> Result<Vec<u8>, PackageError> {
    let mut bytes = Vec::new();
    input
        .take(65537)
        .read_until(b'\n', &mut bytes)
        .map_err(|_| PackageError::Invalid)?;
    if bytes.len() > 65536 || bytes.last() != Some(&b'\n') {
        return Err(PackageError::Invalid);
    }
    Ok(bytes)
}

fn emit(value: serde_json::Value) -> Result<(), PackageError> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, &value).map_err(|_| PackageError::Storage)?;
    output
        .write_all(b"\n")
        .and_then(|_| output.flush())
        .map_err(|_| PackageError::Storage)
}

fn policy_error(error: NativePluginPolicyError) -> PackageError {
    match error {
        NativePluginPolicyError::Unconfigured => PackageError::PolicyUnconfigured,
        NativePluginPolicyError::TargetUnconfigured => PackageError::TargetUnconfigured,
        NativePluginPolicyError::Rejected | NativePluginPolicyError::StoreUnavailable => {
            PackageError::Trust
        }
    }
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn load_policy(
    index_path: &Path,
    expected_index_sha256: &str,
    target: &NativePluginArtifactTarget,
) -> Result<LoadedNativePluginPolicy, PackageError> {
    if !valid_digest(expected_index_sha256)
        || target.runtime_mode == RuntimeTargetMode::ServerRuntime
    {
        return Err(PackageError::Invalid);
    }
    let loaded = load_host_policy_index_for_target(index_path, target).map_err(policy_error)?;
    if loaded.index_sha256() != expected_index_sha256 {
        return Err(PackageError::Trust);
    }
    Ok(loaded)
}

fn valid_project_lock_path(path: &Path) -> bool {
    path.is_absolute()
        && path.components().all(|component| {
            !matches!(
                component,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )
        })
}

fn validate_project_lock_request(
    schema_version: u8,
    target: &NativePluginArtifactTarget,
    project_root: &Path,
) -> Result<(), PackageError> {
    if schema_version != PROJECT_LOCK_REQUEST_SCHEMA_VERSION_V1
        || target.runtime_mode == RuntimeTargetMode::ServerRuntime
        || !valid_project_lock_path(project_root)
    {
        return Err(PackageError::Invalid);
    }
    Ok(())
}

fn project_lock_error(error: ProjectPackageLockProviderError) -> PackageError {
    match error {
        ProjectPackageLockProviderError::Policy => PackageError::Trust,
        ProjectPackageLockProviderError::BuildSetMismatch
        | ProjectPackageLockProviderError::ProjectOverlap
        | ProjectPackageLockProviderError::SelectionMissing
        | ProjectPackageLockProviderError::InvalidProjectIdentity
        | ProjectPackageLockProviderError::InvalidLock => PackageError::Trust,
        ProjectPackageLockProviderError::InstallationMissing => PackageError::Storage,
        ProjectPackageLockProviderError::InstallationChanged => PackageError::Conflict,
        ProjectPackageLockProviderError::UnsupportedTarget => PackageError::Invalid,
    }
}

fn line_request_target(request: &Request) -> Result<&NativePluginArtifactTarget, PackageError> {
    match request {
        Request::Install { request } => {
            request.validate()?;
            Ok(&request.target)
        }
        Request::Inventory {
            schema_version,
            target,
            ..
        }
        | Request::Receipt {
            schema_version,
            target,
            ..
        } => {
            if *schema_version != INSTALL_REQUEST_SCHEMA_V2
                || target.runtime_mode == RuntimeTargetMode::ServerRuntime
            {
                return Err(PackageError::Invalid);
            }
            Ok(target)
        }
        Request::ProjectLock {
            schema_version,
            target,
            project_root,
            ..
        } => {
            validate_project_lock_request(*schema_version, target, project_root)?;
            Ok(target)
        }
    }
}

fn run() -> Result<(), PackageError> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or(PackageError::Invalid)?;
    let expected_index_sha256 = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or(PackageError::Invalid)?;
    if args.next().is_some() || !Path::new(&path).is_absolute() {
        return Err(PackageError::Invalid);
    }

    let stdin = io::stdin();
    let mut input = stdin.lock();
    let request: Request =
        serde_json::from_slice(&line(&mut input)?).map_err(|_| PackageError::Invalid)?;
    let target = line_request_target(&request)?.clone();
    let loaded = load_policy(Path::new(&path), &expected_index_sha256, &target)?;

    match request {
        Request::Inventory {
            identity_digest, ..
        } => {
            let inventory =
                match PackageStore::open_existing(&loaded.policy().root, &identity_digest)? {
                    Some(store) => store.inventory()?,
                    None => PackageInventory::default(),
                };
            emit(serde_json::json!({
                "status": "ready",
                "target": target,
                "inventory": inventory
            }))
        }
        Request::Receipt {
            identity_digest,
            operation_id,
            ..
        } => {
            let receipt =
                match PackageStore::open_existing(&loaded.policy().root, &identity_digest)? {
                    Some(store) => store.receipt(&operation_id)?,
                    None => None,
                };
            if let Some(receipt) = &receipt {
                if receipt.schema_version != 2 {
                    return Err(PackageError::Trust);
                }
                loaded
                    .persist_installed_receipt_selection(&identity_digest, receipt)
                    .map_err(|_| PackageError::OutcomeUnknown)?;
            }
            emit(serde_json::json!({
                "status": "ready",
                "target": target,
                "receipt": receipt
            }))
        }
        Request::ProjectLock {
            project_root,
            project,
            ..
        } => {
            let lock = capture_project_package_lock(loaded, project, &project_root, target.clone())
                .map_err(project_lock_error)?;
            emit(serde_json::json!({
                "status": "ready",
                "target": target,
                "lock": lock
            }))
        }
        Request::Install { request } => {
            let mut bytes = vec![
                0;
                usize::try_from(request.artifact_size)
                    .ok()
                    .filter(|size| *size <= MAX_PACKAGE_BYTES)
                    .ok_or(PackageError::Capacity)?
            ];
            input
                .read_exact(&mut bytes)
                .map_err(|_| PackageError::Invalid)?;
            let prepared = PreparedPackage::verify(request.clone(), bytes, loaded.policy())?;
            emit(serde_json::json!({
                "status": "prepared",
                "operationId": request.operation_id
            }))?;

            // The broker rechecks account generation and entitlement before this command.
            if line(&mut input)? != b"commit\n" {
                return Err(PackageError::Invalid);
            }

            // Re-open the host-selected index at the commit boundary. The request digest came
            // from Hub's pinned host configuration, and the same exact selected policy must
            // still be current before durable package publication.
            let refreshed = load_policy(Path::new(&path), &expected_index_sha256, &target)?;
            if refreshed.policy_sha256() != loaded.policy_sha256()
                || refreshed.policy_path() != loaded.policy_path()
            {
                return Err(PackageError::Trust);
            }
            let store = PackageStore::open(&loaded.policy().root, &request.identity_digest)?;
            let receipt = store.commit(prepared)?;
            loaded
                .persist_installed_receipt_selection(&request.identity_digest, &receipt)
                .map_err(|_| PackageError::OutcomeUnknown)?;
            emit(serde_json::json!({
                "status": "committed",
                "receipt": receipt
            }))
        }
    }
}

fn main() -> ExitCode {
    let (done, receiver) = std::sync::mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        if receiver
            .recv_timeout(std::time::Duration::from_secs(120))
            .is_err()
        {
            // Abrupt termination leaves existing transaction recovery evidence intact.
            std::process::exit(124);
        }
    });
    let result = match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = emit(serde_json::json!({"status":"failed", "error":error.to_string()}));
            ExitCode::FAILURE
        }
    };
    let _ = done.send(());
    let _ = watchdog.join();
    result
}

#[cfg(test)]
#[path = "tests/zircon_package_service.rs"]
mod tests;
