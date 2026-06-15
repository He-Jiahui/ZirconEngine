use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest should live under the repository root")
        .to_path_buf()
}

fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn read_repo_file(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn top_level_entry_names(relative: &str, include_root_mod: bool) -> Vec<String> {
    let dir = repo_path(relative);
    let mut entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| {
            let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
            entry.file_name().into_string().unwrap_or_else(|name| {
                panic!("non-utf8 filename under {}: {name:?}", dir.display())
            })
        })
        .filter(|name| include_root_mod || name != "mod.rs")
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn rust_files_under(relative: &str) -> Vec<PathBuf> {
    let mut pending = vec![repo_path(relative)];
    let mut files = Vec::new();

    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        {
            let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

fn has_component(path: &Path, component: &str) -> bool {
    let component = OsStr::new(component);
    path.components()
        .any(|path_component| path_component.as_os_str() == component)
}

fn production_ui_file(path: &Path) -> bool {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    !has_component(path, "tests")
        && !has_component(path, "test_fixtures")
        && filename != "tests.rs"
        && !filename.ends_with("_tests.rs")
}

fn matching_line_count(files: &[PathBuf], needle: &str) -> usize {
    files
        .iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .filter(|line| line.contains(needle))
                .count()
        })
        .sum()
}

fn files_with_matching_line(files: &[PathBuf], needle: &str) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .any(|line| line.contains(needle))
        })
        .cloned()
        .collect()
}

#[test]
fn runtime_09_ui_architecture_doc_records_current_boundaries() {
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");

    for required_anchor in [
        "runtime_09_m0_ui_architecture_static_passed",
        "Module Boundary Map",
        "`ui/` top-level entries: 17",
        "`surface/` entries: 20",
        "No M0 blocker-level owner inversion",
        "completed_static_passed",
    ] {
        assert!(
            architecture_doc.contains(required_anchor)
                || runtime_09_plan.contains(required_anchor)
                || runtime_index.contains(required_anchor),
            "Runtime 09 M0 docs/index should retain boundary anchor `{required_anchor}`"
        );
    }

    let ui_entries = top_level_entry_names("zircon_runtime/src/ui", false);
    assert_eq!(
        ui_entries.len(),
        17,
        "Runtime 09 M0 architecture doc must be refreshed when ui/ top-level entries change"
    );
    for required_entry in [
        "accessibility",
        "binding",
        "component",
        "dispatch",
        "event_ui",
        "icon_atlas",
        "layout",
        "module.rs",
        "runtime_ui",
        "style.rs",
        "surface",
        "template",
        "tests",
        "text",
        "theme",
        "tree",
        "v2",
    ] {
        assert!(
            ui_entries.iter().any(|entry| entry == required_entry),
            "Runtime 09 UI top-level map should include `{required_entry}`"
        );
    }

    let surface_entries = top_level_entry_names("zircon_runtime/src/ui/surface", true);
    assert_eq!(
        surface_entries.len(),
        20,
        "Runtime 09 M0 architecture doc must be refreshed when surface/ entries change"
    );
    for required_entry in [
        "input",
        "pointer",
        "navigation",
        "render",
        "surface.rs",
        "mod.rs",
    ] {
        assert!(
            surface_entries.iter().any(|entry| entry == required_entry),
            "Runtime 09 surface map should include `{required_entry}`"
        );
    }
}

#[test]
fn runtime_09_ui_architecture_baselines_match_current_source_scan() {
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let all_ui_files = rust_files_under("zircon_runtime/src/ui");
    let production_ui_files = all_ui_files
        .iter()
        .filter(|path| production_ui_file(path))
        .cloned()
        .collect::<Vec<_>>();

    let legacy_full_hits = matching_line_count(&all_ui_files, "legacy");
    let legacy_production_hits = matching_line_count(&production_ui_files, "legacy");
    let legacy_production_files = files_with_matching_line(&production_ui_files, "legacy");
    let taffy_production_hits = matching_line_count(&production_ui_files, "taffy");
    let taffy_production_files = files_with_matching_line(&production_ui_files, "taffy");

    assert_eq!(
        legacy_full_hits, 167,
        "update Runtime 09 docs if full legacy baseline changes"
    );
    assert_eq!(
        legacy_production_hits, 102,
        "update Runtime 09 docs if production legacy hit baseline changes"
    );
    assert_eq!(
        legacy_production_files.len(),
        12,
        "update Runtime 09 docs if production legacy file baseline changes"
    );
    assert_eq!(
        taffy_production_hits, 161,
        "update Runtime 09 docs if production taffy hit baseline changes"
    );
    assert_eq!(
        taffy_production_files.len(),
        7,
        "update Runtime 09 docs if production taffy file baseline changes"
    );

    for required_anchor in [
        "ui_legacy_hits=167",
        "ui_legacy_production_hits=102",
        "ui_legacy_production_files=12",
        "ui_taffy_production_hits=161",
        "ui_taffy_production_files=7",
    ] {
        assert!(
            architecture_doc.contains(required_anchor)
                || runtime_09_plan.contains(required_anchor)
                || runtime_index.contains(required_anchor),
            "Runtime 09 docs/index should retain source-scan baseline `{required_anchor}`"
        );
    }
}

