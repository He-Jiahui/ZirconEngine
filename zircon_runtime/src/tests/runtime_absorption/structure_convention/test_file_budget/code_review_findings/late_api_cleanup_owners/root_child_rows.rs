use super::*;

pub(super) const REVIEW_GUARDS: &[&str] = &[
    "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
    "review_f15_editor_pane_data_conversion_top_row_uses_projection_owners",
    "review_f17_entity_path_option_lookup_uses_get_verb",
    "review_f18_asset_manager_resolution_returns_registered_handle",
    "review_f19_scene_renderer_construction_modules_use_construct_names",
];

pub(super) const LATE_API_CLEANUP_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        LATE_API_CLEANUP_ROOT_PATHS_CHILD,
        "LATE_API_CLEANUP_ROOT_PATHS_CHILD",
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
];
