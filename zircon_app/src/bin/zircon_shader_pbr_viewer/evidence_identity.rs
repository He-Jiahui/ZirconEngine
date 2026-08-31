use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

pub(crate) const READY_FRAME_EVIDENCE_IDENTITY_SCHEMA: &str =
    "zircon_shader_pbr_viewer_evidence_identity_v1";
pub(crate) const READY_FRAME_EVIDENCE_VALIDATION_POLICY: &str =
    "zircon_shader_pbr_viewer_ready_frame_v17";

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
    let manifest: EvidenceIdentityManifest =
        serde_json::from_str(contents.trim_start_matches('\u{feff}')).map_err(|error| {
            format!(
                "parse Ready-frame evidence identity {}: {error}",
                identity_manifest.path.display()
            )
        })?;

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
mod tests {
    use super::{is_safe_run_id, is_sha256};

    #[test]
    fn evidence_identity_ids_and_hashes_have_stable_machine_readable_forms() {
        assert!(is_safe_run_id(
            "shader-pbr-20260825-a1b2c3d4-warm-measured-01"
        ));
        assert!(!is_safe_run_id("a"));
        assert!(!is_safe_run_id("1abc"));
        assert!(!is_safe_run_id("shader-pbr/unsafe"));
        assert!(is_sha256(&"a".repeat(64)));
        assert!(!is_sha256(&"A".repeat(64)));
    }

    #[cfg(windows)]
    #[test]
    fn evidence_transport_paths_do_not_leak_windows_verbatim_prefixes() {
        assert_eq!(
            super::evidence_transport_path(std::path::Path::new(r"\\?\E:\profile\ready.png")),
            std::path::PathBuf::from(r"E:\profile\ready.png")
        );
        assert_eq!(
            super::evidence_transport_path(std::path::Path::new(r"\\?\UNC\host\share\ready.png")),
            std::path::PathBuf::from(r"\\host\share\ready.png")
        );
    }
}
