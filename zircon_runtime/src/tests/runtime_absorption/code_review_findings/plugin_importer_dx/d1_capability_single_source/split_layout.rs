use super::{
    D1_FOLDER_BACKED_GUARD, D1_FOLDER_BACKED_SLICE, D1_FOLDER_BACKED_STATUS, D1_FRAMEWORKS_STATUS,
};

const D1_PARENT_PATH: &str =
    "code_review_findings/plugin_importer_dx/d1_capability_single_source.rs";
const D1_CHILD_PATHS: &[&str] = &[
    "code_review_findings/plugin_importer_dx/d1_capability_single_source/runtime_roots.rs",
    "code_review_findings/plugin_importer_dx/d1_capability_single_source/audit_surfaces.rs",
    "code_review_findings/plugin_importer_dx/d1_capability_single_source/sdk_builder.rs",
    "code_review_findings/plugin_importer_dx/d1_capability_single_source/status_docs.rs",
    "code_review_findings/plugin_importer_dx/d1_capability_single_source/support.rs",
    "code_review_findings/plugin_importer_dx/d1_capability_single_source/split_layout.rs",
];

#[test]
fn runtime_15_plugin_importer_d1_capability_guard_is_folder_backed() {
    let parent = include_str!("../d1_capability_single_source.rs");
    let runtime_roots = include_str!("runtime_roots.rs");
    let audit_surfaces = include_str!("audit_surfaces.rs");
    let sdk_builder = include_str!("sdk_builder.rs");
    let status_docs = include_str!("status_docs.rs");
    let support = include_str!("support.rs");
    let split_layout = include_str!("split_layout.rs");
    let row_data = include_str!(
        "../../../../../../../zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/review_guards.rs"
    );
    let status_map = include_str!(
        "../../../../../../../zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/plugin_importer_maps.rs"
    );
    let date_map = include_str!(
        "../../../../../../../zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/plugin_importer_maps.rs"
    );
    let docs = [
        include_str!(
            "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
        ),
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        include_str!(
            "../../../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
        ),
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md"),
        include_str!(
            "../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"
        ),
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        include_str!(
            "../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
        ),
    ]
    .join("\n");

    for mount in [
        "mod audit_surfaces;",
        "mod runtime_roots;",
        "mod sdk_builder;",
        "mod split_layout;",
        "mod status_docs;",
        "mod support;",
    ] {
        assert!(
            parent.contains(mount),
            "D1 route parent should mount `{mount}`"
        );
    }
    for moved_anchor in [
        "FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS",
        "audit_plugin_capability_conformance",
        "PluginFeatureBundleBuilder",
        "let d1_row = review_findings",
        "fn assert_contains_all",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "D1 route parent should not retain moved body anchor `{moved_anchor}`"
        );
    }
    for (source, anchors) in [
        (
            runtime_roots,
            &[
                "D1_RUNTIME_CAPABILITY_ROOTS",
                "assert_runtime_capability_roots_use_single_source",
                "15",
            ][..],
        ),
        (
            audit_surfaces,
            &[
                "assert_capability_audit_surfaces_are_wired",
                "audit_plugin_capability_conformance",
                "plugins_12_capability_single_source_conformance",
            ],
        ),
        (
            sdk_builder,
            &[
                "assert_sdk_builder_mirrors_capabilities",
                "PluginFeatureBundleBuilder",
                "feature_bundle_builder_projects_capability_to_feature_and_modules",
            ],
        ),
        (
            status_docs,
            &[
                "assert_d1_status_docs_are_synced",
                "D1_FOLDER_BACKED_SLICE",
                "D1_FOLDER_BACKED_STATUS",
            ],
        ),
        (support, &["assert_contains_all"]),
        (split_layout, &[D1_FOLDER_BACKED_GUARD]),
    ] {
        for anchor in anchors {
            assert!(
                source.contains(anchor),
                "D1 child should contain `{anchor}`"
            );
        }
    }
    for source in [
        parent,
        runtime_roots,
        audit_surfaces,
        sdk_builder,
        status_docs,
        support,
    ] {
        assert!(
            source.lines().count() < 220,
            "D1 split files should stay focused; got {} lines",
            source.lines().count()
        );
    }
    assert!(
        split_layout.lines().count() < 260,
        "D1 split-layout guard should stay focused"
    );

    for anchor in [
        D1_FOLDER_BACKED_SLICE,
        D1_FOLDER_BACKED_STATUS,
        D1_FOLDER_BACKED_GUARD,
        D1_PARENT_PATH,
        D1_CHILD_PATHS[0],
        D1_CHILD_PATHS[5],
    ] {
        assert!(
            row_data.contains(anchor),
            "D1 row data should contain `{anchor}`"
        );
    }
    for anchor in [
        D1_FOLDER_BACKED_SLICE,
        D1_FOLDER_BACKED_STATUS,
        D1_FOLDER_BACKED_GUARD,
    ] {
        assert!(
            status_map.contains(anchor),
            "D1 status map should contain `{anchor}`"
        );
    }
    for anchor in [D1_FOLDER_BACKED_SLICE, "2026-07-06"] {
        assert!(
            date_map.contains(anchor),
            "D1 date map should contain `{anchor}`"
        );
    }
    for anchor in [
        D1_FOLDER_BACKED_SLICE,
        D1_FOLDER_BACKED_STATUS,
        D1_FRAMEWORKS_STATUS,
        D1_FOLDER_BACKED_GUARD,
        D1_PARENT_PATH,
        D1_CHILD_PATHS[0],
        D1_CHILD_PATHS[5],
    ] {
        assert!(
            docs.contains(anchor),
            "D1 docs/session should contain `{anchor}`"
        );
    }
}
