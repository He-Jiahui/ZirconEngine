use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_runtime_parents_are_folder_backed(
    sources: &TypedErrorConvergenceMountSources,
) {
    assert_contains_all(
        "scene world typed-error parent mounts focused child owners",
        &sources.scene_world_parent,
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
        sources.scene_world_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/scene_world.rs should only mount child test owners"
    );
    assert_contains_all(
        "script host typed-error parent mounts focused child owners",
        &sources.script_host_parent,
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
        sources.script_host_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/script_host.rs should only mount child test owners"
    );
    assert_contains_all(
        "shader prewarm CLI typed-error parent mounts focused child owners",
        &sources.shader_prewarm_cli_parent,
        &[
            "#[path = \"shader_prewarm_cli/args_boundary.rs\"]",
            "mod args_boundary;",
            "#[path = \"shader_prewarm_cli/run_boundary.rs\"]",
            "mod run_boundary;",
        ],
    );
    assert_eq!(
        sources.shader_prewarm_cli_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/shader_prewarm_cli.rs should only mount child test owners"
    );
    assert_contains_all(
        "UI input typed-error parent mounts focused child owners",
        &sources.ui_input_parent,
        &[
            "#[path = \"ui_input/surface_effects.rs\"]",
            "mod surface_effects;",
            "#[path = \"ui_input/surrounding_text.rs\"]",
            "mod surrounding_text;",
        ],
    );
    assert_eq!(
        sources.ui_input_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/ui_input.rs should only mount child test owners"
    );
}
