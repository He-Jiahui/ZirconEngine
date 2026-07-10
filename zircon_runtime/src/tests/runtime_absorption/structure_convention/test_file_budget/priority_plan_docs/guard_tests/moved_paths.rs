use super::*;

const MOVED_PRIORITY_PLAN_DOC_GUARD_PATHS: &[&str] = &[
    "priority_plan_docs/code_paths.rs::runtime_15_priority_plan_docs_code_paths_stay_current",
    "priority_plan_docs/test_paths.rs::runtime_15_priority_plan_docs_test_paths_stay_current",
    "priority_plan_docs/frontmatter_status.rs::runtime_15_priority_plan_docs_frontmatter_status_stays_current",
    "priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
    "priority_plan_docs/header_sections.rs::runtime_15_priority_plan_docs_required_header_sections_stay_complete",
    "priority_plan_docs/plan_sources.rs::runtime_15_priority_plan_docs_plan_sources_stay_cross_linked",
    "priority_plan_docs/guard_tests/listing.rs::runtime_15_priority_plan_docs_guard_tests_stay_listed",
    "priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed",
    "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources",
    "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory",
    "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_children_are_folder_backed",
    "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory",
    "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_guard_paths_stay_current",
    "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory",
];
const MOVED_MIRROR_SLICE: &str =
    "Runtime 15 M3 priority plan docs moved mirror full inventory sync";
const MOVED_MIRROR_STATUS: &str =
    "runtime_15_priority_plan_docs_moved_mirror_full_inventory_sync_static_passed_cargo_deferred";
const MOVED_MIRROR_GUARD: &str = "runtime_15_priority_plan_docs_moved_mirror_names_full_inventory";
const MOVED_MIRROR_GUARD_PATH: &str =
    "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory";

#[test]
fn runtime_15_priority_plan_docs_moved_guard_paths_stay_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = priority_plan_doc_owner_row_source();
    let status_map = priority_plan_doc_status_map_source();
    let date_map = priority_plan_doc_date_map_source();

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 priority plan docs moved guard path mirror",
                "runtime_15_priority_plan_docs_moved_guard_path_mirror_static_passed_cargo_deferred",
                "Cargo gate deferred",
            ],
        );

        assert_contains_all(label, source, MOVED_PRIORITY_PLAN_DOC_GUARD_PATHS);

        assert!(
            !source.contains("priority_plan_docs.rs::runtime_15_priority_plan_docs"),
            "{label} should not keep stale pre-split priority_plan_docs.rs function anchors"
        );
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[
            "Runtime 15 M3 priority plan docs moved guard path mirror",
            "runtime_15_priority_plan_docs_moved_guard_path_mirror_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[
            "Runtime 15 M3 priority plan docs moved guard path mirror",
            "2026-07-01",
        ],
    );
}

#[test]
fn runtime_15_priority_plan_docs_moved_mirror_names_full_inventory() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = priority_plan_doc_owner_row_source();
    let status_map = priority_plan_doc_status_map_source();
    let date_map = priority_plan_doc_date_map_source();

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                MOVED_MIRROR_SLICE,
                MOVED_MIRROR_STATUS,
                MOVED_MIRROR_GUARD,
                MOVED_MIRROR_GUARD_PATH,
                "full priority-plan-doc moved guard inventory",
                "Cargo gate deferred",
            ],
        );
    }

    let current_owner_archive = priority_plan_doc_current_owner_archive_source();
    assert_moved_mirror_windows_name_full_inventory(
        "priority-plan-doc current-owner archive",
        &current_owner_archive,
    );

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[MOVED_MIRROR_SLICE, MOVED_MIRROR_STATUS],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[MOVED_MIRROR_SLICE, "2026-07-04"],
    );
}

fn assert_moved_mirror_windows_name_full_inventory(label: &str, source: &str) {
    let lines: Vec<&str> = source.lines().collect();
    let anchor_indexes: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.contains("runtime_15_priority_plan_docs_moved_guard_path_mirror_static_passed_cargo_deferred").then_some(index))
        .collect();

    assert!(
        !anchor_indexes.is_empty(),
        "{label} should keep a moved guard path mirror status anchor"
    );

    for index in anchor_indexes {
        let window = lines[index..usize::min(index + 40, lines.len())].join("\n");
        assert_contains_all(
            label,
            &window,
            &[
                "full priority-plan-doc moved guard inventory",
                "priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
                "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources",
                "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory",
                MOVED_MIRROR_GUARD_PATH,
            ],
        );
    }
}
