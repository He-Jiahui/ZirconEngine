use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::scope::export_template_files;
use super::support::{relative_path, trimmed_snippet};

#[derive(Clone, Copy, Debug)]
struct GeneratedBehaviorDecision {
    label: &'static str,
    classification: &'static str,
    target_owner: &'static str,
    allowed_generated_shape: &'static str,
}

const GENERATED_BEHAVIOR_DECISIONS: &[GeneratedBehaviorDecision] = &[
    GeneratedBehaviorDecision {
        label: "entry-bootstrap",
        classification: "handwritten-owner-required",
        target_owner: "handwritten export bootstrap owner",
        allowed_generated_shape: "thin call into one stable export bootstrap facade",
    },
    GeneratedBehaviorDecision {
        label: "generated-main",
        classification: "entry-glue-review",
        target_owner: "handwritten export entry facade",
        allowed_generated_shape: "minimal main function forwarding to a stable facade",
    },
    GeneratedBehaviorDecision {
        label: "native-loader",
        classification: "native-loader-isolation",
        target_owner: "isolated native plugin loader namespace",
        allowed_generated_shape: "native manifest data or isolated loader facade call",
    },
    GeneratedBehaviorDecision {
        label: "plugin-registration",
        classification: "handwritten-owner-required",
        target_owner: "runtime-owned plugin catalog or export provider owner",
        allowed_generated_shape: "provider ids, manifest rows, or registration tables",
    },
    GeneratedBehaviorDecision {
        label: "runtime-selection-function",
        classification: "data-adapter-review",
        target_owner: "generated data table adapter",
        allowed_generated_shape: "pure DTO/table functions without lifecycle side effects",
    },
];

#[derive(Debug)]
struct GeneratedBehaviorLocation {
    label: &'static str,
    path: String,
    line: usize,
    snippet: String,
}

#[test]
fn export_template_generated_behavior_stays_classified_by_owner() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let behavior_locations = collect_export_template_behavior_locations(manifest_root);
    let decisions = generated_behavior_decisions_by_label();

    let unclassified = behavior_locations
        .iter()
        .filter(|location| !decisions.contains_key(location.label))
        .map(|location| {
            format!(
                "{}:{} [{}]: {}",
                location.path, location.line, location.label, location.snippet
            )
        })
        .collect::<Vec<_>>();
    assert!(
        unclassified.is_empty(),
        "export generated behavior labels must be classified before accepting the boundary:\n{}",
        unclassified.join("\n")
    );

    let used_labels = behavior_locations
        .iter()
        .map(|location| location.label)
        .collect::<BTreeSet<_>>();
    for label in used_labels {
        let decision = decisions
            .get(label)
            .expect("used generated behavior label should be classified");
        assert!(!decision.classification.is_empty());
        assert!(!decision.target_owner.is_empty());
        assert!(!decision.allowed_generated_shape.is_empty());
    }
}

#[test]
fn export_template_generated_behavior_is_adapter_only_after_m4_cutover() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let behavior_locations = collect_export_template_behavior_locations(manifest_root);

    let migration_debt = behavior_locations
        .iter()
        .filter(|location| generated_behavior_location_requires_migration(location))
        .map(|location| {
            format!(
                "{}:{} [{}]: {}",
                location.path, location.line, location.label, location.snippet
            )
        })
        .collect::<Vec<_>>();

    assert!(
        migration_debt.is_empty(),
        "export generated behavior must stay limited to allowed adapters after the M4 cutover:\n{}",
        migration_debt.join("\n")
    );
}

fn collect_export_template_behavior_locations(
    manifest_root: &Path,
) -> Vec<GeneratedBehaviorLocation> {
    let mut locations = Vec::new();
    for path in export_template_files(manifest_root) {
        let relative = relative_path(manifest_root, &path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        for (line_index, line) in source.lines().enumerate() {
            for label in generated_behavior_labels_for_line(line) {
                locations.push(GeneratedBehaviorLocation {
                    label,
                    path: relative.clone(),
                    line: line_index + 1,
                    snippet: trimmed_snippet(line),
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

fn generated_behavior_location_requires_migration(location: &GeneratedBehaviorLocation) -> bool {
    match location.label {
        "entry-bootstrap" => {
            location.snippet.contains("EntryRunner::")
                || location.snippet.contains("bootstrap_with_runtime_plugin")
        }
        "native-loader" | "plugin-registration" => true,
        "generated-main" | "runtime-selection-function" => false,
        _ => true,
    }
}

fn generated_behavior_decisions_by_label(
) -> BTreeMap<&'static str, &'static GeneratedBehaviorDecision> {
    GENERATED_BEHAVIOR_DECISIONS
        .iter()
        .map(|decision| (decision.label, decision))
        .collect()
}
