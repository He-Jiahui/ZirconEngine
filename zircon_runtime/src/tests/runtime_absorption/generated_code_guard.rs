use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const GENERATED_MARKER_PREFIX: &str = "// @generated ";
const GENERATED_MARKER_SUFFIX: &str = " - do not edit by hand";

const MARKED_GENERATED_FORBIDDEN_TOKENS: &[&str] = &[
    "impl ",
    "fn ",
    "match ",
    "for ",
    "while ",
    "if let ",
    "EntryRunner::",
    "NativePluginLoader",
    "runtime_plugin_registrations",
    "plugin_registration()",
    "SceneSchedule",
    "CoreRuntime",
];

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
fn generated_marker_format_is_uniform_when_source_files_are_marked() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);

    let mut invalid_markers = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, &path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        for (line_index, line) in source.lines().enumerate() {
            if !line.trim_start().starts_with(GENERATED_MARKER_PREFIX) {
                continue;
            }
            let is_valid_first_line = line_index == 0
                && line.starts_with(GENERATED_MARKER_PREFIX)
                && line.ends_with(GENERATED_MARKER_SUFFIX)
                && line.len() > GENERATED_MARKER_PREFIX.len() + GENERATED_MARKER_SUFFIX.len();
            if !is_valid_first_line {
                invalid_markers.push(format!("{}:{}: {}", relative, line_index + 1, line.trim()));
            }
        }
    }

    assert!(
        invalid_markers.is_empty(),
        "generated source markers must use `{GENERATED_MARKER_PREFIX}<generator>{GENERATED_MARKER_SUFFIX}` on line 1:\n{}",
        invalid_markers.join("\n")
    );
}

#[test]
fn marked_generated_source_files_stay_leaf_data_only() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);

    let mut behavior_locations = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, &path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        let Some(first_line) = source.lines().next() else {
            continue;
        };
        if !first_line.starts_with(GENERATED_MARKER_PREFIX) {
            continue;
        }
        for (line_index, line) in source.lines().enumerate().skip(1) {
            if let Some(token) = MARKED_GENERATED_FORBIDDEN_TOKENS
                .iter()
                .find(|token| line.contains(**token))
            {
                behavior_locations.push(format!(
                    "{}:{}: `{}` in {}",
                    relative,
                    line_index + 1,
                    token,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        behavior_locations.is_empty(),
        "generated source files must stay leaf data/DTO/table artifacts and cannot own behavior:\n{}",
        behavior_locations.join("\n")
    );
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

#[test]
fn export_template_scan_scope_stays_folder_backed() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let template_files = export_template_files(manifest_root);
    let relative_files = template_files
        .iter()
        .map(|path| relative_path(manifest_root, path))
        .collect::<Vec<_>>();

    assert!(
        !relative_files.is_empty(),
        "generated-code guard should scan export template files"
    );
    assert!(
        relative_files
            .iter()
            .all(|path| path.starts_with("src/plugin/export_build_plan/")),
        "generated-code guard must stay scoped to export_build_plan templates:\n{}",
        relative_files.join("\n")
    );
}

#[test]
fn export_entry_templates_delegate_to_app_export_bootstrap_facade() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let entry_template_paths = [
        manifest_root
            .join("src")
            .join("plugin")
            .join("export_build_plan")
            .join("main_template.rs"),
        manifest_root
            .join("src")
            .join("plugin")
            .join("export_build_plan")
            .join("platform_host_files.rs"),
    ];
    let forbidden_entry_tokens = [
        "EntryRunner::",
        "EntryConfig::new",
        "NativePluginLoader",
        "load_runtime_from_load_manifest",
        "zircon_plugins::runtime_plugin_registrations()",
        "zircon_plugins::runtime_plugin_feature_registrations()",
    ];
    let mut violations = Vec::new();
    let mut combined_source = String::new();

    for path in entry_template_paths {
        let relative = relative_path(manifest_root, &path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        combined_source.push_str(&source);
        combined_source.push('\n');

        for (line_index, line) in source.lines().enumerate() {
            for token in forbidden_entry_tokens {
                if line.contains(token) {
                    violations.push(format!(
                        "{}:{}: `{}` in {}",
                        relative,
                        line_index + 1,
                        token,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "generated entry templates must call the handwritten app export bootstrap facade instead of owning startup/native-loader behavior:\n{}",
        violations.join("\n")
    );
    for required in [
        "zircon_app::bootstrap_export_runtime",
        "zircon_app::bootstrap_export_runtime_with_native_plugins_from_export_root",
        "zircon_app::discover_export_root()?",
        "zircon_plugins::export_runtime_bootstrap_config()",
    ] {
        assert!(
            combined_source.contains(required),
            "generated entry templates should keep the thin export-bootstrap facade call `{required}`"
        );
    }
}

#[test]
fn export_plugin_selection_template_delegates_registration_execution_to_app_providers() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let template_path = manifest_root
        .join("src")
        .join("plugin")
        .join("export_build_plan")
        .join("plugin_selection_template.rs");
    let source = fs::read_to_string(&template_path).unwrap_or_else(|error| {
        panic!(
            "failed to read {}: {error}",
            relative_path(manifest_root, &template_path)
        )
    });

    for forbidden in ["plugin_registration()", "plugin_feature_registration()"] {
        assert!(
            !source.contains(forbidden),
            "plugin selection templates must pass registration providers to the app facade instead of directly calling `{forbidden}`"
        );
    }
    for required in [
        "ExportRuntimePluginRegistrationProvider::new",
        "ExportRuntimePluginFeatureRegistrationProvider::new",
        ".with_runtime_plugin_registration_providers(runtime_plugin_registration_providers())",
        ".with_runtime_plugin_feature_registration_providers(runtime_plugin_feature_registration_providers())",
    ] {
        assert!(
            source.contains(required),
            "plugin selection templates should keep provider-table handoff `{required}`"
        );
    }
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

fn export_template_files(manifest_root: &Path) -> Vec<PathBuf> {
    let export_root = manifest_root
        .join("src")
        .join("plugin")
        .join("export_build_plan");
    let mut files = Vec::new();
    collect_rust_source_files(&export_root, &mut files);
    files.retain(|path| is_export_template_file(&export_root, path));
    files.sort();
    files
}

fn is_export_template_file(export_root: &Path, path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let relative = path
        .strip_prefix(export_root)
        .expect("export template path should live under export root")
        .to_string_lossy()
        .replace('\\', "/");

    file_name.contains("template")
        || file_name == "generated_files.rs"
        || file_name == "platform_host_files.rs"
        || relative.starts_with("platform_host_files/")
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(root).unwrap_or_else(|error| {
        panic!(
            "failed to read source directory {}: {error}",
            root.display()
        )
    }) {
        let entry = entry.expect("source directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("source path should live under manifest root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn trimmed_snippet(line: &str) -> String {
    const MAX_SNIPPET_LEN: usize = 220;
    let trimmed = line.trim();
    if trimmed.len() <= MAX_SNIPPET_LEN {
        trimmed.to_string()
    } else {
        format!("{}...", trimmed[..MAX_SNIPPET_LEN].trim_end())
    }
}
