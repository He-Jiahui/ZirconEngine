use super::{assert_contains_all, assert_contains_all_exact, read_repo, read_runtime_src};

#[test]
fn runtime_15_frame_extract_geometry_is_child_owner() {
    let root = read_runtime_src("core/framework/render/frame_extract.rs");
    let geometry = read_runtime_src("core/framework/render/frame_extract/geometry.rs");

    let plan_09 = read_repo(
        "docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let frame_extract_doc = read_repo("docs/zircon_runtime/core/framework/render/frame_extract.md");

    assert_contains_all(
        "frame_extract root mounts and re-exports the geometry child owner",
        &root,
        &[
            "mod geometry;",
            "pub use geometry::{GeometryExtract, GeometryPhaseInput, StaticMeshBatchExtract};",
            "pub struct RenderFrameExtract",
            "pub fn from_snapshot(",
            "pub fn phase_queue_summary(&self) -> RenderFramePhaseQueueSummary",
        ],
    );

    for moved_geometry_anchor in [
        "pub struct GeometryPhaseInput",
        "pub struct GeometryExtract",
        "pub struct StaticMeshBatchExtract",
        "struct StaticMeshBatchKey",
        "fn build_static_mesh_batches(",
    ] {
        assert!(
            !root.contains(moved_geometry_anchor),
            "frame_extract.rs should delegate geometry owner `{moved_geometry_anchor}` to geometry.rs"
        );
    }

    assert_contains_all(
        "geometry child owns mesh phase input, frame geometry DTOs, and static batching",
        &geometry,
        &[
            "pub struct GeometryPhaseInput",
            "pub struct GeometryExtract",
            "pub struct StaticMeshBatchExtract",
            "struct StaticMeshBatchKey",
            "fn build_static_mesh_batches(",
            "pub fn from_meshes(",
            "pub fn rebuild_phase_queue(",
            "pub fn phase_queue_summary(&self) -> RenderPhaseQueueSummary",
        ],
    );

    for (path, source) in [
        ("frame_extract.rs", root.as_str()),
        ("frame_extract/geometry.rs", geometry.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the geometry split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("frame extract docs", frame_extract_doc.as_str()),
    ] {
        assert_contains_all_exact(
            label,
            doc,
            &[
                "frame extract geometry owner split",
                "render_plan09_frame_extract_geometry_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "core/framework/render/frame_extract.rs",
                "core/framework/render/frame_extract/geometry.rs",
                "runtime_15_frame_extract_geometry_is_child_owner",
            ],
        );
    }
}
