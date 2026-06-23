use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 serialization source folder-split guard sync",
        &[
            "scene_project_serialization_sources_do_not_store_editor_authoring_state",
            "src/asset/assets/scene/mod.rs",
            "src/scene/world/project_io/{camera,physics,post_process,references,script,transform}.rs",
            "SOURCE_AUTHORING_TOKENS",
        ],
    ),
    (
        "Runtime 05 scene/project serialization Markdown renderer split",
        &[
            "scene_project_serialization_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "scene_project_serialization_markdown.py",
            "audited files 24",
            "forbidden_location_count = 0",
        ],
    ),
    (
        "Runtime 05 scene/editor surface Markdown renderer split",
        &[
            "runtime_scene_editor_surface_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "runtime_scene_editor_surface_markdown.py",
            "editor_named_paths = 0",
            "public_editor_named_locations = 0",
        ],
    ),
    (
        "Runtime 05 editor_projection residual guard verdict",
        &[
            "scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover",
            "runtime_scene_exposes_neutral_world_inspection_surface",
            "retired editor-projection module name only as a resurrection guard",
            "editor_projection text remains guard-owned",
        ],
    ),
];
