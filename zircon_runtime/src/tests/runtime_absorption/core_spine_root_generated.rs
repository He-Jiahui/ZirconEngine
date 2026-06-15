use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_CORE_ROOT_ENTRIES: &[&str] = &[
    "framework",
    "manager",
    "math",
    "mod.rs",
    "resource",
    "runtime",
];

const EXPECTED_CORE_PUBLIC_MODULES: &[&str] =
    &["runtime", "framework", "manager", "math", "resource"];

const RETIRED_CORE_ROOT_ENTRIES: &[&str] = &[
    "channel_util.rs",
    "config_store.rs",
    "diagnostics",
    "event_bus",
    "event_bus.rs",
    "frame_clock.rs",
    "job_scheduler.rs",
    "lifecycle.rs",
    "modules",
    "state",
    "tasks",
    "time.rs",
    "types.rs",
];

const EXPECTED_RUNTIME_02_GUARD_TEST_ANCHORS: &[(&str, &[&str])] = &[
    (
        "zircon_runtime/src/tests/runtime_absorption/root_entries.rs",
        &[
            "core_root_retires_channel_and_service_alias_fragments",
            "core_root_retires_runtime_kernel_fragment_files",
            "core_root_splits_event_dto_from_runtime_event_bus",
            "core_root_reexports_runtime_diagnostics_without_root_directory",
            "core_module_tree_matches_decided_spine_shape",
            "runtime_crate_root_does_not_flatten_plugin_surface",
            "runtime_crate_root_does_not_flatten_builtin_module_assembly_functions",
            "builtin_root_stays_structural_after_runtime_module_split",
        ],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_surface.rs",
        &[
            "runtime_crate_root_public_surface_stays_curated",
            "graphics_alias_debt_is_private_and_documented_until_m3_cutover",
            "graphics_type_alias_debt_has_m3_2_pre_guard_until_render_cutover",
            "core_spine_and_root_surface_docs_stay_in_sync",
            "root_surface_m1_gate_matches_runtime_14_module_family_seats",
            "root_surface_interface_convergence_mirror_uses_current_audit_counts",
        ],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs",
        &[
            "generated_marker_format_is_uniform_when_source_files_are_marked",
            "marked_generated_source_files_stay_leaf_data_only",
            "export_template_generated_behavior_stays_classified_by_owner",
            "export_template_generated_behavior_is_adapter_only_after_m4_cutover",
            "export_template_scan_scope_stays_folder_backed",
            "export_entry_templates_delegate_to_app_export_bootstrap_facade",
            "export_plugin_selection_template_delegates_registration_execution_to_app_providers",
        ],
    ),
];

const MIRROR_DOCS: &[(&str, &str)] = &[
    (
        "Runtime 02 plan",
        include_str!(
            "../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
        ),
    ),
    (
        "runtime index",
        include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
    ),
    (
        "root surface doc",
        include_str!("../../../../docs/zircon_runtime/core/root_surface.md"),
    ),
    (
        "generated-code boundary",
        include_str!("../../../../docs/engine-architecture/generated-code-boundary.md"),
    ),
    (
        "interface convergence",
        include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
    ),
    (
        "M0 review",
        include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
    ),
];

