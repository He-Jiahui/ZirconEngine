const PARENT_SOURCE: &str = include_str!("../dynamic_scene.rs");
const SOURCES_SOURCE: &str = include_str!("sources.rs");
const PATCH_PREVIEW_API_SOURCE: &str = include_str!("patch_preview_api.rs");
const PATCH_PREVIEW_BEHAVIOR_SOURCE: &str = include_str!("patch_preview_behavior.rs");
const PATCH_PREVIEW_STATUS_DOCS_SOURCE: &str = include_str!("patch_preview_status_docs.rs");
const SESSION_CAPTURE_PERSISTENCE_SOURCE: &str = include_str!("session_capture_persistence.rs");
const SESSION_LOAD_QUERY_PATH_SOURCE: &str = include_str!("session_load_query_path.rs");
const SESSION_RETENTION_MUTATION_MERGE_SOURCE: &str =
    include_str!("session_retention_mutation_merge.rs");
const ASSET_RELOAD_SELECTION_STATUS_SOURCE: &str = include_str!("asset_reload_selection_status.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const FRAMEWORKS_02_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md");
const RUNTIME_15_PLAN: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
);
const RUNTIME_INDEX: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
const STRUCTURE_CONVENTION_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
const REVIEW_FINDINGS_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
#[rustfmt::skip]
const NUMBERED_STATUS_RECORDS: &str = concat!(
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const SESSION_NOTE: &str = include_str!(
    "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
);
const STATUS_ROW_DATA: &str = include_str!(
    "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests.rs"
);
const STATUS_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps.rs"
);
const DATE_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps.rs"
);

#[test]
fn runtime_15_dynamic_scene_route_owner_is_folder_backed() {
    assert_contains_all(
        "dynamic_scene route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"dynamic_scene/asset_reload_selection_status.rs\"]",
            "#[path = \"dynamic_scene/patch_preview_api.rs\"]",
            "#[path = \"dynamic_scene/patch_preview_behavior.rs\"]",
            "#[path = \"dynamic_scene/patch_preview_status_docs.rs\"]",
            "#[path = \"dynamic_scene/session_capture_persistence.rs\"]",
            "#[path = \"dynamic_scene/session_load_query_path.rs\"]",
            "#[path = \"dynamic_scene/session_retention_mutation_merge.rs\"]",
            "#[path = \"dynamic_scene/sources.rs\"]",
            "#[path = \"dynamic_scene/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owners_are_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "dynamic_scene.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "include_str!(",
        "const PATCH_SOURCE",
        "const RUNTIME_05_PLAN",
        "const DYNAMIC_SCENE_DOC",
        "use super::*;",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "dynamic_scene.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "sources child",
        SOURCES_SOURCE,
        &[
            "pub(super) const PATCH_SOURCE",
            "pub(super) const RUNTIME_05_PLAN",
            "pub(super) const DYNAMIC_SCENE_DOC",
            "PATH_MANAGEMENT_SLOT_MUTATIONS_SOURCE",
        ],
    );
    for (label, source, guard_anchor) in [
        (
            "patch preview api child",
            PATCH_PREVIEW_API_SOURCE,
            "runtime_05_dynamic_scene_patch_preview_api_stays_read_only",
        ),
        (
            "patch preview behavior child",
            PATCH_PREVIEW_BEHAVIOR_SOURCE,
            "runtime_05_dynamic_scene_patch_preview_behavior_anchors_stay_visible",
        ),
        (
            "patch preview status docs child",
            PATCH_PREVIEW_STATUS_DOCS_SOURCE,
            "runtime_05_dynamic_scene_patch_preview_status_docs_stay_synced",
        ),
        (
            "session capture persistence child",
            SESSION_CAPTURE_PERSISTENCE_SOURCE,
            "runtime_05_dynamic_scene_session_capture_persistence_anchors_stay_visible",
        ),
        (
            "session load query path child",
            SESSION_LOAD_QUERY_PATH_SOURCE,
            "runtime_05_dynamic_scene_session_load_query_path_anchors_stay_visible",
        ),
        (
            "session retention mutation merge child",
            SESSION_RETENTION_MUTATION_MERGE_SOURCE,
            "runtime_05_dynamic_scene_session_retention_mutation_merge_anchors_stay_visible",
        ),
        (
            "asset reload selection status child",
            ASSET_RELOAD_SELECTION_STATUS_SOURCE,
            "runtime_05_dynamic_scene_asset_reload_selection_and_status_anchors_stay_visible",
        ),
    ] {
        assert_contains_all(label, source, &["use super::sources::*;", guard_anchor]);
    }
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 24),
        ("sources child", SOURCES_SOURCE, 70),
        ("patch preview api child", PATCH_PREVIEW_API_SOURCE, 140),
        (
            "patch preview behavior child",
            PATCH_PREVIEW_BEHAVIOR_SOURCE,
            60,
        ),
        (
            "patch preview status docs child",
            PATCH_PREVIEW_STATUS_DOCS_SOURCE,
            140,
        ),
        (
            "session capture persistence child",
            SESSION_CAPTURE_PERSISTENCE_SOURCE,
            130,
        ),
        (
            "session load query path child",
            SESSION_LOAD_QUERY_PATH_SOURCE,
            130,
        ),
        (
            "session retention mutation merge child",
            SESSION_RETENTION_MUTATION_MERGE_SOURCE,
            120,
        ),
        (
            "asset reload selection status child",
            ASSET_RELOAD_SELECTION_STATUS_SOURCE,
            300,
        ),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 240),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    for (label, source) in [
        ("Frameworks 02 plan", FRAMEWORKS_02_PLAN),
        ("Runtime 15 plan", RUNTIME_15_PLAN),
        ("runtime index", RUNTIME_INDEX),
        ("structure convention plan", STRUCTURE_CONVENTION_PLAN),
        ("review findings plan", REVIEW_FINDINGS_PLAN),
        ("module convention doc", MODULE_CONVENTION_DOC),
        ("session note", SESSION_NOTE),
        ("status row data", STATUS_ROW_DATA),
        ("status map", STATUS_MAP),
    ] {
        assert!(
            source.contains(
                "runtime_15_dynamic_scene_route_owner_split_static_passed_cargo_deferred"
            ) || NUMBERED_STATUS_RECORDS.contains(
                "runtime_15_dynamic_scene_route_owner_split_static_passed_cargo_deferred"
            ),
            "{label} should mirror the dynamic_scene route-owner split status"
        );
    }
    assert!(
        DATE_MAP.contains("Runtime 15 M3 dynamic-scene route-owner split")
            || NUMBERED_STATUS_RECORDS.contains("Runtime 15 M3 dynamic-scene route-owner split"),
        "date map should mirror the dynamic_scene route-owner split slice"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "dynamic_scene/sources.rs",
            "dynamic_scene/split_layout.rs",
            "runtime_15_dynamic_scene_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 status",
        NUMBERED_STATUS_RECORDS,
        &[
            "frameworks_02_m3_dynamic_scene_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 dynamic-scene route-owner split",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain split anchor `{anchor}`"
        );
    }
}
