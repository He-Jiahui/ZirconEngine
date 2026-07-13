use super::*;

pub(super) fn assert_status_rows_and_docs_are_synced() {
    let row_data = read_top_level_support_row_sources();
    assert_contains_all(
        "Runtime 15 expected-slice maps row data",
        &row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_OWNER_PATH,
            NAMING_BOUNDARY_PATH,
            SPLIT_LAYOUT_PATH,
            GUARD,
        ],
    );

    let status_map = read_status_support_status_map_sources();
    assert_contains_all(
        "Runtime 15 expected-slice maps status map",
        &status_map,
        &[SLICE, STATUS],
    );
    let date_map = read_status_support_date_map_sources();
    assert_contains_all(
        "Runtime 15 expected-slice maps date map",
        &date_map,
        &[SLICE, "2026-07-05"],
    );

    for (label, source) in [
        (
            "Runtime 15 plan",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            ),
        ),
        (
            "Runtime index",
            read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "review findings",
            read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        ),
        (
            "structure convention",
            read_repo("docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "module convention doc",
            read_repo("docs/zircon_runtime/structure/module-convention.md"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &[SLICE, STATUS, GUARD, CHILD_OWNER_DOC_ANCHOR],
        );
    }

    let frameworks = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    assert_contains_all(
        "frameworks mirror",
        &frameworks,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
}
