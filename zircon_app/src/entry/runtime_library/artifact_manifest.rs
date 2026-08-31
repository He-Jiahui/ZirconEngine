use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use zircon_runtime_interface::runtime_build_set::{
    ZrRuntimeArtifactIdentityV1, ZrRuntimeArtifactManifestV1, ZrRuntimeBuildSetExpectationV1,
    ZrRuntimeDigestV1, ZrRuntimeTargetModelV1,
};

use super::RuntimeLibraryError;

const MAX_RUNTIME_ARTIFACT_MANIFEST_BYTES: u64 = 64 * 1024;
const RUNTIME_ARTIFACT_HASH_BUFFER_BYTES: usize = 1024 * 1024;

/// Reads and verifies the staged Runtime DLL identity before dynamic loading can execute code.
pub(super) fn validate_runtime_library_artifact(
    library_path: &Path,
) -> Result<ZrRuntimeArtifactManifestV1, RuntimeLibraryError> {
    let manifest_path = runtime_artifact_manifest_path(library_path)?;
    let manifest = read_runtime_artifact_manifest(&manifest_path)?;
    let library = artifact_identity_for_file(library_path)?;
    if manifest.artifact != library {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime artifact manifest {} does not match staged runtime library {}",
            manifest_path.display(),
            library_path.display(),
        )));
    }

    let host_path = env::current_exe().map_err(|error| {
        RuntimeLibraryError::new(format!(
            "failed to resolve current executable while validating runtime BuildSet: {error}"
        ))
    })?;
    let host = artifact_identity_for_file(&host_path)?;
    let expected = ZrRuntimeBuildSetExpectationV1::new(
        manifest.build_set_id.clone(),
        ZrRuntimeTargetModelV1::current(),
        std::iter::empty::<String>(),
    )
    .map_err(|error| {
        RuntimeLibraryError::protocol_violation(format!(
            "runtime artifact manifest {} cannot construct a host BuildSet expectation: {error}",
            manifest_path.display(),
        ))
    })?
    .with_host_artifact(host);
    manifest.validate_against(&expected).map_err(|error| {
        RuntimeLibraryError::protocol_violation(format!(
            "runtime artifact manifest {} rejected before dynamic loading: {error}",
            manifest_path.display(),
        ))
    })?;
    Ok(manifest)
}

pub(super) fn runtime_artifact_manifest_path(
    library_path: &Path,
) -> Result<PathBuf, RuntimeLibraryError> {
    let file_name = library_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            RuntimeLibraryError::new(format!(
                "runtime library path {} has no UTF-8 file name for its artifact manifest",
                library_path.display(),
            ))
        })?;
    Ok(library_path.with_file_name(format!("{file_name}.manifest.json")))
}

fn read_runtime_artifact_manifest(
    manifest_path: &Path,
) -> Result<ZrRuntimeArtifactManifestV1, RuntimeLibraryError> {
    let metadata = fs::metadata(manifest_path).map_err(|error| {
        RuntimeLibraryError::new(format!(
            "failed to inspect runtime artifact manifest {}: {error}",
            manifest_path.display(),
        ))
    })?;
    if metadata.len() > MAX_RUNTIME_ARTIFACT_MANIFEST_BYTES {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime artifact manifest {} exceeds {} bytes",
            manifest_path.display(),
            MAX_RUNTIME_ARTIFACT_MANIFEST_BYTES,
        )));
    }
    let mut source = File::open(manifest_path).map_err(|error| {
        RuntimeLibraryError::new(format!(
            "failed to open runtime artifact manifest {}: {error}",
            manifest_path.display(),
        ))
    })?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    source
        .by_ref()
        .take(MAX_RUNTIME_ARTIFACT_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            RuntimeLibraryError::new(format!(
                "failed to read runtime artifact manifest {}: {error}",
                manifest_path.display(),
            ))
        })?;
    if bytes.len() as u64 > MAX_RUNTIME_ARTIFACT_MANIFEST_BYTES {
        return Err(RuntimeLibraryError::protocol_violation(format!(
            "runtime artifact manifest {} exceeds {} bytes",
            manifest_path.display(),
            MAX_RUNTIME_ARTIFACT_MANIFEST_BYTES,
        )));
    }
    serde_json::from_slice(&bytes).map_err(|error| {
        RuntimeLibraryError::protocol_violation(format!(
            "failed to decode runtime artifact manifest {}: {error}",
            manifest_path.display(),
        ))
    })
}

fn artifact_identity_for_file(
    path: &Path,
) -> Result<ZrRuntimeArtifactIdentityV1, RuntimeLibraryError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            RuntimeLibraryError::new(format!(
                "runtime BuildSet artifact path {} has no UTF-8 file name",
                path.display(),
            ))
        })?;
    let digest = file_sha256(path)?;
    ZrRuntimeArtifactIdentityV1::new(file_name, digest).map_err(|error| {
        RuntimeLibraryError::protocol_violation(format!(
            "runtime BuildSet artifact path {} is invalid: {error}",
            path.display(),
        ))
    })
}

fn file_sha256(path: &Path) -> Result<ZrRuntimeDigestV1, RuntimeLibraryError> {
    let mut source = File::open(path).map_err(|error| {
        RuntimeLibraryError::new(format!(
            "failed to open runtime BuildSet artifact {}: {error}",
            path.display(),
        ))
    })?;
    let mut hasher = Sha256::new();
    // This runs once at process startup before `Library::new` to detect a
    // static staging mismatch. It is not a handle-bound anti-replacement
    // defense: loading by path still has a time-of-check/time-of-use window.
    let mut chunk = [0_u8; RUNTIME_ARTIFACT_HASH_BUFFER_BYTES];
    loop {
        let read = source.read(&mut chunk).map_err(|error| {
            RuntimeLibraryError::new(format!(
                "failed to hash runtime BuildSet artifact {}: {error}",
                path.display(),
            ))
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&chunk[..read]);
    }
    ZrRuntimeDigestV1::parse(format!("{:x}", hasher.finalize())).map_err(|error| {
        RuntimeLibraryError::protocol_violation(format!(
            "runtime BuildSet artifact {} produced an invalid SHA-256 digest: {error}",
            path.display(),
        ))
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::runtime_artifact_manifest_path;

    #[test]
    fn runtime_artifact_manifest_is_a_library_sidecar() {
        assert_eq!(
            runtime_artifact_manifest_path(Path::new("E:/build/zircon_runtime.dll")).unwrap(),
            Path::new("E:/build/zircon_runtime.dll.manifest.json"),
        );
    }
}
