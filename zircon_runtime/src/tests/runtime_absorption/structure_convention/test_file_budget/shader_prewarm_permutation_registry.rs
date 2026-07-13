use super::*;

const STATUS: &str =
    "render_plan08_shader_permutation_registry_overlay_focused_tests_passed_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_permutation_registry_overlay_is_wired() {
    let args = read_runtime_src("bin/zircon_shader_prewarm/args.rs");
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let permutation_registry =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/permutation_registry.rs");
    let build_tool = read_zircon_build_sources();
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "shader prewarm CLI accepts project/plugin permutation registries",
        &args,
        &[
            "pub permutation_registries: Vec<PathBuf>",
            "\"--shader-permutation-registry\"",
            "shader_prewarm_args_parse_shader_permutation_registry_path",
            "pub(crate) fn normalized_custom_geometry_source_token",
            "pub(crate) fn normalized_custom_shading_model_token",
        ],
    );
    assert_contains_all(
        "shader prewarm run merges permutation registries before manifest expansion",
        &run,
        &[
            "shader_permutation_registry_paths",
            "ShaderPrewarmPermutationRegistryOverlay",
            "merge_into",
            "&geometry_sources",
            "&shading_model_ids",
        ],
    );
    assert_contains_all(
        "manifest root exposes permutation registry owner without root bloat",
        &manifest,
        &["pub(crate) mod permutation_registry;"],
    );
    assert_contains_all(
        "permutation registry owner reads explicit and asset-root registry inputs",
        &permutation_registry,
        &[
            "SHADER_PERMUTATION_REGISTRY_FILE",
            "shader_permutation_registry.json",
            "shader_permutation_registry_paths",
            "geometry_source_ids",
            "shading_model_ids",
            "GEOMETRY_SOURCE_PLUGIN_ID_START",
            "SHADING_MODEL_PLUGIN_ID_START",
            "shader_prewarm_permutation_registry_merges_custom_geometry_and_shading_ids",
            "shader_prewarm_permutation_registry_discovers_asset_root_registry",
        ],
    );
    assert_contains_all(
        "zircon build CLI forwards explicit permutation registries",
        &build_tool,
        &[
            "shader_permutation_registries: tuple[Path, ...]",
            "\"--shader-permutation-registry\"",
            "resolve_optional_paths(",
            "args.shader_permutation_registry",
            "auto-discover shader_permutation_registry.json",
        ],
    );
    assert_contains_all(
        "shader prewarm command helper forwards registry paths",
        &build_prewarm,
        &[
            "shader permutation registries",
            "for registry in shader_permutation_registry_paths_for_prewarm(config)",
            "\"--shader-permutation-registry\"",
        ],
    );
    assert_contains_all(
        "python focused test covers registry forwarding",
        &build_tests,
        &[
            "build_shader_prewarm_command",
            "test_build_command_forwards_shader_permutation_registries",
            "shader_permutation_registries",
        ],
    );

    for (path, source) in [
        ("bin/zircon_shader_prewarm/args.rs", args.as_str()),
        ("bin/zircon_shader_prewarm/run.rs", run.as_str()),
        ("bin/zircon_shader_prewarm/manifest.rs", manifest.as_str()),
        (
            "bin/zircon_shader_prewarm/manifest/permutation_registry.rs",
            permutation_registry.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("build tool doc", build_tool_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Shader permutation registry overlay",
                STATUS,
                "shader_permutation_registry_paths",
                "shader_prewarm_permutation_registry_merges_custom_geometry_and_shading_ids",
                "runtime_15_shader_prewarm_permutation_registry_overlay_is_wired",
            ],
        );
    }
}
