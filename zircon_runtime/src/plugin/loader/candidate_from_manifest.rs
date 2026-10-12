use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

use super::discovery_refresh::{
    NativePluginDiscoveryRefreshError, NativePluginDiscoveryRefreshRequest,
    NativePluginDiscoveryRefreshSink,
};
use super::dynamic_library_name::dynamic_library_file_name;
use super::NativePluginCandidate;

pub(super) type NativePluginManifestCandidateResult<T> =
    std::result::Result<T, NativePluginManifestCandidateError>;

#[derive(Debug)]
pub(super) enum NativePluginManifestCandidateError {
    ParseManifest {
        manifest_path: PathBuf,
        source: toml::de::Error,
    },
    MissingRuntimeOrEditorModule {
        plugin_id: String,
    },
}

impl std::fmt::Display for NativePluginManifestCandidateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseManifest {
                manifest_path,
                source,
            } => write!(
                formatter,
                "failed to parse native plugin manifest {}: {source}",
                manifest_path.display()
            ),
            Self::MissingRuntimeOrEditorModule { plugin_id } => write!(
                formatter,
                "native plugin {plugin_id} has no runtime or editor module crate declared"
            ),
        }
    }
}

impl std::error::Error for NativePluginManifestCandidateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseManifest { source, .. } => Some(source),
            Self::MissingRuntimeOrEditorModule { .. } => None,
        }
    }
}

pub(super) fn append_candidate_from_manifest_path(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    manifest_path: PathBuf,
) -> Result<(), NativePluginDiscoveryRefreshError> {
    metered_candidate_from_manifest_path(request, sink, manifest_path)?.insert(sink);
    Ok(())
}

/// A parsed candidate whose admission slot is deliberately held until a caller has applied its
/// selection-specific validation. Full scans insert it immediately; load manifests validate path
/// and id declarations before publication.
pub(super) struct MeteredNativePluginCandidate {
    reservation: super::discovery_refresh::NativePluginDiscoveryRefreshCandidateReservation,
    candidate: NativePluginCandidate,
}

impl MeteredNativePluginCandidate {
    pub(super) fn candidate(&self) -> &NativePluginCandidate {
        &self.candidate
    }

    pub(super) fn insert(self, sink: &mut NativePluginDiscoveryRefreshSink) {
        self.reservation.insert(sink, self.candidate);
    }
}

pub(super) fn metered_candidate_from_manifest_path(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    manifest_path: PathBuf,
) -> Result<MeteredNativePluginCandidate, NativePluginDiscoveryRefreshError> {
    let candidate_reservation = sink.reserve_candidate(request)?;
    sink.record_manifest_read();
    let source = read_bounded_utf8_file(request, sink, &manifest_path, "native plugin manifest")?;
    let _parse_admission = sink.reserve_additional_scratch_bytes(request, source.len() as u64)?;
    sink.record_manifest_parse();
    let candidate = candidate_from_manifest_source(manifest_path, &source)
        .map_err(|error| NativePluginDiscoveryRefreshError::collector(error.to_string()))?;
    Ok(MeteredNativePluginCandidate {
        reservation: candidate_reservation,
        candidate,
    })
}

/// Reads exactly one bounded UTF-8 file through the refresh sink. Both directory traversal and
/// explicit load-manifest selection use this one path so metadata changes, chunk reads, and
/// scratch admission cannot diverge.
pub(super) fn read_bounded_utf8_file(
    request: &NativePluginDiscoveryRefreshRequest,
    sink: &mut NativePluginDiscoveryRefreshSink,
    path: &Path,
    file_kind: &str,
) -> Result<String, NativePluginDiscoveryRefreshError> {
    const READ_CHUNK_BYTES: usize = 8 * 1024;

    request.check_active()?;
    let mut file = fs::File::open(path).map_err(|source| read_error(file_kind, path, source))?;
    let expected_bytes = file
        .metadata()
        .map_err(|source| read_error(file_kind, path, source))?
        .len();
    if expected_bytes > sink.remaining_read_bytes() {
        // Metadata is only an early rejection hint. Actual reads below remain individually
        // admitted, including when a file grows after this first handle query.
        return match sink.reserve_read_bytes(request, expected_bytes) {
            Err(error) => Err(error),
            Ok(_) => Err(NativePluginDiscoveryRefreshError::collector(format!(
                "{file_kind} {} exceeded the remaining read budget without rejection",
                path.display()
            ))),
        };
    }
    let expected_len = usize::try_from(expected_bytes).map_err(|_| {
        NativePluginDiscoveryRefreshError::collector(format!(
            "{file_kind} {} cannot fit this platform's bounded buffer",
            path.display()
        ))
    })?;
    let _source_admission = sink.reserve_additional_scratch_bytes(request, expected_bytes)?;
    let mut source = Vec::<u8>::new();
    source.try_reserve_exact(expected_len).map_err(|_| {
        NativePluginDiscoveryRefreshError::collector(format!(
            "{file_kind} {} cannot reserve its bounded source buffer",
            path.display()
        ))
    })?;
    source.resize(expected_len, 0);
    let mut filled = 0;
    while filled < source.len() {
        request.check_active()?;
        let end = filled.saturating_add(READ_CHUNK_BYTES).min(source.len());
        let read_reservation = sink.reserve_read_bytes(request, (end - filled) as u64)?;
        let actual_bytes = match file.read(&mut source[filled..end]) {
            Ok(actual_bytes) => actual_bytes,
            Err(source) => {
                read_reservation.commit(sink, 0)?;
                return Err(read_error(file_kind, path, source));
            }
        };
        read_reservation.commit(sink, actual_bytes as u64)?;
        if actual_bytes == 0 {
            break;
        }
        filled = filled.saturating_add(actual_bytes);
    }

    request.check_active()?;
    let current_bytes = file
        .metadata()
        .map_err(|source| read_error(file_kind, path, source))?
        .len();
    ensure_bounded_read_is_stable(
        file_kind,
        path,
        expected_bytes,
        source.len(),
        filled,
        current_bytes,
    )?;
    String::from_utf8(source).map_err(|error| {
        read_error(
            file_kind,
            path,
            io::Error::new(io::ErrorKind::InvalidData, error),
        )
    })
}

