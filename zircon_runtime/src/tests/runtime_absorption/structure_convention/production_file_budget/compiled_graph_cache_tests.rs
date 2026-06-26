use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_compiled_graph_cache_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/pipeline/compiled_graph_cache.rs");
    let tests = read_runtime_src("graphics/pipeline/compiled_graph_cache/tests.rs");

    let plan_01 = read_repo("docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let builder_doc = read_repo("docs/zircon_runtime/render_graph/builder.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "compiled graph cache parent keeps production cache API and child test mount",
        &parent,
        &[
            "pub struct CompiledGraphCacheKey",
            "pub fn extract_compile_fingerprint(",
            "pub struct CompiledGraphCache",
            "pub fn get_or_compile_with_status(",
            "pub fn invalidate_pipeline(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_anchor in [
        "fn compiled_render_pipeline_cache_hits_on_identical_key(",
        "fn compiled_render_pipeline_cache_reports_lookup_status(",
        "fn render_graph_compile_frame_fingerprint_tracks_compile_extract_inputs(",
        "fn render_graph_compile_frame_fingerprint_tracks_camera_target_and_stack_inputs(",
        "fn compiled_render_pipeline_cache_key_tracks_texture_target_format_class(",
        "fn compiled_render_pipeline_cache_evicts_least_recently_used_entry(",
        "trait DescriptorTestExt",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "compiled_graph_cache.rs should delegate `{moved_anchor}` to compiled_graph_cache/tests.rs"
        );
        assert!(
            tests.contains(moved_anchor),
            "compiled_graph_cache/tests.rs should own `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "compiled graph cache tests child keeps cache hit, fingerprint, target, and LRU coverage",
        &tests,
        &[
            "CompiledGraphCache",
            "CompiledGraphCacheKey",
            "RenderGraphCompileCameraTargetFingerprint::from_target_and_view_size",
            "RenderGraphCompileTextureTargetFormat::Rgba8UnormSrgb",
            "RenderGraphBuilder::new(\"cache-test\").compile().unwrap()",
            "DescriptorTestExt",
        ],
    );

    for (path, source) in [
        ("graphics/pipeline/compiled_graph_cache.rs", parent.as_str()),
        (
            "graphics/pipeline/compiled_graph_cache/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R1.4 production/test owner budget after the tests split; got {line_count}"
        );
    }

    for (label, source) in [
        ("Plan 01", plan_01.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render graph builder doc", builder_doc.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "CompiledGraphCache tests owner split",
                "render_plan01_compiled_graph_cache_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/pipeline/compiled_graph_cache.rs",
                "graphics/pipeline/compiled_graph_cache/tests.rs",
                "runtime_15_compiled_graph_cache_tests_are_child_owner",
            ],
        );
    }
}
