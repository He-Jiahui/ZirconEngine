use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_builtin_postprocess_executors_are_folder_backed() {
    let parent = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs",
    );
    let frame_effects = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs",
    );
    let graph_resources = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs",
    );
    let resource_routing = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let plan_07 = format!(
        "{}\n{}",
        read_repo(
            "docs/plans/zircon_runtime/render/07/2026-07-09-postprocess-color-pipeline-output-records.md",
        ),
        render_index,
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let graph_execution_doc = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.md",
    );

    assert_contains_all(
        "builtin post-process executor parent mounts focused child owners",
        &parent,
        &[
            "mod frame_effects;",
            "mod graph_resources;",
            "mod resource_routing;",
            "use self::frame_effects::{",
            "use self::graph_resources::product_postprocess_executor;",
            "use self::resource_routing::{",
            "pub(super) fn bloom_postprocess_executor",
            "pub(super) fn screen_space_reflection_resolve_executor",
            "pub(super) fn uber_postprocess_executor",
        ],
    );
    for moved_owner in [
        "fn product_postprocess_executor",
        "fn require_graph_resource_by_name",
        "fn output_transfer_output_resource",
        "fn bloom_input_resource",
        "fn frame_post_process_effect_stack",
        "fn frame_uses_scene_velocity",
        "mod tests {",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "builtin_postprocess_executors.rs should delegate {moved_owner} to child owners"
        );
    }
    assert_contains_all(
        "frame-effects child owns frame effect predicates",
        &frame_effects,
        &[
            "fn frame_post_process_effect_stack",
            "pub(super) fn frame_uses_scene_velocity",
            "pub(super) fn frame_uses_taa",
            "pub(super) fn frame_uses_reconstructed_motion_vectors",
            "pub(super) fn frame_uses_depth_of_field",
            "pub(super) fn frame_uses_screen_space_reflection",
        ],
    );
    assert_contains_all(
        "graph-resources child owns post-process graph resource validation",
        &graph_resources,
        &[
            "pub(super) fn product_postprocess_executor",
            "fn require_graph_resource_by_name",
            "fn pass_resource_kind",
            "fn external_resource_type",
            "RenderGraphExternalResourceType::Buffer",
            "RenderGraphResourceKind::TransientTexture",
        ],
    );
    assert_contains_all(
        "resource-routing child owns terminal/bloom/uber resource routing tests",
        &resource_routing,
        &[
            "pub(super) fn output_transfer_output_resource",
            "pub(super) fn output_transfer_input_resource",
            "pub(super) fn bloom_input_resource",
            "pub(super) fn uber_input_resource",
            "fn output_transfer_executor_targets_terminal_input_when_declared",
            "fn bloom_executor_reads_motion_blurred_source_when_declared",
            "fn uber_executor_reads_blurred_source_when_declared",
        ],
    );

    for (path, source, budget) in [
        (
            "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs",
            parent.as_str(),
            800,
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs",
            frame_effects.as_str(),
            120,
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs",
            graph_resources.as_str(),
            160,
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs",
            resource_routing.as_str(),
            280,
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
        ("graph execution doc", graph_execution_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 07 built-in post-process executor owner split",
                "render_plan07_builtin_postprocess_executor_owner_split_static_passed",
                "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs",
                "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs",
                "graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs",
                "runtime_15_builtin_postprocess_executors_are_folder_backed",
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
