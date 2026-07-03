use super::*;

const STATUS: &str =
    "render_plan08_virtual_geometry_asset_payload_decode_static_passed_cargo_deferred";

#[test]
fn runtime_15_virtual_geometry_asset_payload_decode_is_wired() {
    let core_output =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/extract_output.rs");
    let frame_context = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs",
    );
    let context_builder =
        read_runtime_src("graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs");
    let snapshot_builder = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs",
    );
    let resident_upload = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_resident_upload.rs",
    );
    let nanite_mod =
        read_repo("zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/mod.rs");
    let page_payload = read_repo(
        "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/page_payload.rs",
    );
    let automatic_extract = read_repo(
        "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/automatic_extract.rs",
    );
    let provider = read_repo("zircon_plugins/virtual_geometry/runtime/src/provider.rs");
    let imported_extract_test = read_repo(
        "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/virtual_geometry_imported_extract.rs",
    );
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let vg_snapshot_doc =
        read_repo("docs/zircon_runtime/core/framework/render/virtual_geometry_debug_snapshot.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "nanite automatic extract owns cooked page payload decode in a separate module",
        &format!("{nanite_mod}{page_payload}{automatic_extract}"),
        &[
            "mod page_payload;",
            "render_page_payloads_for_asset",
            "CLUSTER_PAYLOAD_MAGIC",
            "RenderVirtualGeometryPagePayloadVertex",
            "append_triangle_range_vertices",
            "payload_u32_words",
            "from_model_primitive",
            "primitive.virtual_geometry.clone()",
            "page_remap",
            "resident_page_payloads",
            "render_page_payloads_decode_cooked_triangle_vertices_with_global_page_ids",
        ],
    );
    assert_contains_all(
        "runtime extract output and frame context carry decoded page payload sidecar",
        &format!("{core_output}{frame_context}{context_builder}{snapshot_builder}"),
        &[
            "RenderVirtualGeometryPagePayload",
            "resident_page_payloads",
            "resident_page_payloads().to_vec()",
            "virtual_geometry_resident_page_payloads",
            "context.virtual_geometry_resident_page_payloads().to_vec()",
        ],
    );
    assert_contains_all(
        "provider and imported model test expose decoded payload evidence",
        &format!("{provider}{imported_extract_test}"),
        &[
            "output.resident_page_payloads().to_vec()",
            "output.resident_page_payloads().len()",
            "vertices.len(), 3",
        ],
    );
    assert_contains_all(
        "resident upload remains the only render-side consumer of decoded payloads",
        &resident_upload,
        &[
            "resident_page_payloads",
            "append_page_payload_cluster_words",
        ],
    );

    for (path, source, budget) in [
        (
            "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/page_payload.rs",
            page_payload.as_str(),
            220,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_asset_payload_decode.rs",
            include_str!("virtual_geometry_asset_payload_decode.rs"),
            220,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("VG debug snapshot doc", vg_snapshot_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "VirtualGeometry asset payload decode",
                STATUS,
                "runtime_15_virtual_geometry_asset_payload_decode_is_wired",
            ],
        );
    }
}
