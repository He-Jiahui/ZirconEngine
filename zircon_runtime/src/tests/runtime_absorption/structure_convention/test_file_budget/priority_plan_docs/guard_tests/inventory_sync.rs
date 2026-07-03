use super::*;

const SLICE: &str = "Runtime 15 M3 priority plan docs guard inventory row-data source sync";
const STATUS: &str =
    "runtime_15_priority_plan_docs_guard_inventory_row_data_source_sync_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources";
const LISTING_PROSE_SLICE: &str =
    "Runtime 15 M3 priority plan docs listing prose full inventory sync";
const LISTING_PROSE_STATUS: &str =
    "runtime_15_priority_plan_docs_listing_prose_full_inventory_sync_static_passed_cargo_deferred";
const LISTING_PROSE_GUARD: &str =
    "runtime_15_priority_plan_docs_listing_prose_names_full_inventory";
const FRONTMATTER_UNIQUENESS_GUARD_PATH: &str =
    "priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries";
const INVENTORY_SYNC_GUARD_PATH: &str =
    "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources";
const LISTING_PROSE_GUARD_PATH: &str =
    "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory";
const INTEGRITY_ROW_DATA_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs";
const OWNER_ROW_DATA_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs";
const STALE_PARENT_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs";

#[test]
fn runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources() {
    assert_priority_plan_doc_guard_row_data_sources_are_child_owned();

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(OWNER_ROW_DATA_PATH);
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "priority-plan-doc owner status-output row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                GUARD,
                FRONTMATTER_UNIQUENESS_GUARD_PATH,
                INVENTORY_SYNC_GUARD_PATH,
                INTEGRITY_ROW_DATA_PATH,
                OWNER_ROW_DATA_PATH,
                "child row-data source",
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all("status expected-slice map", &status_map, &[SLICE, STATUS]);
    assert_contains_all("date expected-slice map", &date_map, &[SLICE, "2026-07-04"]);
}

#[test]
fn runtime_15_priority_plan_docs_listing_prose_names_full_inventory() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(OWNER_ROW_DATA_PATH);
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "priority-plan-doc owner status-output row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                LISTING_PROSE_SLICE,
                LISTING_PROSE_STATUS,
                LISTING_PROSE_GUARD,
                FRONTMATTER_UNIQUENESS_GUARD_PATH,
                INVENTORY_SYNC_GUARD_PATH,
                LISTING_PROSE_GUARD_PATH,
                "full priority-plan-doc guard inventory",
                "Cargo gate deferred",
            ],
        );
        assert_listing_prose_has_no_stale_inventory_terms(label, source);
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[LISTING_PROSE_SLICE, LISTING_PROSE_STATUS],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[LISTING_PROSE_SLICE, "2026-07-04"],
    );
}

fn assert_priority_plan_doc_guard_row_data_sources_are_child_owned() {
    for path in [
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/code_paths.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/header_sections.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/test_paths.rs",
    ] {
        let source = read_runtime_src(path);
        assert!(
            source.contains(INTEGRITY_ROW_DATA_PATH),
            "{path} should read priority-plan-doc integrity row-data child"
        );
        assert!(
            !source.contains(STALE_PARENT_ROW_DATA_PATH),
            "{path} should not read stale status_support.rs parent row data"
        );
    }

    for path in [
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs",
    ] {
        let source = read_runtime_src(path);
        assert!(
            source.contains(OWNER_ROW_DATA_PATH),
            "{path} should read priority-plan-doc owner row-data child"
        );
        assert!(
            !source.contains(STALE_PARENT_ROW_DATA_PATH),
            "{path} should not read stale status_support.rs parent row data"
        );
    }

    let priority_plan_docs = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs.rs",
    );
    let listing_guard = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs",
    );
    let moved_paths_guard = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs",
    );

    assert_contains_all(
        "priority-plan-doc guard inventory",
        &priority_plan_docs,
        &[
            "runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
            GUARD,
        ],
    );
    assert_contains_all(
        "priority-plan-doc guard listing",
        &listing_guard,
        &[
            FRONTMATTER_UNIQUENESS_GUARD_PATH,
            INVENTORY_SYNC_GUARD_PATH,
            LISTING_PROSE_GUARD_PATH,
            "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory",
            "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory",
        ],
    );
    assert_contains_all(
        "priority-plan-doc moved guard paths",
        &moved_paths_guard,
        &[
            FRONTMATTER_UNIQUENESS_GUARD_PATH,
            INVENTORY_SYNC_GUARD_PATH,
            LISTING_PROSE_GUARD_PATH,
            "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory",
            "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory",
        ],
    );
}

fn assert_listing_prose_has_no_stale_inventory_terms(label: &str, source: &str) {
    for stale in [
        "priority_plan_docs.rs` guards",
        "priority_plan_docs.rs guards",
        "frontmatter status、required header sections、plan-source cross-link 与本 listing guard",
        "`...frontmatter_status_stays_current`、`...required_header_sections_stay_complete`",
    ] {
        assert!(
            !source.contains(stale),
            "{label} should not describe priority-plan-doc listing inventory with stale term `{stale}`"
        );
    }
}
