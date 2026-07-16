use super::*;

const STATUS: &str = "render_plan08_virtual_geometry_meshlet_vertex_ordinal_direct_binary_asset_shader_passed_renderdoc_deferred";
const PROJECT_ASSET_MANAGER_FIXTURE_STATUS: &str = "render_plan08_virtual_geometry_project_asset_manager_fixture_cargo_wrapper_passed_renderdoc_deferred";

#[test]
fn runtime_15_virtual_geometry_meshlet_vertex_ordinal_is_wired() {
    let primitive = read_runtime_src("asset/assets/model/primitive.rs");
    let primitive_importer =
        read_runtime_src("asset/importer/ingest/primitive_from_indexed_mesh.rs");
    let mesh_asset = read_runtime_src("asset/assets/mesh/mesh_asset.rs");
    let virtual_geometry_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_geometry_virtual_geometry.wgsl");
    let shader_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs");
    let shader_source_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs",
    );
    let model_tests = read_runtime_src("asset/tests/assets/model.rs");
    let mesh_conversion_tests = read_runtime_src("asset/tests/assets/mesh/conversion_import.rs");
    let importer_tests = read_runtime_src("asset/tests/assets/importer.rs");
    let gltf_importer_tests = read_runtime_src("asset/tests/assets/gltf_importer.rs");
    let project_asset_manager_tests =
        read_runtime_src("asset/tests/pipeline/manager/model_import.rs");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let model_doc = read_repo("docs/zircon_runtime/asset/assets/model.md");
    let render_assets_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "model primitive owns VG vertex ordinal encoding",
        &primitive,
        &[
            "VIRTUAL_GEOMETRY_VERTEX_ORDINAL_LOW_JOINT_SLOT",
            "VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_JOINT_SLOT",
            "assign_virtual_geometry_vertex_ordinals",
            "encode_virtual_geometry_vertex_ordinal",
            "decode_virtual_geometry_vertex_ordinal",
            "ordinal >> VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT",
        ],
    );
    assert_contains_all(
        "import and mesh conversion paths standardize VG ordinals",
        &format!("{primitive_importer}{mesh_asset}"),
        &[
            "primitive.assign_virtual_geometry_vertex_ordinals();",
            "if primitive.virtual_geometry.is_none()",
            "let mut primitive = primitive.clone();",
            "primitive.assign_virtual_geometry_vertex_ordinals();",
        ],
    );
    assert_contains_all(
        "WGSL and shader assembly unpack the 16+16 ordinal",
        &format!("{virtual_geometry_wgsl}{shader_source}{shader_source_tests}"),
        &[
            "ZR_VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT",
            "v.joints.x | (v.joints.y << ZR_VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT)",
            "mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings",
        ],
    );
    assert_contains_all(
        "tests cover primitive, mesh conversion, OBJ and GLTF importer ordinal evidence",
        &format!("{model_tests}{mesh_conversion_tests}{importer_tests}{gltf_importer_tests}"),
        &[
            "virtual_geometry_vertex_ordinals_pack_into_joint_index_slots",
            "virtual_geometry_vertex_ordinals_do_not_rewrite_non_vg_primitives",
            "MeshAttributeValues::Uint16x4(vec![[0, 0, 0, 0], [1, 0, 0, 0], [2, 0, 0, 0]])",
            "assert_virtual_geometry_vertex_ordinals",
            "decode_virtual_geometry_vertex_ordinal",
        ],
    );
    assert_contains_all(
        "ProjectAssetManager expected fixture consumes the same VG ordinal helper",
        &project_asset_manager_tests,
        &[
            "asset_manager_imports_model_toml_with_virtual_geometry_payload",
            "expected_model.primitives[0].assign_virtual_geometry_vertex_ordinals();",
            "assert_eq!(loaded, expected_model);",
        ],
    );

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("model doc", model_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "VirtualGeometry meshlet vertex ordinal",
                STATUS,
                "runtime_15_virtual_geometry_meshlet_vertex_ordinal_is_wired",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("model doc", model_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                PROJECT_ASSET_MANAGER_FIXTURE_STATUS,
                "asset_manager_imports_model_toml_with_virtual_geometry_payload",
                "05:27:25 +08:00",
            ],
        );
    }

    let line_count = include_str!("virtual_geometry_meshlet_vertex_ordinal.rs")
        .lines()
        .count();
    assert!(
        line_count < 220,
        "virtual_geometry_meshlet_vertex_ordinal.rs should stay below the Runtime 15 test budget; got {line_count} lines"
    );
}
