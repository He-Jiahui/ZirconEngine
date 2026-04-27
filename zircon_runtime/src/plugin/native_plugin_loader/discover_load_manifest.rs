use std::fs;
use std::path::Path;

use super::candidate_from_manifest::{push_candidate_from_manifest_path, resolve_manifest_path};
use super::{NativePluginLoadManifest, NativePluginLoadReport, NativePluginLoader};

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
        for entry in load_manifest.plugins {
            let manifest_path = resolve_manifest_path(export_root, &entry.manifest);
            push_candidate_from_manifest_path(&mut report, manifest_path);
        }
        report
    }

    pub fn load_from_load_manifest(&self, export_root: impl AsRef<Path>) -> NativePluginLoadReport {
        let report = self.discover_from_load_manifest(export_root);
        self.load_candidates(report)
    }
}
