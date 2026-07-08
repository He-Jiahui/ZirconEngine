use super::*;

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings)
struct LateApiCleanupSources
{
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

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn read_late_api_cleanup_sources(
) -> LateApiCleanupSources {
    LateApiCleanupSources {
        parent: read_runtime_src(PARENT),
        f11_shading_model_registry: read_runtime_src(F11_SHADING_MODEL_REGISTRY),
        f15_editor_pane_data_conversion: read_runtime_src(F15_EDITOR_PANE_DATA_CONVERSION),
        f17_entity_path_lookup: read_runtime_src(F17_ENTITY_PATH_LOOKUP),
        f18_asset_manager_resolution: read_runtime_src(F18_ASSET_MANAGER_RESOLUTION),
        f19_scene_renderer_construction: read_runtime_src(F19_SCENE_RENDERER_CONSTRUCTION),
    }
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn folder_backed_child_sources(
) -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn folder_backed_child_source_blob(
) -> String {
    let mut blob = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        blob.push_str(&child_source);
        blob.push('\n');
    }
    blob
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn late_api_cleanup_structure_guard_child_source_blob(
) -> String {
    let mut blob = String::new();
    blob.push_str(&read_runtime_src(STRUCTURE_GUARD_OWNER));
    blob.push('\n');
    blob.push_str(&folder_backed_child_source_blob());
    blob.push_str(&read_runtime_src(LATE_API_CLEANUP_ROOT_STATUSES_CHILD));
    blob.push('\n');
    let review_sources = read_late_api_cleanup_sources();
    for (path, source) in review_sources.all_sources() {
        blob.push_str(path);
        blob.push('\n');
        blob.push_str(source);
        blob.push('\n');
    }
    blob
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn late_api_cleanup_review_guard_count(
) -> usize {
    let review_sources = read_late_api_cleanup_sources();
    review_sources
        .all_sources()
        .iter()
        .map(|(_, source)| source.matches("#[test]").count())
        .sum()
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn late_api_cleanup_status_row_source(
) -> String {
    format!(
        "{}\n{}",
        read_runtime_src(STRUCTURE_GUARD_ROW_PARENT),
        read_runtime_src(STRUCTURE_GUARD_ROWS),
    )
}
