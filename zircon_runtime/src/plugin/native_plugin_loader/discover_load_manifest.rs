use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::candidate_from_manifest::{push_candidate_from_manifest_path, resolve_manifest_path};
use super::{
    NativePluginCandidate, NativePluginLoadManifest, NativePluginLoadManifestEntry,
    NativePluginLoadReport, NativePluginLoader,
};

const NATIVE_PLUGIN_LOAD_MANIFEST_PATH: &str = "plugins/native_plugins.toml";

impl NativePluginLoader {
    pub fn discover_from_load_manifest(
        &self,
        export_root: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        let export_root = export_root.as_ref();
        let load_manifest_path = export_root.join(NATIVE_PLUGIN_LOAD_MANIFEST_PATH);
        let mut report = NativePluginLoadReport::default();
        let source = match fs::read_to_string(&load_manifest_path) {
            Ok(source) => source,
            Err(error) => {
                report.diagnostics.push(format!(
                    "failed to read native plugin load manifest {}: {error}",
                    load_manifest_path.display()
                ));
                return report;
            }
        };
        let load_manifest = match toml::from_str::<NativePluginLoadManifest>(&source) {
            Ok(load_manifest) => load_manifest,
            Err(error) => {
                report.diagnostics.push(format!(
                    "failed to parse native plugin load manifest {}: {error}",
                    load_manifest_path.display()
                ));
                return report;
            }
        };
        let mut discovered_package_ids = BTreeSet::new();
        for entry in load_manifest.plugins {
            let Some(manifest_path) = resolve_load_manifest_entry_path(
                &mut report,
                export_root,
                &entry,
                "manifest",
                &entry.manifest,
            ) else {
                continue;
            };
            let candidate_index = report.discovered.len();
            push_candidate_from_manifest_path(&mut report, manifest_path);
            if let Some(candidate) = report.discovered.get(candidate_index).cloned() {
                let keep_candidate =
                    validate_load_manifest_entry(&mut report, export_root, &entry, &candidate);
                if !keep_candidate {
                    report.discovered.remove(candidate_index);
                    continue;
                }
                if !discovered_package_ids.insert(candidate.package_manifest.id.clone()) {
                    report.discovered.remove(candidate_index);
                    report.diagnostics.push(format!(
                        "native plugin {} load manifest duplicate package id ignored",
                        candidate.package_manifest.id
                    ));
                }
            }
        }
        report
    }

    pub fn load_all_from_load_manifest(
        &self,
        export_root: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        let report = self.discover_from_load_manifest(export_root);
        self.load_all_candidates(report)
    }

    pub fn load_runtime_from_load_manifest(
        &self,
        export_root: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        let report = self.discover_from_load_manifest(export_root);
        self.load_candidates_for_module_kinds(report, &[crate::PluginModuleKind::Runtime])
    }

    pub fn load_editor_from_load_manifest(
        &self,
        export_root: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        let report = self.discover_from_load_manifest(export_root);
        self.load_candidates_for_module_kinds(report, &[crate::PluginModuleKind::Editor])
    }
}

fn validate_load_manifest_entry(
    report: &mut NativePluginLoadReport,
    export_root: &Path,
    entry: &NativePluginLoadManifestEntry,
    candidate: &NativePluginCandidate,
) -> bool {
    if entry.id != candidate.package_manifest.id {
        report.diagnostics.push(format!(
            "native plugin {} load manifest id mismatch: entry id {}",
            candidate.package_manifest.id, entry.id
        ));
    }

    let Some(package_path) =
        resolve_load_manifest_entry_path(report, export_root, entry, "path", &entry.path)
    else {
        return false;
    };
    let package_path = canonical_or_normalized(package_path);
    let manifest_parent = candidate
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| candidate.manifest_path.clone());
    let manifest_parent = canonical_or_normalized(manifest_parent);
    if !manifest_parent.starts_with(&package_path) {
        report.diagnostics.push(format!(
            "native plugin {} load manifest path mismatch: manifest {} is outside package path {}",
            candidate.package_manifest.id,
            candidate.manifest_path.display(),
            package_path.display()
        ));
    }
    true
}

fn resolve_load_manifest_entry_path(
    report: &mut NativePluginLoadReport,
    export_root: &Path,
    entry: &NativePluginLoadManifestEntry,
    field_name: &str,
    field_path: &str,
) -> Option<PathBuf> {
    let path = resolve_manifest_path(export_root, field_path);
    let normalized_export_root = canonical_or_normalized(export_root.to_path_buf());
    let normalized_path = canonical_or_normalized(path.clone());
    if !normalized_path.starts_with(&normalized_export_root) {
        report.diagnostics.push(format!(
            "native plugin {} load manifest {} escapes export root: {}",
            entry.id, field_name, field_path
        ));
        return None;
    }
    Some(path)
}

fn canonical_or_normalized(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| normalize_path(path))
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}
