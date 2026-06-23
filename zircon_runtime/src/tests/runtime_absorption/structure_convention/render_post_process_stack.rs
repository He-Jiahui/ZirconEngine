use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_post_process_stack_is_folder_backed() {
    let post_process_mod = read_runtime_src("core/framework/render/post_process/mod.rs");
    let stack = read_runtime_src("core/framework/render/post_process/stack.rs");
    let graph_names =
        read_runtime_src("core/framework/render/post_process/graph_resource_names.rs");
    let tests_parent = read_runtime_src("core/framework/render/post_process/stack/tests.rs");
    let exposure_tests =
        read_runtime_src("core/framework/render/post_process/stack/tests/exposure.rs");
    let terminal_tests =
        read_runtime_src("core/framework/render/post_process/stack/tests/terminal_chain.rs");
    let ssr_tests = read_runtime_src(
        "core/framework/render/post_process/stack/tests/screen_space_reflection.rs",
    );
    let temporal_tests =
        read_runtime_src("core/framework/render/post_process/stack/tests/temporal_history.rs");
    let effect_stack_tests =
        read_runtime_src("core/framework/render/post_process/stack/tests/effect_stack.rs");
    let plan_07 = read_repo("docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let post_process_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/post_process/index.md");

    assert_contains_all(
        "post-process module exposes stack and graph-resource owners",
        &post_process_mod,
        &[
            "mod graph_resource_names;",
            "mod stack;",
            "pub use graph_resource_names::PostProcessGraphResourceNames;",
            "pub use stack::PostProcessStackDescriptor;",
        ],
    );
    assert_contains_all(
        "stack parent keeps descriptor construction and delegates tests",
        &stack,
        &[
            "pub struct PostProcessStackDescriptor",
            "pub fn from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale",
            "PostProcessPassGraph::validate_stack(self)",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );
    for moved_owner in [
        "pub struct PostProcessGraphResourceNames",
        "mod tests {",
        "fn manual_exposure_declares_resolve_without_histogram",
        "fn screen_space_reflection_declares_specular_occlusion_and_resolve_inputs",
        "fn effect_stack_motion_blur_declares_depth_and_reconstructed_motion_vector_inputs",
    ] {
        assert!(
            !stack.contains(moved_owner),
            "post_process/stack.rs should delegate {moved_owner} to graph_resource_names.rs or stack/tests/*"
        );
    }
    assert_contains_all(
        "graph resource owner keeps render-graph resource name contract",
        &graph_names,
        &[
            "pub struct PostProcessGraphResourceNames;",
            "pub const SCENE_COLOR: &'static str = \"scene-color\";",
            "pub const EXPOSURE_CURRENT: &'static str = \"history.current.exposure\";",
            "pub const SCREEN_SPACE_REFLECTION_HISTORY: &'static str",
            "pub const FINAL_COLOR: &'static str = \"final-color\";",
        ],
    );
    assert_contains_all(
        "stack test parent mounts behavior-domain children",
        &tests_parent,
        &[
            "mod effect_stack;",
            "mod exposure;",
            "mod screen_space_reflection;",
            "mod temporal_history;",
            "mod terminal_chain;",
            "fn expected_uber_effect_stack_outputs",
        ],
    );
    assert_contains_all(
        "exposure tests own exposure graph contracts",
        &exposure_tests,
        &[
            "fn manual_exposure_declares_resolve_without_histogram",
            "fn default_stack_declares_light_list_for_uber_cluster_bind_group",
            "fn histogram_exposure_declares_histogram_before_resolve",
        ],
    );
    assert_contains_all(
        "terminal tests own terminal AA and dynamic-resolution contracts",
        &terminal_tests,
        &[
            "fn fxaa_terminal_anti_alias_routes_output_transfer_through_terminal_input",
            "fn smaa_terminal_anti_alias_routes_output_transfer_through_terminal_input",
            "fn dynamic_resolution_declares_upscale_before_output_transfer",
        ],
    );
    assert_contains_all(
        "SSR tests own reflection graph contracts",
        &ssr_tests,
        &[
            "fn screen_space_reflection_declares_specular_occlusion_and_resolve_inputs",
            "fn screen_space_reflection_resolve_temporal_declares_history_and_motion_vector_inputs",
            "fn screen_space_reflection_resolve_feeds_scene_composite_before_output_transfer",
        ],
    );
    assert_contains_all(
        "temporal tests own TAA/history stripping contracts",
        &temporal_tests,
        &[
            "fn taa_resolve_declares_history_velocity_and_output_transfer_input",
            "fn without_history_resources_disables_taa_and_restores_scene_color_input",
            "fn without_history_resources_keeps_scene_velocity_for_motion_blur",
        ],
    );
    assert_contains_all(
        "effect stack tests own split effect routing contracts",
        &effect_stack_tests,
        &[
            "fn enabled_effect_stack_declares_tonemapped_for_uber_descriptor",
            "fn effect_stack_depth_of_field_feeds_uber_from_dedicated_intermediate",
            "fn effect_stack_blur_feeds_uber_from_dedicated_intermediate",
            "fn effect_stack_motion_blur_declares_depth_and_reconstructed_motion_vector_inputs",
            "fn effect_stack_omits_depth_of_field_intermediate_outputs_when_dof_is_disabled",
        ],
    );

    let total_stack_tests = [
        exposure_tests.as_str(),
        terminal_tests.as_str(),
        ssr_tests.as_str(),
        temporal_tests.as_str(),
        effect_stack_tests.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("\nfn ").count())
    .sum::<usize>();
    assert_eq!(
        total_stack_tests, 17,
        "post-process stack child owners should preserve all 17 moved stack tests"
    );

    for (path, source, budget) in [
        (
            "core/framework/render/post_process/stack.rs",
            stack.as_str(),
            900,
        ),
        (
            "core/framework/render/post_process/graph_resource_names.rs",
            graph_names.as_str(),
            200,
        ),
        (
            "core/framework/render/post_process/stack/tests.rs",
            tests_parent.as_str(),
            100,
        ),
        (
            "core/framework/render/post_process/stack/tests/exposure.rs",
            exposure_tests.as_str(),
            800,
        ),
        (
            "core/framework/render/post_process/stack/tests/terminal_chain.rs",
            terminal_tests.as_str(),
            800,
        ),
        (
            "core/framework/render/post_process/stack/tests/screen_space_reflection.rs",
            ssr_tests.as_str(),
            800,
        ),
        (
            "core/framework/render/post_process/stack/tests/temporal_history.rs",
            temporal_tests.as_str(),
            800,
        ),
        (
            "core/framework/render/post_process/stack/tests/effect_stack.rs",
            effect_stack_tests.as_str(),
            800,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the owner budget {budget}; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 07", plan_07.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("post-process module doc", post_process_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 07 post-process stack owner split",
                "render_plan07_post_process_stack_owner_split_static_passed",
                "core/framework/render/post_process/graph_resource_names.rs",
                "core/framework/render/post_process/stack/tests/effect_stack.rs",
                "runtime_15_post_process_stack_is_folder_backed",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
