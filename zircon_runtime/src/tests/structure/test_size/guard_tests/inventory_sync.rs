//! 约束优先计划文档守卫的入口清单与子文件归属；保留名称的文本证据需随职责迁移更新，归档文字不等于当前验证结果。
use super::*;

#[path = "../inventory_sync/source_ownership.rs"]
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
}

#[test]
fn runtime_15_priority_plan_docs_listing_prose_names_full_inventory() {
    for path in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/crates/zircon_runtime/structure/module-convention.md",
    ] {
        assert_listing_prose_has_no_stale_inventory_terms(path, &read_repo(path));
    }
    for (label, source) in independent_listing_mirror_sources() {
        assert_listing_prose_has_no_stale_inventory_terms(label, &source);
        assert_contains_all(
            label,
            &source,
            &[
                LISTING_PROSE_SLICE,
                LISTING_PROSE_STATUS,
                LISTING_PROSE_GUARD,
                "full priority-plan-doc guard inventory",
                FRONTMATTER_UNIQUENESS_GUARD_PATH,
                INVENTORY_SYNC_GUARD_PATH,
                LISTING_PROSE_GUARD_PATH,
            ],
        );
    }
}

pub(super) fn independent_listing_mirror_sources() -> [(&'static str, String); 5] {
    let history =
        crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source;
    [
        ("current Runtime 15", read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md")),
        ("current module convention", read_repo("docs/crates/zircon_runtime/structure/module-convention.md")),
        ("fixed historical runtime index", history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md").to_owned()),
        ("fixed historical structure mirror", history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md").to_owned()),
        ("fixed historical review mirror", history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md").to_owned()),
    ]
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
