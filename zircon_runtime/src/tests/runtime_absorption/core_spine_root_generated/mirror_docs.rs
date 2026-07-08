use std::path::Path;

use super::generated_behavior::{behavior_labels, generated_behavior_locations};
use super::inventory::{
    EXPECTED_CORE_PUBLIC_MODULES, EXPECTED_CORE_ROOT_ENTRIES,
    EXPECTED_RUNTIME_02_GUARD_TEST_ANCHORS, MIRROR_DOCS, RETIRED_CORE_ROOT_ENTRIES,
};
use super::source_helpers::{
    core_root_entries, crate_visible_graphics_reexport_count, export_template_files,
    public_modules, public_use_count, read_source, rust_test_count_in_files, string_set,
    string_vec,
};

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
        19,
        "runtime root public module count changed without Runtime 02 audit sync"
    );
    assert_eq!(
        public_use_count(&crate_root),
        2,
        "runtime root public `pub use` count changed without Runtime 02 audit sync"
    );
    assert_eq!(
        crate_visible_graphics_reexport_count(&crate_root),
        0,
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

    let root_entries_guard_files = [
        "src/tests/runtime_absorption/root_entries/core_spine.rs",
        "src/tests/runtime_absorption/root_entries/module_families/navigation.rs",
        "src/tests/runtime_absorption/root_entries/module_families/animation_backlog.rs",
        "src/tests/runtime_absorption/root_entries/module_families/animation_status_json.rs",
        "src/tests/runtime_absorption/root_entries/module_families/root_seats.rs",
        "src/tests/runtime_absorption/root_entries/module_families/mirror_docs.rs",
        "src/tests/runtime_absorption/root_entries/runtime_root.rs",
    ];
    assert_eq!(
        rust_test_count_in_files(runtime_root, &root_entries_guard_files),
        13,
        "root_entries guard test count changed without Runtime 02 audit sync"
    );
    let root_surface_guard_files = [
        "src/tests/runtime_absorption/root_surface/public_surface.rs",
        "src/tests/runtime_absorption/root_surface/graphics_alias.rs",
        "src/tests/runtime_absorption/root_surface/docs.rs",
    ];
    assert_eq!(
        rust_test_count_in_files(runtime_root, &root_surface_guard_files),
        6,
        "root_surface guard test count changed without Runtime 02 audit sync"
    );
    let generated_guard_files = [
        "src/tests/runtime_absorption/generated_code_guard/markers.rs",
        "src/tests/runtime_absorption/generated_code_guard/behavior.rs",
        "src/tests/runtime_absorption/generated_code_guard/scope.rs",
        "src/tests/runtime_absorption/generated_code_guard/delegation.rs",
    ];
    assert_eq!(
        rust_test_count_in_files(runtime_root, &generated_guard_files),
        7,
        "generated-code guard test count changed without Runtime 02 audit sync"
    );
    assert_runtime_02_guard_test_anchors(workspace_root);

    for relative in [
        "zircon_runtime/src/tests/runtime_absorption/root_entries.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_entries/core_spine.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/navigation.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/animation_backlog.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/animation_status_json.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/root_seats.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/mirror_docs.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_entries/runtime_root.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_surface.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_surface/public_surface.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_surface/graphics_alias.rs",
        "zircon_runtime/src/tests/runtime_absorption/root_surface/docs.rs",
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs",
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/markers.rs",
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/behavior.rs",
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/scope.rs",
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/delegation.rs",
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
            "runtime root public modules 19/19",
            "public `pub use` sites 2/2",
            "crate-visible graphics alias debt 0/0",
            "root-surface M1 gate `classified-and-clear`",
            "generated export templates 10/10",
            "generated behavior 6/6",
            "generated allowed adapters 6/6",
            "generated migration debt 0/0",
            "generated-code M1 gate `classified-and-clear`",
            "root_entries guard tests 13",
            "root_surface guard tests 6/6",
            "generated-code guard tests 7/7",
            "guard_test_anchor_count = 26",
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
        guard_test_anchor_count, 26,
        "Runtime 02 guard test anchor inventory should stay at 26 anchors"
    );
}
