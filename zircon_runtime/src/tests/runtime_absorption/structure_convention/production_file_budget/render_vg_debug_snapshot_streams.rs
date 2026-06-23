use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_vg_debug_snapshot_stream_types_are_child_owner() {
    let parent =
        read_runtime_src("core/framework/render/virtual_geometry_debug_snapshot_streams.rs");
    let types =
        read_runtime_src("core/framework/render/virtual_geometry_debug_snapshot_streams/types.rs");
    let diagnostics = read_runtime_src(
        "core/framework/render/virtual_geometry_debug_snapshot_streams/diagnostics.rs",
    );
    let metrics = read_runtime_src(
        "core/framework/render/virtual_geometry_debug_snapshot_streams/metrics.rs",
    );

    let plan_02 = read_repo("docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let advanced_doc = read_repo("docs/zircon_runtime/core/framework/render/advanced.md");
    let visibility_doc = read_repo("docs/zircon_runtime/graphics/visibility.md");

    assert_contains_all(
        "VG stream parent keeps decode orchestration and child mounts",
        &parent,
        &[
            "mod diagnostics;",
            "mod metrics;",
            "mod types;",
            "pub use types::*;",
            "impl RenderVirtualGeometryDebugSnapshot {",
            "pub fn try_decode_debug_readback_streams(",
            "pub fn try_decode_node_and_cluster_cull_word_streams(",
            "pub fn try_decode_render_path_word_streams(",
            "pub fn try_decode_visbuffer64_readback_stream(",
        ],
    );

    for moved_type in [
        "pub struct RenderVirtualGeometryNodeAndClusterCullWordStreams",
        "pub struct RenderVirtualGeometryRenderPathWordStreams",
        "pub struct RenderVirtualGeometryVisBuffer64ReadbackStream",
        "pub struct RenderVirtualGeometryDebugSnapshotReadbackStreams",
        "pub enum RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError",
        "pub struct RenderVirtualGeometryDebugSnapshotReadbackStreamSummary",
    ] {
        assert!(
            !parent.contains(moved_type),
            "VG stream parent should not own moved type `{moved_type}`"
        );
        assert!(
            types.contains(moved_type),
            "VG stream types owner should contain moved type `{moved_type}`"
        );
    }

    assert_contains_all(
        "existing stream child owners remain attached to the same types",
        &(diagnostics + &metrics),
        &[
            "RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeDiagnostic",
            "RenderVirtualGeometryDebugSnapshotReadbackStreamSection",
            "impl RenderVirtualGeometryNodeAndClusterCullWordStreams",
            "impl RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint",
        ],
    );

    for (path, source) in [
        (
            "core/framework/render/virtual_geometry_debug_snapshot_streams.rs",
            parent.as_str(),
        ),
        (
            "core/framework/render/virtual_geometry_debug_snapshot_streams/types.rs",
            types.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 02", &plan_02),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("advanced render docs", &advanced_doc),
        ("visibility docs", &visibility_doc),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "VG debug snapshot stream types owner split",
                "render_plan02_vg_debug_snapshot_stream_types_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "core/framework/render/virtual_geometry_debug_snapshot_streams.rs",
                "core/framework/render/virtual_geometry_debug_snapshot_streams/types.rs",
                "runtime_15_vg_debug_snapshot_stream_types_are_child_owner",
            ],
        );
    }
}
