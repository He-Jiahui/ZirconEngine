use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_plugin_feature_compile_particle_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/plugin_feature_compile.rs");
    let particle = read_runtime_src("graphics/tests/plugin_feature_compile/particle.rs");

    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_framework_architecture =
        read_repo("docs/assets-and-rendering/render-framework-architecture.md");
    let rendering_plugin_options = read_repo("docs/zircon_plugins/rendering-plugin-options.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "plugin feature compile parent keeps generic compile contracts, fixtures, and child mount",
        &parent,
        &[
            "mod particle;",
            "fn default_pipeline_assets_do_not_embed_pluginized_advanced_builtin_features(",
            "fn compiled_pipeline_collects_enabled_plugin_feature_capability_requirements(",
            "fn gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph(",
            "fn builtin_smaa_terminal_aa_pass_compiles_after_output_transfer_when_requested(",
            "fn test_extract(",
        ],
    );

    for moved_anchor in [
        "fn particle_plugin_render_feature_adds_transparent_pass_to_default_pipeline(",
        "fn core_scene_particle_extract_adds_billboard_pass_without_plugin_feature_identity(",
        "fn compile_options_can_disable_core_scene_particle_billboard_pass(",
        "fn compile_options_can_disable_particle_plugin_feature_by_name(",
        "fn test_extract_with_particle_sprite(",
        "fn assert_particle_pass_uses_depth_read_color_write(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "plugin_feature_compile.rs should delegate `{moved_anchor}` to particle.rs"
        );
        assert!(
            particle.contains(moved_anchor),
            "plugin_feature_compile/particle.rs should own `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "particle compile child keeps particle descriptor, core-scene sprite fixture, and pass IO assertions",
        &particle,
        &[
            "particle_render_feature_descriptor",
            "RenderParticleSpriteSnapshot",
            "PostProcessGraphResourceNames",
            "RenderGraphResourceAccessKind",
            "use super::test_extract;",
        ],
    );

    for (path, source) in [
        ("graphics/tests/plugin_feature_compile.rs", parent.as_str()),
        (
            "graphics/tests/plugin_feature_compile/particle.rs",
            particle.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render compile test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        (
            "render framework architecture",
            render_framework_architecture.as_str(),
        ),
        (
            "rendering plugin options",
            rendering_plugin_options.as_str(),
        ),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plugin feature compile particle tests owner split",
                "render_plugin_feature_compile_particle_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/plugin_feature_compile.rs",
                "graphics/tests/plugin_feature_compile/particle.rs",
                "runtime_15_plugin_feature_compile_particle_tests_are_child_owner",
            ],
        );
    }
}
