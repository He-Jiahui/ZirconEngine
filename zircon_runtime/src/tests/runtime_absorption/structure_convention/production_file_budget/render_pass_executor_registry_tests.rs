use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_pass_executor_registry_tests_are_child_owners() {
    let root = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs",
    );
    let registry = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs",
    );
    let postprocess = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/postprocess_context_guards.rs",
    );
    let renderer = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/renderer_context_guards.rs",
    );

    let plan_01 = read_repo("docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let graph_execution_doc = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.md",
    );

    assert_contains_all(
        "executor registry test root keeps support and child owner mounts",
        &root,
        &[
            "#[path = \"plugin_executor_policy.rs\"]",
            "#[path = \"support.rs\"]",
            "mod registry_contracts;",
            "mod postprocess_context_guards;",
            "mod renderer_context_guards;",
        ],
    );

    for moved_test in [
        "fn registry_rejects_unregistered_executor_ids(",
        "fn taa_reactive_mask_clear_executor_requires_graph_resources_instead_of_nooping(",
        "fn screen_space_ui_executor_uses_graph_attachment_ops_for_viewport_output(",
        "fn deferred_lighting_executor_requires_renderer_context_instead_of_nooping(",
    ] {
        assert!(
            !root.contains(moved_test),
            "executor registry test root should not own moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "registry contract child owns registry coverage",
        &registry,
        &[
            "use super::*;",
            "fn registry_rejects_unregistered_executor_ids(",
            "fn builtin_registry_covers_compiled_pipeline_executor_ids(",
            "fn registry_invokes_object_backed_executor_with_mutable_context(",
            "fn registry_ignores_culled_pass_with_unknown_executor_id(",
        ],
    );
    assert_contains_all(
        "postprocess guard child owns temporal and effect-stack guards",
        &postprocess,
        &[
            "use super::*;",
            "fn taa_reactive_mask_clear_executor_requires_graph_resources_instead_of_nooping(",
            "fn optional_postprocess_executors_skip_resource_work_when_effects_are_disabled(",
            "fn effect_stack_with_screen_space_reflection(",
        ],
    );
    assert_contains_all(
        "renderer guard child owns surface mesh shadow and deferred guards",
        &renderer,
        &[
            "use super::*;",
            "fn screen_space_ui_executor_uses_graph_attachment_ops_for_viewport_output(",
            "fn mesh_executor_requires_mesh_context_instead_of_nooping(",
            "fn shadow_atlas_executor_records_depth_only_pass_when_graph_resource_is_bound(",
            "fn deferred_lighting_executor_requires_renderer_context_instead_of_nooping(",
            "fn import_shadow_atlas_texture(",
        ],
    );

    for (path, source) in [
        (
            "graph_execution/render_pass_executor_registry/tests.rs",
            root.as_str(),
        ),
        (
            "graph_execution/render_pass_executor_registry/tests/registry_contracts.rs",
            registry.as_str(),
        ),
        (
            "graph_execution/render_pass_executor_registry/tests/postprocess_context_guards.rs",
            postprocess.as_str(),
        ),
        (
            "graph_execution/render_pass_executor_registry/tests/renderer_context_guards.rs",
            renderer.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the executor registry test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 01", &plan_01),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("graph execution docs", &graph_execution_doc),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "Render pass executor registry test owner split",
                "render_plan01_executor_registry_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs",
                "graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs",
                "graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/postprocess_context_guards.rs",
                "graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/renderer_context_guards.rs",
                "runtime_15_render_pass_executor_registry_tests_are_child_owners",
            ],
        );
    }
}
