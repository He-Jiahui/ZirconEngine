use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_mesh_pass_processors_are_folder_backed() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs");
    let tests =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs");
    let plan_02 = read_repo("docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");
    let module_convention = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "mesh pass processor root stays navigational",
        &parent,
        &[
            "mod depth_prepass;",
            "mod opaque_base;",
            "mod shadow;",
            "mod taa_reactive_mask;",
            "mod tests;",
            "mod transparent;",
            "mod velocity;",
            "pub(crate) use depth_prepass::DepthPrepassProcessor;",
        ],
    );
    for moved_owner in [
        "fn processors_emit_expected_mesh_phases",
        "fn render_mesh_draw_processor_depth_prepass_filters_transparent",
        "fn render_mesh_draw_processor_shadow_excludes_non_casters_and_picks_alpha_mask_variant",
        "MeshPipelineVariantRegistry",
        "PrimitiveRelevance",
        "mod tests {",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "processors/mod.rs should delegate {moved_owner} to child owners"
        );
    }

    assert_contains_all(
        "mesh pass processor tests child owns processor behavior coverage",
        &tests,
        &[
            "fn processors_emit_expected_mesh_phases",
            "fn render_mesh_draw_processor_depth_prepass_filters_transparent",
            "fn render_mesh_draw_processor_opaque_preserves_material_slots_for_fallback_shader_selection",
            "fn render_mesh_draw_processor_shadow_excludes_non_casters_and_picks_alpha_mask_variant",
            "fn velocity_processor_requires_velocity_history_and_previous_transform",
            "fn shadow_processor_respects_shadow_view_visibility",
        ],
    );

    for (path, source, budget) in [
        (
            "graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs",
            parent.as_str(),
            80,
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs",
            tests.as_str(),
            430,
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
        ("mesh pass doc", mesh_pass_doc.as_str()),
        ("module convention doc", module_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 02 mesh pass processor tests owner split",
                "render_plan02_mesh_pass_processor_tests_owner_split_static_passed",
                "graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs",
                "runtime_15_mesh_pass_processors_are_folder_backed",
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
