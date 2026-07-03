use super::super::super::super::*;

pub(super) fn assert_typed_error_convergence_parents_are_folder_backed() {
    let typed_error_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
    );
    let asset_loaders_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
    );
    let asset_records_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
    );
    let scene_world_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
    );
    let script_host_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
    );
    let shader_prewarm_cli_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli.rs",
    );
    let ui_input_parent = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
    );

    assert_contains_all(
        "typed-error convergence parent mounts focused child owners",
        &typed_error_parent,
        &[
            "mod animation_resource;",
            "mod asset_loaders;",
            "mod asset_records;",
            "mod diagnostics;",
            "mod dynamic_api;",
            "mod export_cli;",
            "mod native_plugin_loader;",
            "mod scene_world;",
            "mod script_host;",
            "mod shader_prewarm_cli;",
            "mod ui_asset_documents;",
            "mod ui_input;",
            "mod ui_template_resource;",
        ],
    );
    assert_eq!(
        typed_error_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/mod.rs should only mount child test owners"
    );
    assert_contains_all(
        "asset loaders typed-error parent mounts focused child owners",
        &asset_loaders_parent,
        &[
            "#[path = \"asset_loaders/animation_binary.rs\"]",
            "mod animation_binary;",
            "#[path = \"asset_loaders/artifact_importer.rs\"]",
            "mod artifact_importer;",
            "#[path = \"asset_loaders/mesh_obj.rs\"]",
            "mod mesh_obj;",
            "#[path = \"asset_loaders/texture.rs\"]",
            "mod texture;",
        ],
    );
    assert_eq!(
        asset_loaders_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/asset_loaders.rs should only mount child test owners"
    );
    assert_contains_all(
        "asset records typed-error parent mounts focused child owners",
        &asset_records_parent,
        &[
            "#[path = \"asset_records/authoring.rs\"]",
            "mod authoring;",
            "#[path = \"asset_records/font.rs\"]",
            "mod font;",
            "#[path = \"asset_records/meta.rs\"]",
            "mod meta;",
            "#[path = \"asset_records/navigation.rs\"]",
            "mod navigation;",
            "#[path = \"asset_records/sound.rs\"]",
            "mod sound;",
            "#[path = \"asset_records/zshader.rs\"]",
            "mod zshader;",
        ],
    );
    assert_eq!(
        asset_records_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/asset_records.rs should only mount child test owners"
    );
    assert_contains_all(
        "scene world typed-error parent mounts focused child owners",
        &scene_world_parent,
        &[
            "#[path = \"scene_world/typed_mutation_surface.rs\"]",
            "mod typed_mutation_surface;",
            "#[path = \"scene_world/fixed_mutation.rs\"]",
            "mod fixed_mutation;",
            "#[path = \"scene_world/dynamic_components.rs\"]",
            "mod dynamic_components;",
            "#[path = \"scene_world/property_access.rs\"]",
            "mod property_access;",
        ],
    );
    assert_eq!(
        scene_world_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/scene_world.rs should only mount child test owners"
    );
    assert_contains_all(
        "script host typed-error parent mounts focused child owners",
        &script_host_parent,
        &[
            "#[path = \"script_host/gameplay_scene.rs\"]",
            "mod gameplay_scene;",
            "#[path = \"script_host/plugin_management.rs\"]",
            "mod plugin_management;",
            "#[path = \"script_host/host_reflection_docs.rs\"]",
            "mod host_reflection_docs;",
        ],
    );
    assert_eq!(
        script_host_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/script_host.rs should only mount child test owners"
    );
    assert_contains_all(
        "shader prewarm CLI typed-error parent mounts focused child owners",
        &shader_prewarm_cli_parent,
        &[
            "#[path = \"shader_prewarm_cli/args_boundary.rs\"]",
            "mod args_boundary;",
            "#[path = \"shader_prewarm_cli/run_boundary.rs\"]",
            "mod run_boundary;",
        ],
    );
    assert_eq!(
        shader_prewarm_cli_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/shader_prewarm_cli.rs should only mount child test owners"
    );
    assert_contains_all(
        "UI input typed-error parent mounts focused child owners",
        &ui_input_parent,
        &[
            "#[path = \"ui_input/surface_effects.rs\"]",
            "mod surface_effects;",
            "#[path = \"ui_input/surrounding_text.rs\"]",
            "mod surrounding_text;",
        ],
    );
    assert_eq!(
        ui_input_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/ui_input.rs should only mount child test owners"
    );
}
