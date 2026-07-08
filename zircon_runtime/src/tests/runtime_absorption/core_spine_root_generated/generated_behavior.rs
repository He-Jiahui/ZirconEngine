use std::collections::BTreeSet;
use std::path::PathBuf;

use super::source_helpers::read_source;

pub(super) struct GeneratedBehaviorLocation {
    pub(super) label: &'static str,
    pub(super) requires_migration: bool,
}

pub(super) fn generated_behavior_locations(paths: &[PathBuf]) -> Vec<GeneratedBehaviorLocation> {
    let mut locations = Vec::new();
    for path in paths {
        let source = read_source(path);
        for line in source.lines() {
            for label in generated_behavior_labels_for_line(line) {
                locations.push(GeneratedBehaviorLocation {
                    label,
                    requires_migration: generated_behavior_requires_migration(label, line),
                });
            }
        }
    }
    locations
}

fn generated_behavior_labels_for_line(line: &str) -> Vec<&'static str> {
    let mut labels = Vec::new();
    if line.contains("EntryRunner::")
        || line.contains("bootstrap_with_runtime_plugin")
        || line.contains("bootstrap_export_runtime")
    {
        labels.push("entry-bootstrap");
    }
    if line.contains("NativePluginLoader") || line.contains("load_runtime_from_load_manifest") {
        labels.push("native-loader");
    }
    if line.contains("plugin_registration()")
        || line.contains("plugin_feature_registration()")
        || line.contains("runtime_plugin_registrations()")
        || line.contains("runtime_plugin_feature_registrations()")
    {
        labels.push("plugin-registration");
    }
    if line.contains("pub fn target_mode(")
        || line.contains("pub fn export_profile(")
        || line.contains("pub fn project_plugins(")
    {
        labels.push("runtime-selection-function");
    }
    if line.contains("fn main()") {
        labels.push("generated-main");
    }
    labels
}

fn generated_behavior_requires_migration(label: &str, line: &str) -> bool {
    match label {
        "entry-bootstrap" => {
            line.contains("EntryRunner::") || line.contains("bootstrap_with_runtime_plugin")
        }
        "native-loader" | "plugin-registration" => true,
        "generated-main" | "runtime-selection-function" => false,
        _ => true,
    }
}

pub(super) fn behavior_labels(locations: &[GeneratedBehaviorLocation]) -> BTreeSet<&'static str> {
    locations.iter().map(|location| location.label).collect()
}
