use super::*;

const STATUS: &str =
    "render_plan08_virtual_geometry_product_draw_source_cargo_wrapper_wgpu_passed_renderdoc_deferred";
const READBACK_STATUS: &str =
    "render_plan08_virtual_geometry_product_draw_source_readback_passed_targeted_cargo";
const DEFAULT_FEATURE_STATUS: &str =
    "render_plan08_virtual_geometry_product_draw_source_default_features_wgpu_passed_renderdoc_deferred";
const PAGE_CLUSTER_PRODUCT_STATUS: &str =
    "render_plan08_virtual_geometry_page_cluster_product_execution_wgpu_passed_renderdoc_deferred";
const PAGE_CLUSTER_PRODUCT_DEFAULT_FEATURE_STATUS: &str =
    "render_plan08_virtual_geometry_page_cluster_product_default_features_wgpu_passed_renderdoc_deferred";
const PAGE_CLUSTER_PRODUCT_PNG_STATUS: &str =
    "render_plan08_virtual_geometry_page_cluster_product_readback_png_passed_renderdoc_deferred";

#[test]
fn runtime_15_virtual_geometry_product_draw_source_is_wired() {
    let context_builder =
        read_runtime_src("graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs");
    let provider_contract =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/provider.rs");
    let product_fixture = read_runtime_src("graphics/tests/plugin_render_feature_fixtures.rs");
    let product_provider = read_runtime_src(
        "graphics/tests/plugin_render_feature_fixtures/virtual_geometry_provider.rs",
    );
    let product_test =
        read_runtime_src("graphics/tests/render_product_mesh_cache/virtual_geometry.rs");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "submit context chooses automatic VG only after authored extract is absent",
        &context_builder,
        &[
            "let authored_virtual_geometry_present = authored_virtual_geometry_extract.is_some();",
            "if virtual_geometry_enabled && !authored_virtual_geometry_present",
            "build_automatic_virtual_geometry_extract(framework, sized_extract)",
            "registration.provider().build_extract_from_meshes",
            "asset_manager.load_model_asset(model_id)",
            "RenderVirtualGeometryPayloadSource::AutomaticFallback",
        ],
    );
    assert_contains_all(
        "runtime provider contract exposes model-backed automatic extract",
        &provider_contract,
        &[
            "fn build_extract_from_meshes",
            "_load_model: &mut dyn FnMut(ResourceId) -> Option<ModelAsset>",
        ],
    );
    assert_contains_all(
        "product fixture provider can build VG extract from model assets",
        &format!("{product_fixture}{product_provider}"),
        &[
            "mod virtual_geometry_provider;",
            "test_virtual_geometry_extract_from_model_meshes",
            "append_test_virtual_geometry_asset",
            "primitive.virtual_geometry",
            "RenderVirtualGeometryExtract",
            "source_model: Some(source_model)",
        ],
    );
    assert_contains_all(
        "product fixture registers VG geometry source descriptor for WGPU pipeline creation",
        &product_fixture,
        &[
            "virtual_geometry_geometry_source_descriptors",
            "GeometrySourceDescriptor",
            "custom:virtual_geometry",
            "GeometrySourceBindingKind::VirtualGeometryPages",
            "GeometrySourceBindingKind::VirtualGeometryClusters",
            "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY",
            "new_with_plugin_render_extensions_and_shading_models",
            "new_with_plugin_render_extensions_and_solari_and_shading_models",
        ],
    );
    assert_contains_all(
        "product test uses registered ModelAsset draw-source instead of authored extract",
        &product_test,
        &[
            "render_product_virtual_geometry_model_asset_uses_automatic_draw_source",
            "register_virtual_geometry_model_revision",
            "automatic_virtual_geometry_model_extract",
            "GeometryExtract::from_meshes",
            "cook_virtual_geometry_from_mesh",
            "assign_virtual_geometry_vertex_ordinals",
            "CameraRenderDescriptor::from_camera_payload",
            "RenderCameraClear::Color(Vec4::ZERO)",
            "DisplayMode::Shaded",
            "unlit_virtual_geometry_material",
            "capture_frame(viewport)",
            "assert_virtual_geometry_capture_visible(&frame)",
            "RenderVirtualGeometryPayloadSource::AutomaticFallback",
        ],
    );
    assert_contains_all(
        "product test proves VG page/cluster bindings through a visible frame",
        &product_test,
        &[
            "render_product_virtual_geometry_page_cluster_bindings_drive_visible_frame",
            PAGE_CLUSTER_PRODUCT_STATUS,
            "export_virtual_geometry_page_cluster_product_png",
            PAGE_CLUSTER_PRODUCT_PNG_STATUS,
            "runtime_render_plan08_virtual_geometry_page_cluster_product_20260703.png",
            "save_virtual_geometry_product_frame",
            "capture_automatic_virtual_geometry_product_frame",
            "assert_virtual_geometry_page_cluster_product_bindings_executed",
            "last_virtual_geometry_input_page_count",
            "last_virtual_geometry_visible_cluster_count",
            "last_virtual_geometry_execution_resident_segment_count",
            "last_virtual_geometry_execution_pending_segment_count",
            "last_virtual_geometry_visbuffer64_entry_count",
            "last_virtual_geometry_hardware_rasterization_record_count",
        ],
    );

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "VirtualGeometry product draw-source",
                STATUS,
                "runtime_15_virtual_geometry_product_draw_source_is_wired",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "VirtualGeometry product draw-source default-feature WGPU backfill",
                DEFAULT_FEATURE_STATUS,
                "render_product_virtual_geometry_model_asset_uses_automatic_draw_source",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("render product doc", render_product_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "VirtualGeometry product draw-source readback fixture",
                READBACK_STATUS,
                "render_product_virtual_geometry_model_asset_uses_automatic_draw_source",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "VirtualGeometry page/cluster product execution",
                PAGE_CLUSTER_PRODUCT_STATUS,
                "render_product_virtual_geometry_page_cluster_bindings_drive_visible_frame",
                PAGE_CLUSTER_PRODUCT_PNG_STATUS,
                "product readback PNG",
                "runtime_15_virtual_geometry_product_draw_source_is_wired",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "VirtualGeometry page/cluster product default-feature WGPU backfill",
                PAGE_CLUSTER_PRODUCT_DEFAULT_FEATURE_STATUS,
                "render_product_virtual_geometry_page_cluster_bindings_drive_visible_frame",
                "runtime_15_virtual_geometry_product_draw_source_is_wired",
            ],
        );
    }

    for (path, source, budget) in [
        (
            "graphics/tests/plugin_render_feature_fixtures.rs",
            product_fixture.as_str(),
            800,
        ),
        (
            "graphics/tests/plugin_render_feature_fixtures/virtual_geometry_provider.rs",
            product_provider.as_str(),
            800,
        ),
        (
            "graphics/tests/render_product_mesh_cache/virtual_geometry.rs",
            product_test.as_str(),
            800,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_product_draw_source.rs",
            include_str!("virtual_geometry_product_draw_source.rs"),
            260,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }
}