#[test]
fn runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = runtime_root
        .parent()
        .expect("zircon_runtime manifest should be inside the workspace root");

    assert_eq!(
        core_root_entries(runtime_root),
        string_set(EXPECTED_CORE_ROOT_ENTRIES),
        "core root entries changed; update core_spine_root_generated_boundary and Runtime 02 mirror docs"
    );
    assert_eq!(
        public_modules(&runtime_root.join("src").join("core").join("mod.rs")),
        string_vec(EXPECTED_CORE_PUBLIC_MODULES),
        "core public module declarations changed without Runtime 02 audit sync"
    );
    for retired_entry in RETIRED_CORE_ROOT_ENTRIES {
        assert!(
            !runtime_root
                .join("src")
                .join("core")
                .join(retired_entry)
                .exists(),
            "retired core root entry `{retired_entry}` reappeared"
        );
    }

    let crate_root = runtime_root.join("src").join("lib.rs");
    assert_eq!(
        public_modules(&crate_root).len(),
        20,
        "runtime root public module count changed without Runtime 02 audit sync"
    );
    assert_eq!(
        public_use_count(&crate_root),
        3,
        "runtime root public `pub use` count changed without Runtime 02 audit sync"
    );
    assert_eq!(
        crate_visible_graphics_reexport_count(&crate_root),
        80,
        "crate-visible graphics alias debt count changed without Runtime 02 audit sync"
    );

    let export_root = runtime_root
        .join("src")
        .join("plugin")
        .join("export_build_plan");
    let template_files = export_template_files(&export_root);
    assert_eq!(
        template_files.len(),
        10,
        "generated export template count changed without Runtime 02 audit sync"
    );
    let behavior_locations = generated_behavior_locations(&template_files);
    assert_eq!(
        behavior_locations.len(),
        6,
        "generated behavior location count changed without Runtime 02 audit sync"
    );
    assert_eq!(
        behavior_locations
            .iter()
            .filter(|location| location.requires_migration)
            .count(),
        0,
        "generated behavior migration debt reappeared in export templates"
    );
    assert_eq!(
        behavior_labels(&behavior_locations).len(),
        3,
        "generated behavior decision count changed without Runtime 02 audit sync"
    );

    assert_eq!(
        rust_test_count(&runtime_root.join("src/tests/runtime_absorption/root_entries.rs")),
        13,
        "root_entries guard test count changed without Runtime 02 audit sync"
    );
    assert_eq!(
        rust_test_count(&runtime_root.join("src/tests/runtime_absorption/root_surface.rs")),
        6,
        "root_surface guard test count changed without Runtime 02 audit sync"
    );
    assert_eq!(
        rust_test_count(&runtime_root.join("src/tests/runtime_absorption/generated_code_guard.rs")),
        7,
        "generated-code guard test count changed without Runtime 02 audit sync"
    );
    assert_runtime_02_guard_test_anchors(workspace_root);

    for relative in [
        "zircon_runtime/src/tests/runtime_absorption/root_entries.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_surface.rs",
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs",
        "zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs",
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py",
    ] {
        assert!(
            workspace_root.join(relative).exists(),
            "Runtime 02 audit source `{relative}` is missing"
        );
    }

    for (doc_name, doc_source) in MIRROR_DOCS {
        for required_anchor in [
            "core_spine_root_generated_boundary",
            "core root entries 6/6",
            "core public modules 5/5",
            "retired core root entries 0",
            "runtime root public modules 20/20",
            "public `pub use` sites 3/3",
            "crate-visible graphics alias debt 80/80",
            "root-surface M1 gate `migration-debt-present`",
            "generated export templates 10/10",
            "generated behavior 6/6",
            "generated allowed adapters 6/6",
            "generated migration debt 0/0",
            "generated-code M1 gate `classified-and-clear`",
            "root_entries guard tests 13",
            "root_surface guard tests 6/6",
            "generated-code guard tests 7/7",
            "guard_test_anchor_count = 21",
            "missing_guard_test_anchors = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 02 core/root/generated audit anchor `{required_anchor}`"
            );
        }
    }
}

fn assert_runtime_02_guard_test_anchors(workspace_root: &Path) {
    let mut guard_test_anchor_count = 0;
    for (relative_file, expected_anchors) in EXPECTED_RUNTIME_02_GUARD_TEST_ANCHORS {
        let source = read_source(&workspace_root.join(relative_file));
        for expected_anchor in *expected_anchors {
            guard_test_anchor_count += 1;
            assert!(
                source.contains(expected_anchor),
                "`{relative_file}` should keep Runtime 02 guard test anchor `{expected_anchor}`"
            );
        }
    }
    assert_eq!(
        guard_test_anchor_count, 21,
        "Runtime 02 guard test anchor inventory should stay at 21 anchors"
    );
}

struct GeneratedBehaviorLocation {
    label: &'static str,
    requires_migration: bool,
}

fn core_root_entries(runtime_root: &Path) -> BTreeSet<String> {
    fs::read_dir(runtime_root.join("src").join("core"))
        .unwrap_or_else(|error| panic!("failed to read core root: {error}"))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read core root entry: {error}"))
                .file_name()
                .into_string()
                .unwrap_or_else(|name| panic!("non-utf8 core root entry: {name:?}"))
        })
        .collect()
}

fn public_modules(path: &Path) -> Vec<String> {
    let source = read_source(path);
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|decl| decl.trim_end_matches(';'))
        .map(String::from)
        .collect()
}

fn public_use_count(path: &Path) -> usize {
    let source = read_source(path);
    source
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use "))
        .count()
}

fn crate_visible_graphics_reexport_count(path: &Path) -> usize {
    let source = read_source(path);
    let start_marker = "pub(crate) use graphics::{";
    let start = source
        .find(start_marker)
        .expect("runtime root should keep the current crate-visible graphics alias block");
    let body_start = start + start_marker.len();
    let body_end = source[body_start..]
        .find("};")
        .map(|offset| body_start + offset)
        .expect("crate-visible graphics alias block should terminate");
    source[body_start..body_end]
        .replace('\n', " ")
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .count()
}

fn export_template_files(export_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(export_root, &mut files);
    files.retain(|path| is_export_template_file(export_root, path));
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
        .expect("template path should live below export_build_plan")
        .to_string_lossy()
        .replace('\\', "/");

    file_name.contains("template")
        || file_name == "generated_files.rs"
        || file_name == "platform_host_files.rs"
        || relative.starts_with("platform_host_files/")
}

fn generated_behavior_locations(paths: &[PathBuf]) -> Vec<GeneratedBehaviorLocation> {
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

fn behavior_labels(locations: &[GeneratedBehaviorLocation]) -> BTreeSet<&'static str> {
    locations.iter().map(|location| location.label).collect()
}

fn rust_test_count(path: &Path) -> usize {
    let source = read_source(path);
    source.matches("#[test]").count()
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
    {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn string_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().copied().map(String::from).collect()
}

fn string_vec(values: &[&str]) -> Vec<String> {
    values.iter().copied().map(String::from).collect()
}
