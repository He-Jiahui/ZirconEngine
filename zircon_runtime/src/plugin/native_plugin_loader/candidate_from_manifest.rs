use std::fs;
use std::path::{Path, PathBuf};

use crate::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

use super::dynamic_library_name::dynamic_library_file_name;
use super::{NativePluginCandidate, NativePluginLoadReport};

pub(super) fn push_candidate_from_manifest_path(
    report: &mut NativePluginLoadReport,
    manifest_path: PathBuf,
) {
    let source = match fs::read_to_string(&manifest_path) {
        Ok(source) => source,
        Err(error) => {
            report.diagnostics.push(format!(
                "failed to read native plugin manifest {}: {error}",
                manifest_path.display()
            ));
            return;
        }
    };
    let manifest = match toml::from_str::<PluginPackageManifest>(&source) {
        Ok(manifest) => manifest,
        Err(error) => {
            report.diagnostics.push(format!(
                "failed to parse native plugin manifest {}: {error}",
                manifest_path.display()
            ));
            return;
        }
    };
    let Some(library_path) = native_library_path_for_manifest(
        &manifest_path,
        &manifest,
        &[PluginModuleKind::Runtime, PluginModuleKind::Editor],
    ) else {
        report.diagnostics.push(format!(
            "native plugin {} has no runtime or editor module crate declared",
            manifest.id
        ));
        return;
    };
    report.discovered.push(NativePluginCandidate {
        plugin_id: manifest.id.clone(),
        package_manifest: manifest,
        manifest_path,
        library_path,
    });
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

pub(super) fn resolve_manifest_path(export_root: &Path, manifest_path: &str) -> PathBuf {
    let path = PathBuf::from(manifest_path);
    if path.is_absolute() {
        path
    } else {
        export_root.join(path)
    }
}
