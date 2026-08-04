use super::*;

#[path = "inventory_sync/source_ownership.rs"]
mod source_ownership;

use source_ownership::assert_priority_plan_doc_guard_row_data_sources_are_child_owned;

const SLICE: &str = "Runtime 15 M3 priority plan docs guard inventory row-data source sync";
const STATUS: &str = "runtime_15_priority_plan_docs_guard_inventory_row_data_source_sync_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources";
const LISTING_PROSE_SLICE: &str =
    "Runtime 15 M3 priority plan docs listing prose full inventory sync";
const LISTING_PROSE_STATUS: &str =
    "runtime_15_priority_plan_docs_listing_prose_full_inventory_sync_static_passed_cargo_deferred";
const LISTING_PROSE_GUARD: &str =
    "runtime_15_priority_plan_docs_listing_prose_names_full_inventory";
const FRONTMATTER_UNIQUENESS_GUARD_PATH: &str = "priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries";
const INVENTORY_SYNC_GUARD_PATH: &str = "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources";
const LISTING_PROSE_GUARD_PATH: &str = "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory";

#[test]
fn runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources() {
    assert_priority_plan_doc_guard_row_data_sources_are_child_owned();

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}

#[test]
fn runtime_15_priority_plan_docs_listing_prose_names_full_inventory() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
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
