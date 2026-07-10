use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_shader_geometry_source_descriptor_contract_is_complete() {
    let geometry_source = read_runtime_src("core/framework/render/shader/geometry_source.rs");
    let shader_mod = read_runtime_src("core/framework/render/shader/mod.rs");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert!(
        !geometry_source.contains("wgpu::"),
        "framework GeometrySource contract must remain backend-neutral"
    );

    assert_contains_all(
        "GeometrySource descriptor owner",
        &geometry_source,
        &[
            "pub struct GeometrySourceDescriptor",
            "pub enum GeometrySourceVertexAttribute",
            "pub enum GeometrySourceBindingKind",
            "pub struct GeometrySourceBindingRequirement",
            "pub fn builtin_geometry_source_descriptors()",
            "pub fn builtin_geometry_source_descriptor(",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH",
            "GeometrySourceBindingKind::GpuSceneInstance",
            "GeometrySourceBindingKind::SkinningPaletteStorage",
            "GeometrySourceBindingKind::MorphWeightsStorage",
            "GeometrySourceBindingKind::MorphTargetStorage",
            "RenderShaderDefinitionValue::uint(\"ZR_GEOMETRY_SOURCE_ID\"",
            "RenderShaderDefinitionValue::bool(primary_define, true)",
            "render_shader_geometry_source_descriptors_cover_builtin_segment",
            "render_shader_geometry_source_descriptors_report_shape_requirements",
        ],
    );

    assert_contains_all(
        "shader module re-exports GeometrySource descriptor contract",
        &shader_mod,
        &[
            "builtin_geometry_source_descriptor",
            "builtin_geometry_source_descriptors",
            "GeometrySourceBindingKind",
            "GeometrySourceBindingRequirement",
            "GeometrySourceDescriptor",
            "GeometrySourceVertexAttribute",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH",
        ],
    );

    let line_count = geometry_source.lines().count();
    assert!(
        line_count < 800,
        "GeometrySource descriptor contract owner should stay below R4.3 budget; got {line_count}"
    );

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "GeometrySource descriptor contract foundation",
                "render_plan08_geometry_source_descriptor_contract_static_passed_cargo_deferred_implementation_cadence",
                "core/framework/render/shader/geometry_source.rs",
                "GeometrySourceDescriptor",
                "runtime_15_render_shader_geometry_source_descriptor_contract_is_complete",
            ],
        );
    }
}
