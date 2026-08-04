use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_graph_builder_compile_is_child_owner() {
    let parent = read_runtime_src("render_graph/builder.rs");
    let compile = read_runtime_src("render_graph/builder/compile.rs");
    let plan_01 = read_repo(
        "docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let builder_doc = read_repo("docs/zircon_runtime/render_graph/builder.md");

    assert_contains_all(
        "RenderGraphBuilder parent keeps authoring surface and child mount",
        &parent,
        &[
            "mod compile;",
            "pub struct RenderGraphBuilder",
            "pub fn add_pass(",
            "pub fn create_texture(",
            "pub fn create_buffer(",
            "pub fn import_external_resource_with_usage_and_binding(",
            "pub fn write_storage_texture(",
            "fn add_resource_access(",
        ],
    );
    for moved_compile_owner in [
        "pub fn compile(",
        "fn validate_unique_resource_names(",
        "fn validate_write_dependencies(",
        "fn infer_resource_dependencies(",
        "fn topological_order(",
        "fn validate_reads_have_ordered_producers(",
        "fn cull_passes(",
        "fn resource_lifetimes(",
        "struct LatestWriter",
    ] {
        assert!(
            !parent.contains(moved_compile_owner),
            "render_graph/builder.rs should delegate `{moved_compile_owner}` to builder/compile.rs"
        );
        assert!(
            compile.contains(moved_compile_owner),
            "render_graph/builder/compile.rs should own `{moved_compile_owner}`"
        );
    }
    assert_contains_all(
        "RenderGraphBuilder compile child owns graph compilation analysis",
        &compile,
        &[
            "impl RenderGraphBuilder",
            "CompiledRenderGraph::new(",
            "RenderGraphPassResourceAccess",
            "RenderGraphError::WriteAfterWriteMissingDependency",
            "RenderGraphError::MissingCullRoot",
            "RenderGraphResourceLifetime",
        ],
    );

    for (path, source) in [
        ("render_graph/builder.rs", parent.as_str()),
        ("render_graph/builder/compile.rs", compile.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R1.4 production owner budget after the compile split; got {line_count}"
        );
    }

    for (label, source) in [
        ("Plan 01", plan_01.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render graph builder doc", builder_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "RenderGraphBuilder compile owner split",
                "render_graph_builder_compile_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "zircon_runtime/src/render_graph/builder.rs",
                "zircon_runtime/src/render_graph/builder/compile.rs",
                "runtime_15_render_graph_builder_compile_is_child_owner",
            ],
        );
    }
}
