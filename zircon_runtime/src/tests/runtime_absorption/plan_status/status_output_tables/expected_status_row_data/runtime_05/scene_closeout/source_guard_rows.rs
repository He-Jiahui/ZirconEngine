use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 serialization source folder-split guard sync",
        [
            "scene_project_serialization_sources_do_not_store_editor_authoring_state",
            "src/asset/assets/scene/mod.rs",
            "src/scene/world/project_io/{camera,physics,post_process,references,script,transform}.rs",
            "SOURCE_AUTHORING_TOKENS",
        ],
    ),
    (
        "Runtime 05 editor_projection residual guard verdict",
        [
            "scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover",
            "runtime_scene_exposes_neutral_world_inspection_surface",
            "retired editor-projection module name only as a resurrection guard",
            "editor_projection text remains guard-owned",
        ],
    ),
];
