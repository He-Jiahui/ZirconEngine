#[path = "module_families/animation_backlog.rs"]
mod animation_backlog;
#[path = "module_families/animation_status_json.rs"]
mod animation_status_json;
#[path = "module_families/mirror_docs.rs"]
mod mirror_docs;
#[path = "module_families/navigation.rs"]
mod navigation;
#[path = "module_families/root_seats.rs"]
mod root_seats;
#[path = "module_families/split_layout.rs"]
mod split_layout;

const SLICE: &str = "Runtime 15 M3 root entries module-families guard folder-backed split";
const STATUS: &str =
    "runtime_15_root_entries_module_families_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_root_entries_module_families_guard_is_folder_backed";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_root_entries_module_families_guard_folder_backed_static_passed_cargo_deferred";
