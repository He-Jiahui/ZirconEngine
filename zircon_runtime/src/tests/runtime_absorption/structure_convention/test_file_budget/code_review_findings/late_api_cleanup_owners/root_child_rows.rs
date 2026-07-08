use super::*;

pub(super) const REVIEW_GUARDS: &[&str] = &[
    "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
    "review_f15_editor_pane_data_conversion_top_row_uses_projection_owners",
    "review_f17_entity_path_option_lookup_uses_get_verb",
    "review_f18_asset_manager_resolution_returns_registered_handle",
    "review_f19_scene_renderer_construction_modules_use_construct_names",
];

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    // tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_owners/delegation.rs
    (
        "delegation",
        LATE_API_CLEANUP_DELEGATION_CHILD,
        FOLDER_BACKED_GUARD,
    ),
    // tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_owners/route_ownership.rs
    (
        "route_ownership",
        LATE_API_CLEANUP_ROUTE_OWNERSHIP_CHILD,
        GUARD,
    ),
    // tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_owners/status_mirrors.rs
    (
        "status_mirrors",
        LATE_API_CLEANUP_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_STATUS_GUARD,
    ),
    // tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_owners/budgets.rs
    ("budgets", LATE_API_CLEANUP_BUDGETS_CHILD, BUDGET_GUARD),
];

pub(super) const LATE_API_CLEANUP_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        LATE_API_CLEANUP_ROOT_PATHS_CHILD,
        "LATE_API_CLEANUP_ROOT_PATHS_CHILD",
    ),
    (
        "root_statuses",
        LATE_API_CLEANUP_ROOT_STATUSES_CHILD,
        LATE_API_CLEANUP_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        LATE_API_CLEANUP_ROOT_CHILD_ROWS_CHILD,
        "LATE_API_CLEANUP_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        LATE_API_CLEANUP_ROOT_SOURCES_CHILD,
        "read_late_api_cleanup_sources",
    ),
    (
        "root_inventory",
        LATE_API_CLEANUP_ROOT_INVENTORY_CHILD,
        LATE_API_CLEANUP_ROOT_INVENTORY_GUARD,
    ),
];
