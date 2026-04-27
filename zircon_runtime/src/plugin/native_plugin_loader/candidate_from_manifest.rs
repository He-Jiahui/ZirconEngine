use std::fs;
use std::path::{Path, PathBuf};

use crate::{PluginModuleKind, PluginPackageManifest};

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
    let Some(package_root) = manifest_path.parent() else {
        report.diagnostics.push(format!(
            "native plugin manifest has no parent directory: {}",
            manifest_path.display()
        ));
        return;
    };
    let Some(crate_name) = native_library_crate_name(&manifest) else {
        report.diagnostics.push(format!(
            "native plugin {} has no runtime or editor module crate declared",
            manifest.id
        ));
        return;
    };
    let library_name = dynamic_library_file_name(crate_name);
    let library_path = package_root
        .join("native")
        .join(&library_name)
        .canonicalize()
        .unwrap_or_else(|_| package_root.join(&library_name));
    report.discovered.push(NativePluginCandidate {
        plugin_id: manifest.id.clone(),
        package_manifest: manifest,
        manifest_path,
        library_path,
    });
}

fn native_library_crate_name(manifest: &PluginPackageManifest) -> Option<&str> {
    manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .or_else(|| {
            manifest
                .modules
                .iter()
                .find(|module| module.kind == PluginModuleKind::Editor)
        })
        .map(|module| module.crate_name.as_str())
}

pub(super) fn resolve_manifest_path(export_root: &Path, manifest_path: &str) -> PathBuf {
    let path = PathBuf::from(manifest_path);
    if path.is_absolute() {
        path
    } else {
        export_root.join(path)
    }
}
