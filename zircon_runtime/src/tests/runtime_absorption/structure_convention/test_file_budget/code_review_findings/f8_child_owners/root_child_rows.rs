use super::*;

pub(super) const REVIEW_GUARDS: &[&str] = &[
    "review_f8_texture_import_settings_use_fallible_apply_not_with",
    "review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
    "review_f8_first_party_runtime_plugin_descriptors_use_builder",
    "review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
    "review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
    "review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
    "review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending",
];

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    ("delegation", F8_DELEGATION_CHILD, FOLDER_BACKED_GUARD),
    ("route_ownership", F8_ROUTE_OWNERSHIP_CHILD, GUARD),
    (
        "status_mirrors",
        F8_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_STATUS_GUARD,
    ),
    ("budgets", F8_BUDGETS_CHILD, BUDGET_GUARD),
];

pub(super) const F8_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    ("root_paths", F8_ROOT_PATHS_CHILD, "F8_ROOT_PATHS_CHILD"),
    (
        "root_statuses",
        F8_ROOT_STATUSES_CHILD,
        F8_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_child_rows",
        F8_ROOT_CHILD_ROWS_CHILD,
        "F8_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        F8_ROOT_SOURCES_CHILD,
        "read_f8_review_sources",
    ),
    (
        "root_inventory",
        F8_ROOT_INVENTORY_CHILD,
        F8_ROOT_INVENTORY_GUARD,
    ),
];
