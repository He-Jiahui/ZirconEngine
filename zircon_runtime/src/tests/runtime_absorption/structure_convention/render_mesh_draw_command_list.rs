use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_mesh_draw_command_list_is_folder_backed() {
    let parent =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs");
    let builder = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs",
    );
    let tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs",
    );
    let plan_02 = read_repo(
        "docs/plans/zircon_runtime/render/02/2026-07-09-mesh-draw-command-pipeline-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_convention = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let mesh_draw_command_doc = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.md",
    );

    assert_contains_all(
        "mesh draw command list parent mounts focused child owners",
        &parent,
        &[
            "mod builder;",
            "mod tests;",
            "pub(crate) use builder::",
            "pub(crate) struct MeshDrawCommandList",
            "pub(crate) struct MeshPassCommandBuffers",
            "fn indirect_batch_stats",
            "fn sort_mesh_draw_commands",
            "fn summarize_mesh_draw_commands",
        ],
    );
    for moved_owner in [
        "fn build_mesh_pass_command_buffers_from_batches",
        "fn add_cached_static_batch",
        "fn add_cached_or_rebuilt_phase",
        "fn append_dynamic_commands",
        "fn mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame",
        "mod tests {",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "mesh_draw_command_list.rs should delegate {moved_owner} to child owners"
        );
    }

    assert_contains_all(
        "mesh draw command builder owns batch expansion and cache reuse",
        &builder,
        &[
            "pub(crate) fn build_mesh_pass_command_buffers",
            "pub(crate) fn build_mesh_pass_command_buffers_cached",
            "pub(super) fn build_mesh_pass_command_buffers_from_batches",
            "pub(super) fn build_mesh_pass_command_buffers_from_batches_cached",
            "fn add_cached_static_batch",
            "fn add_cached_or_rebuilt_phase",
            "fn append_dynamic_commands",
        ],
    );
    assert_contains_all(
        "mesh draw command tests child owns behavior coverage",
        &tests,
        &[
            "fn mesh_draw_command_list_sorts_by_phase_then_sort_key",
            "fn mesh_batch_ref_emits_gpu_scene_instance_command",
            "fn mesh_pass_command_buffers_build_expected_phase_counts_from_batches",
            "fn mesh_pass_command_buffers_report_indirect_batch_stats_when_gpu_driven_supported",
            "fn mesh_pass_command_buffers_assign_cache_variants_by_pipeline_kind",
            "fn mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame",
            "fn mesh_pass_command_buffers_report_static_cache_invalidation_reasons",
        ],
    );

    for (path, source, budget) in [
        (
            "graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs",
            parent.as_str(),
            420,
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs",
            builder.as_str(),
            360,
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs",
            tests.as_str(),
            500,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the owner budget {budget}; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 02", plan_02.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_convention.as_str()),
        ("mesh draw command list doc", mesh_draw_command_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 02 mesh draw command list owner split",
                "render_plan02_mesh_draw_command_list_owner_split_static_passed",
                "graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs",
                "graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs",
                "runtime_15_mesh_draw_command_list_is_folder_backed",
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