#[test]
fn runtime_09_v2_verdict_matches_runtime_and_interface_modules() {
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_v2_mod = read_repo_file("zircon_runtime/src/ui/v2/mod.rs");
    let interface_v2_mod = read_repo_file("zircon_runtime_interface/src/ui/v2/mod.rs");

    for required_runtime_anchor in [
        "mod cache;",
        "mod compiler;",
        "mod file_cache;",
        "mod loader;",
        "mod style;",
        "mod surface_builder;",
        "mod surface_tree;",
        "UiV2PrototypeStoreFileCache",
        "UiV2SurfaceBuilder",
        "UiZuiAssetLoader",
    ] {
        assert!(
            runtime_v2_mod.contains(required_runtime_anchor),
            "runtime ui::v2 module should retain `{required_runtime_anchor}`"
        );
    }

    for required_interface_anchor in [
        "mod arena;",
        "mod asset;",
        "mod compiled;",
        "mod graph;",
        "mod repeat;",
        "mod style;",
        "UiV2AssetDocument",
        "UiV2CompiledDocument",
        "UiV2ResolvedStyle",
    ] {
        assert!(
            interface_v2_mod.contains(required_interface_anchor),
            "interface ui::v2 module should retain `{required_interface_anchor}`"
        );
    }

    for required_verdict_anchor in [
        "v2-replacement-mainline",
        ".zui",
        ".v2.ui.toml",
        "replacement mainline",
        "migration/test-only",
        "old recursive template",
    ] {
        assert!(
            architecture_doc.contains(required_verdict_anchor)
                || runtime_09_plan.contains(required_verdict_anchor)
                || runtime_index.contains(required_verdict_anchor),
            "Runtime 09 docs/index should retain v2 verdict anchor `{required_verdict_anchor}`"
        );
    }
}

#[test]
fn runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts() {
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let architecture_review =
        include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let interface_doc =
        include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let audit_script = include_str!(
        "../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py"
    );
    let ui_guard = include_str!("ui_architecture.rs");
    let cargo_gate_guard = include_str!("plan_status/cargo_gates/middle.rs");

    for guard_anchor in [
        "runtime_09_ui_architecture_doc_records_current_boundaries",
        "runtime_09_ui_architecture_baselines_match_current_source_scan",
        "runtime_09_v2_verdict_matches_runtime_and_interface_modules",
        "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
        "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
    ] {
        assert!(
            ui_guard.contains(guard_anchor) || cargo_gate_guard.contains(guard_anchor),
            "Runtime 09 guard anchor `{guard_anchor}` should stay visible to ui_architecture_boundary"
        );
    }

    for audit_anchor in [
        "EXPECTED_SOURCE_FILE_COUNT = 11",
        "EXPECTED_UI_ENTRY_COUNT = 17",
        "EXPECTED_SURFACE_ENTRY_COUNT = 20",
        "EXPECTED_LEGACY_FULL_HITS = 167",
        "EXPECTED_LEGACY_PRODUCTION_HITS = 102",
        "EXPECTED_LEGACY_PRODUCTION_FILE_COUNT = 12",
        "EXPECTED_TAFFY_PRODUCTION_HITS = 161",
        "EXPECTED_TAFFY_PRODUCTION_FILE_COUNT = 7",
        "MIRROR_DOCS_GUARD",
        "\"runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts\"",
        "\"mirror_docs_guard_present\"",
    ] {
        assert!(
            audit_script.contains(audit_anchor),
            "ui_architecture_boundary should expose audit anchor `{audit_anchor}`"
        );
    }

    let mirror_docs = [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("runtime architecture review", architecture_review),
        ("runtime interface convergence doc", interface_doc),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for expected_anchor in [
            "ui_architecture_boundary",
            "expected_source_file_count = 11",
            "expected_ui_entry_count = 17",
            "expected_surface_entry_count = 20",
            "legacy_full_hits = 167",
            "expected_legacy_full_hits = 167",
            "legacy_production_hits = 102",
            "expected_legacy_production_hits = 102",
            "legacy_production_file_count = 12",
            "expected_legacy_production_file_count = 12",
            "taffy_production_hits = 161",
            "expected_taffy_production_hits = 161",
            "taffy_production_file_count = 7",
            "expected_taffy_production_file_count = 7",
            "runtime_v2_anchor_count = 10",
            "interface_v2_anchor_count = 9",
            "guard_anchor_count = 5",
            "cargo_gate_anchor_count = 7",
            "doc_anchor_count = 11",
            "missing_doc_anchors = []",
            "missing_cargo_gate_anchors = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(expected_anchor),
                "{doc_name} should mirror Runtime 09 UI architecture audit anchor `{expected_anchor}`"
            );
        }
    }
}
