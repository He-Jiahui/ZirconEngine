use std::path::Path;

use super::super::support::{
    assert_contains_all, read_repo_text, read_runtime_15_naming_date_map,
    read_runtime_15_naming_status_map, read_runtime_15_naming_status_rows, read_text,
};

#[path = "banned_names/global_modules.rs"]
mod global_modules;
#[path = "banned_names/graphics_construction.rs"]
mod graphics_construction;
#[path = "banned_names/scene_dynamic.rs"]
mod scene_dynamic;

const SLICE: &str = "Runtime 15 M2 render framework trait/construction owner naming hard cutover";
const STATUS: &str =
    "runtime_15_render_framework_trait_construction_owner_naming_hard_cutover_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_no_banned_name_modules";

const SCENE_DYNAMIC_DOCUMENT_V1_SLICE: &str =
    "Runtime 15 M2 scene dynamic document v1 owner naming hard cutover";
const SCENE_DYNAMIC_DOCUMENT_V1_STATUS: &str =
    "runtime_15_scene_dynamic_document_v1_owner_naming_hard_cutover_static_passed_cargo_deferred";
const SCENE_DYNAMIC_DOCUMENT_V1_GUARD: &str =
    "runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name";

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