fn read_error(
    file_kind: &str,
    path: &Path,
    source: io::Error,
) -> NativePluginDiscoveryRefreshError {
    NativePluginDiscoveryRefreshError::collector(format!(
        "failed to read {file_kind} {}: {source}",
        path.display()
    ))
}

fn ensure_bounded_read_is_stable(
    file_kind: &str,
    path: &Path,
    expected_bytes: u64,
    buffer_len: usize,
    filled: usize,
    current_bytes: u64,
) -> Result<(), NativePluginDiscoveryRefreshError> {
    if filled == buffer_len && current_bytes == expected_bytes {
        return Ok(());
    }
    Err(NativePluginDiscoveryRefreshError::collector(format!(
        "{file_kind} changed while it was read: {}",
        path.display()
    )))
}

fn candidate_from_manifest_source(
    manifest_path: PathBuf,
    source: &str,
) -> NativePluginManifestCandidateResult<NativePluginCandidate> {
    let manifest = toml::from_str::<PluginPackageManifest>(&source).map_err(|source| {
        NativePluginManifestCandidateError::ParseManifest {
            manifest_path: manifest_path.clone(),
            source,
        }
    })?;
    let library_path = native_library_path_for_manifest(
        &manifest_path,
        &manifest,
        &[PluginModuleKind::Runtime, PluginModuleKind::Editor],
    )
    .ok_or_else(
        || NativePluginManifestCandidateError::MissingRuntimeOrEditorModule {
            plugin_id: manifest.id.clone(),
        },
    )?;
    Ok(NativePluginCandidate {
        plugin_id: manifest.id.clone(),
        package_manifest: manifest,
        manifest_path,
        library_path,
    })
}

pub(super) fn native_library_paths_for_candidate(
    candidate: &NativePluginCandidate,
    module_kinds: &[PluginModuleKind],
) -> Vec<(PathBuf, Vec<PluginModuleKind>)> {
    let Some(package_root) = candidate.manifest_path.parent() else {
        return Vec::new();
    };
    let mut paths = Vec::<(PathBuf, Vec<PluginModuleKind>)>::with_capacity(module_kinds.len());
    for module_kind in module_kinds {
        let Some(file_name) = native_library_file_name_for_manifest(
            &candidate.package_manifest,
            std::slice::from_ref(module_kind),
        ) else {
            continue;
        };
        let library_path = package_root.join("native").join(file_name);
        let library_path = library_path.canonicalize().unwrap_or(library_path);
        if let Some((_, existing_kinds)) = paths
            .iter_mut()
            .find(|(existing_path, _)| existing_path == &library_path)
        {
            existing_kinds.push(*module_kind);
        } else {
            paths.push((library_path, vec![*module_kind]));
        }
    }
    paths
}

fn native_library_path_for_manifest(
    manifest_path: &Path,
    manifest: &PluginPackageManifest,
    module_kinds: &[PluginModuleKind],
) -> Option<PathBuf> {
    let package_root = manifest_path.parent()?;
    let file_name = native_library_file_name_for_manifest(manifest, module_kinds)?;
    let expected_library_path = package_root.join("native").join(file_name);
    Some(
        expected_library_path
            .canonicalize()
            .unwrap_or(expected_library_path),
    )
}

pub(crate) fn native_library_file_name_for_manifest(
    manifest: &PluginPackageManifest,
    module_kinds: &[PluginModuleKind],
) -> Option<String> {
    native_library_crate_name(manifest, module_kinds).map(dynamic_library_file_name)
}

fn native_library_crate_name<'a>(
    manifest: &'a PluginPackageManifest,
    module_kinds: &[PluginModuleKind],
) -> Option<&'a str> {
    if let Some(dist_crate) = distribution_dist_crate_name(manifest) {
        return Some(dist_crate);
    }
    for module_kind in module_kinds {
        if let Some(module) = manifest
            .modules
            .iter()
            .find(|module| module.kind == *module_kind)
        {
            return Some(module.crate_name.as_str());
        }
        if let Some(module) = manifest
            .feature_extensions
            .iter()
            .flat_map(|feature| feature.modules.iter())
            .find(|module| module.kind == *module_kind)
        {
            return Some(module.crate_name.as_str());
        }
    }
    None
}

fn distribution_dist_crate_name(manifest: &PluginPackageManifest) -> Option<&str> {
    let distribution = manifest.distribution.as_ref()?;
    if !distribution.forms.iter().any(|form| form == "dist") {
        return None;
    }
    let dist_crate = distribution.dist_crate.trim();
    (!dist_crate.is_empty()).then_some(dist_crate)
}

pub(super) fn resolve_manifest_path(export_root: &Path, manifest_path: &str) -> PathBuf {
    let path = PathBuf::from(manifest_path);
    if path.is_absolute() {
        path
    } else {
        export_root.join(path)
    }
}

#[cfg(test)]
#[path = "tests/candidate_from_manifest.rs"]
mod tests;
