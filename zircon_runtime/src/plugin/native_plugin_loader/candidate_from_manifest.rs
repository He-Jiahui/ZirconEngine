use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

use super::dynamic_library_name::dynamic_library_file_name;
use super::{NativePluginCandidate, NativePluginLoadReport};

pub(super) type NativePluginManifestCandidateResult<T> =
    std::result::Result<T, NativePluginManifestCandidateError>;

#[derive(Debug)]
pub(super) enum NativePluginManifestCandidateError {
    ReadManifest {
        manifest_path: PathBuf,
        source: io::Error,
    },
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
            Self::ReadManifest {
                manifest_path,
                source,
            } => write!(
                formatter,
                "failed to read native plugin manifest {}: {source}",
                manifest_path.display()
            ),
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
            Self::ReadManifest { source, .. } => Some(source),
            Self::ParseManifest { source, .. } => Some(source),
            Self::MissingRuntimeOrEditorModule { .. } => None,
        }
    }
}

pub(super) fn push_candidate_from_manifest_path(
    report: &mut NativePluginLoadReport,
    manifest_path: PathBuf,
) {
    match candidate_from_manifest_path(manifest_path) {
        Ok(candidate) => report.discovered.push(candidate),
        Err(error) => report.diagnostics.push(error.to_string()),
    }
}

fn candidate_from_manifest_path(
    manifest_path: PathBuf,
) -> NativePluginManifestCandidateResult<NativePluginCandidate> {
    let source = fs::read_to_string(&manifest_path).map_err(|source| {
        NativePluginManifestCandidateError::ReadManifest {
            manifest_path: manifest_path.clone(),
            source,
        }
    })?;
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
    let mut paths = Vec::<(PathBuf, Vec<PluginModuleKind>)>::new();
    for module_kind in module_kinds {
        let Some(crate_name) = native_library_crate_name(
            &candidate.package_manifest,
            std::slice::from_ref(module_kind),
        ) else {
            continue;
        };
        let library_path = package_root
            .join("native")
            .join(dynamic_library_file_name(crate_name));
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
    let crate_name = native_library_crate_name(manifest, module_kinds)?;
    let expected_library_path = package_root
        .join("native")
        .join(dynamic_library_file_name(crate_name));
    Some(
        expected_library_path
            .canonicalize()
            .unwrap_or(expected_library_path),
    )
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
mod tests {
    use std::path::PathBuf;

    use crate::plugin::{PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest};

    use super::*;

    #[test]
    fn native_library_path_prefers_distribution_dist_crate_for_runtime_modules() {
        let package_manifest = PluginPackageManifest::new("solari", "Solari")
            .with_runtime_module(PluginModuleManifest::runtime(
                "solari.runtime",
                "zircon_plugin_solari_runtime",
            ))
            .with_native_module(PluginModuleManifest::native(
                "solari.dist",
                "zircon_plugin_solari_dist",
            ))
            .with_distribution(PluginDistributionManifest {
                forms: vec!["dist".to_string()],
                dist_crate: "zircon_plugin_solari_dist".to_string(),
                ..PluginDistributionManifest::default()
            });
        let candidate = NativePluginCandidate {
            plugin_id: "solari".to_string(),
            package_manifest,
            manifest_path: PathBuf::from("plugins/solari/plugin.toml"),
            library_path: PathBuf::new(),
        };

        let paths = native_library_paths_for_candidate(&candidate, &[PluginModuleKind::Runtime]);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].1, vec![PluginModuleKind::Runtime]);
        assert_eq!(
            paths[0].0,
            PathBuf::from("plugins/solari/native")
                .join(dynamic_library_file_name("zircon_plugin_solari_dist"))
        );
    }

    #[test]
    fn candidate_from_manifest_path_reports_read_error_with_typed_source() {
        let missing_manifest = std::env::temp_dir().join(format!(
            "zircon-missing-native-plugin-manifest-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        let error = candidate_from_manifest_path(missing_manifest.clone())
            .expect_err("missing manifest should report typed candidate error");

        match error {
            NativePluginManifestCandidateError::ReadManifest { manifest_path, .. } => {
                assert_eq!(manifest_path, missing_manifest);
            }
            NativePluginManifestCandidateError::ParseManifest { .. }
            | NativePluginManifestCandidateError::MissingRuntimeOrEditorModule { .. } => {
                panic!("missing manifest should fail while reading manifest")
            }
        }
    }

    #[test]
    fn manifest_candidate_typed_error_preserves_missing_module_message() {
        let error = NativePluginManifestCandidateError::MissingRuntimeOrEditorModule {
            plugin_id: "plugin.without.module".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "native plugin plugin.without.module has no runtime or editor module crate declared"
        );
        assert!(
            std::error::Error::source(&error).is_none(),
            "missing-module error should not invent an IO or TOML source"
        );
    }
}
