use std::path::Path;

use super::candidate_from_manifest::push_candidate_from_manifest_path;
use super::collect_manifests::collect_plugin_manifests;
use super::{NativePluginLoadReport, NativePluginLoader};

impl NativePluginLoader {
    pub fn discover(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        let root = root.as_ref();
        let mut report = NativePluginLoadReport::default();
        if !root.exists() {
            report.diagnostics.push(format!(
                "native plugin root does not exist: {}",
                root.display()
            ));
            return report;
        }

        let mut manifest_paths = Vec::new();
        if let Err(error) = collect_plugin_manifests(root, &mut manifest_paths) {
            report.diagnostics.push(error);
            return report;
        }

        for manifest_path in manifest_paths {
            push_candidate_from_manifest_path(&mut report, manifest_path);
        }
        report
    }
}
