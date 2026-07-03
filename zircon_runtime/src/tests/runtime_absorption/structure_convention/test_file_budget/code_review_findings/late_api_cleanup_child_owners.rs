use super::super::*;

#[path = "late_api_cleanup_child_owners/budgets.rs"]
mod budgets;
#[path = "late_api_cleanup_child_owners/delegation.rs"]
mod delegation;
#[path = "late_api_cleanup_child_owners/route_ownership.rs"]
mod route_ownership;
#[path = "late_api_cleanup_child_owners/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STRUCTURE_GUARD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners.rs";
pub(super) const SLICE: &str = "Runtime 15 M3 late API cleanup review guard child-owner split";
pub(super) const STATUS: &str =
    "runtime_15_late_api_cleanup_review_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const DATE: &str = "2026-06-30";
pub(super) const GUARD: &str = "runtime_15_late_api_cleanup_review_guards_are_child_owners";
pub(super) const FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 late API cleanup structure guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS: &str =
    "runtime_15_late_api_cleanup_structure_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_DATE: &str = "2026-07-03";
pub(super) const FOLDER_BACKED_GUARD: &str =
    "runtime_15_late_api_cleanup_structure_guard_is_folder_backed";
pub(super) const FOLDER_BACKED_STATUS_GUARD: &str =
    "runtime_15_late_api_cleanup_structure_guard_folder_backed_status_is_current";
pub(super) const BUDGET_GUARD: &str =
    "runtime_15_late_api_cleanup_structure_guard_budgets_are_focused";

pub(super) const PARENT: &str = "tests/runtime_absorption/code_review_findings/late_api_cleanup.rs";
pub(super) const F11_SHADING_MODEL_REGISTRY: &str =
    "tests/runtime_absorption/code_review_findings/late_api_cleanup/f11_shading_model_registry.rs";
pub(super) const F15_EDITOR_PANE_DATA_CONVERSION: &str =
    "tests/runtime_absorption/code_review_findings/late_api_cleanup/f15_editor_pane_data_conversion.rs";
pub(super) const F17_ENTITY_PATH_LOOKUP: &str =
    "tests/runtime_absorption/code_review_findings/late_api_cleanup/f17_entity_path_lookup.rs";
pub(super) const F18_ASSET_MANAGER_RESOLUTION: &str =
    "tests/runtime_absorption/code_review_findings/late_api_cleanup/f18_asset_manager_resolution.rs";
pub(super) const F19_SCENE_RENDERER_CONSTRUCTION: &str =
    "tests/runtime_absorption/code_review_findings/late_api_cleanup/f19_scene_renderer_construction.rs";
pub(super) const STRUCTURE_GUARD_ROWS: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/late_api_cleanup.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const REVIEW_GUARDS: &[&str] = &[
    "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
    "review_f15_editor_pane_data_conversion_top_row_uses_projection_owners",
    "review_f17_entity_path_option_lookup_uses_get_verb",
    "review_f18_asset_manager_resolution_returns_registered_handle",
    "review_f19_scene_renderer_construction_modules_use_construct_names",
];

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/delegation.rs",
        FOLDER_BACKED_GUARD,
    ),
    (
        "route_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/route_ownership.rs",
        GUARD,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/status_mirrors.rs",
        FOLDER_BACKED_STATUS_GUARD,
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/budgets.rs",
        BUDGET_GUARD,
    ),
];

pub(super) struct LateApiCleanupSources {
    pub(super) parent: String,
    pub(super) f11_shading_model_registry: String,
    pub(super) f15_editor_pane_data_conversion: String,
    pub(super) f17_entity_path_lookup: String,
    pub(super) f18_asset_manager_resolution: String,
    pub(super) f19_scene_renderer_construction: String,
}

impl LateApiCleanupSources {
    pub(super) fn all_sources(&self) -> [(&'static str, &str); 6] {
        [
            (PARENT, self.parent.as_str()),
            (
                F11_SHADING_MODEL_REGISTRY,
                self.f11_shading_model_registry.as_str(),
            ),
            (
                F15_EDITOR_PANE_DATA_CONVERSION,
                self.f15_editor_pane_data_conversion.as_str(),
            ),
            (F17_ENTITY_PATH_LOOKUP, self.f17_entity_path_lookup.as_str()),
            (
                F18_ASSET_MANAGER_RESOLUTION,
                self.f18_asset_manager_resolution.as_str(),
            ),
            (
                F19_SCENE_RENDERER_CONSTRUCTION,
                self.f19_scene_renderer_construction.as_str(),
            ),
        ]
    }
}

pub(super) fn read_late_api_cleanup_sources() -> LateApiCleanupSources {
    LateApiCleanupSources {
        parent: read_runtime_src(PARENT),
        f11_shading_model_registry: read_runtime_src(F11_SHADING_MODEL_REGISTRY),
        f15_editor_pane_data_conversion: read_runtime_src(F15_EDITOR_PANE_DATA_CONVERSION),
        f17_entity_path_lookup: read_runtime_src(F17_ENTITY_PATH_LOOKUP),
        f18_asset_manager_resolution: read_runtime_src(F18_ASSET_MANAGER_RESOLUTION),
        f19_scene_renderer_construction: read_runtime_src(F19_SCENE_RENDERER_CONSTRUCTION),
    }
}

pub(super) fn folder_backed_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        blob.push_str(&child_source);
        blob.push('\n');
    }
    blob
}

pub(super) fn late_api_cleanup_review_guard_count() -> usize {
    let review_sources = read_late_api_cleanup_sources();
    review_sources
        .all_sources()
        .iter()
        .map(|(_, source)| source.matches("#[test]").count())
        .sum()
}

pub(super) fn assert_late_api_cleanup_child_owners_are_folder_backed() {
    route_ownership::assert_late_api_cleanup_child_owners_are_folder_backed();
}
