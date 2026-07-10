use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_status_mirrors_are_synced(
) {
    let row_data = read_status_support_expected_slice_rows();
    let status_map = read_status_support_status_map_sources();
    let date_map = read_status_support_date_map_sources();

    assert_contains_all(
        "child-owner split-layout sources row data",
        &row_data,
        &[
            SOURCES_SLICE,
            SOURCES_STATUS,
            SOURCES_PARENT_PATH,
            SOURCES_CHILDREN[0],
            SOURCES_CHILDREN[1],
            SOURCES_CHILDREN[2],
            SOURCES_CHILDREN[3],
            SOURCES_CHILDREN[4],
            SOURCES_CHILDREN[5],
            SOURCES_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "child-owner split-layout sources status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[SOURCES_SLICE, SOURCES_STATUS, "2026-07-07"],
    );
}

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_docs_are_synced() {
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let frameworks_index = read_repo("docs/plans/zircon_runtime/frameworks/index.md");
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
        ("Frameworks 02", frameworks_02.clone()),
        ("Frameworks index", frameworks_index.clone()),
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
        (
            "session note",
            read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                SOURCES_SLICE,
                SOURCES_STATUS,
                SOURCES_FRAMEWORKS_STATUS,
                SOURCES_PARENT_PATH,
                "owners/split/sources/constants.rs",
                "owners/split/sources/row_sources.rs",
                SOURCES_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks child-owner source mirrors",
        &format!("{frameworks_02}\n{frameworks_index}"),
        &[SOURCES_FRAMEWORKS_STATUS],
    );
}
