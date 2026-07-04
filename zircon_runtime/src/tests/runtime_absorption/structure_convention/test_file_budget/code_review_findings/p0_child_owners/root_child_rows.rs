use super::*;

pub(super) const REVIEW_GUARDS: &[&str] = &[
    "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
    "review_f2_scene_eventbus_locks_recover_after_poison",
    "review_f4_render_submit_capability_gaps_return_typed_errors",
    "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
    "review_d13_native_fixture_importer_is_manifest_described",
    "review_priority_recommendation_tracks_current_remaining_work",
];

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    ("delegation", P0_DELEGATION_CHILD, FOLDER_BACKED_GUARD),
    ("route_ownership", P0_ROUTE_OWNERSHIP_CHILD, GUARD),
    (
        "status_mirrors",
        P0_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_STATUS_GUARD,
    ),
    ("budgets", P0_BUDGETS_CHILD, BUDGET_GUARD),
];

pub(super) const P0_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    ("root_paths", P0_ROOT_PATHS_CHILD, "P0_ROOT_PATHS_CHILD"),
    (
        "root_statuses",
        P0_ROOT_STATUSES_CHILD,
        P0_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        P0_ROOT_CHILD_ROWS_CHILD,
        "P0_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        P0_ROOT_SOURCES_CHILD,
        "read_p0_robustness_sources",
    ),
    (
        "root_inventory",
        P0_ROOT_INVENTORY_CHILD,
        P0_ROOT_INVENTORY_GUARD,
    ),
];
