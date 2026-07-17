use super::super::*;
use super::*;

pub(super) fn assert_structure_guard_plugin_importer_status_mirrors_are_current() {
    let status_rows = structure_guard_status_row_source();
    let status_map = super::super::structure_guard_status_map_source();
    let date_map = super::super::structure_guard_date_map_source();
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("structure guard row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_SLICE,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_STATUS,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_TOP_LEVEL_CHILDREN_CHILD,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_STRUCTURE_ASSERTIONS_CHILD,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_SOURCE_INVENTORY_CHILD,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_DOCS_CHILD,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_OWNERSHIP_CHILD,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_MIRRORS_CHILD,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_GUARD,
                STRUCTURE_GUARD_PLUGIN_IMPORTER_STATUS_MIRROR_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records structure guard plugin-importer child split",
        &status_map,
        &[
            STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_SLICE,
            STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records structure guard plugin-importer child split",
        &date_map,
        &[
            STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_SLICE,
            STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT_DATE,
        ],
    );
}
