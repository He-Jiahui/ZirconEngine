#[path = "group_owner_paths/folder_backed.rs"]
mod folder_backed;
#[path = "group_owner_paths/owner_path_routes.rs"]
mod owner_path_routes;
#[path = "group_owner_paths/plan_status_row_paths.rs"]
mod plan_status_row_paths;
#[path = "group_owner_paths/root_guard_paths.rs"]
mod root_guard_paths;

pub(in super::super) const M3_CHILD_GROUP_OWNER_PATH_GROUPS: &[&[(&str, &str, usize)]] = &[
    root_guard_paths::M3_CHILD_GROUP_ROOT_GUARD_OWNER_PATHS,
    owner_path_routes::M3_CHILD_GROUP_OWNER_PATH_ROUTE_PATHS,
    plan_status_row_paths::M3_CHILD_GROUP_PLAN_STATUS_CORE_ROW_PATHS,
    plan_status_row_paths::M3_CHILD_GROUP_PLAN_STATUS_PRODUCTION_GUARD_ROW_PATHS,
    plan_status_row_paths::M3_CHILD_GROUP_PLAN_STATUS_UI_TEST_ROW_PATHS,
];
