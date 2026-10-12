use super::super::{assert_contains_all, sources::RenderShaderTemplateAssemblySources};

pub(super) fn assert_mesh_cache_contracts(sources: &RenderShaderTemplateAssemblySources) {
    let RenderShaderTemplateAssemblySources {
        mesh_cache_mod,
        mesh_cache_state,
        mesh_cache_ensure,
        mesh_cache_ensure_tests,
        mesh_cache_velocity,
        mesh_cache_taa,
        mesh_cache_shadow,
        mesh_cache_source,
        mesh_cache_source_tests,
        ..
    } = sources;

    assert_contains_all(
        "mesh pipeline cache mounts shader source owner",
        &mesh_cache_mod,
        &[
            "mod shader_source;",
            "pub(crate) use shader_source::{",
            "mesh_pipeline_standard_material_template_source",
            "MeshPipelineShaderSource",
            "mod ensure_shadow_pipeline;",
            "mesh_pipeline_shadow_template_source_for_geometry",
        ],
    );
    assert_contains_all(
        "mesh pipeline shader source owner consumes standard material template source",
        &mesh_cache_source,
        &[
            "builtin_geometry_source_descriptor",
            "GeometrySourceId",
            "GEOMETRY_SOURCE_ID_STATIC_MESH",
            "assemble_material_shader_template",
            "standard_material_surface_source_for_features",
            "pub(crate) struct MeshPipelineShaderSource",
            "MeshPipelineShaderSource",
            "pub(crate) fn mesh_pipeline_shader_source",
            "source_hash",
            "cache_content_hashes",
            "mesh_pipeline_standard_material_template_source",
            "mesh_pipeline_standard_material_template_source_for_geometry",
            "mesh_pipeline_velocity_template_source_for_geometry",
            "mesh_pipeline_shadow_template_source_for_geometry",
            "mesh_pipeline_taa_reactive_mask_template_source_for_geometry",
            "ShaderPassType::Velocity",
            "ShaderPassType::Shadow",
            "assemble_taa_reactive_mask_shader_template",
            "TaaReactiveMaskShaderTemplateRequest",
            "mesh_pipeline_alpha_cutoff",
        ],
    );
    assert_contains_all(
        "mesh pipeline shader source tests keep template source coverage",
        &mesh_cache_source_tests,
        &[
            "mesh_pipeline_standard_material_template_source_assembles_forward_base_source",
            "mesh_pipeline_standard_material_template_source_uses_requested_geometry_source",
            "GEOMETRY_SOURCE_ID_SKINNED_MESH",
            "mesh_pipeline_velocity_template_source_uses_previous_position_vertex_input",
            "mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked",
            "mesh_pipeline_taa_reactive_mask_template_source_uses_material_surface_without_lighting",
            "zr_template_shadow_alpha.wgsl",
            "mesh_pipeline_template_source_hashes_include_template_revision",
        ],
    );
    assert_contains_all(
        "base mesh pipeline cache consumes shader source owner",
        &mesh_cache_ensure,
        &[
            "mesh_pipeline_shader_source",
            "geometry_source_descriptor_for_variant(&shader_variant_key)",
            "&geometry_source",
            "MeshPipelineShaderSource",
            "mesh_pipeline_shader_source_with_cache",
            "pub(in crate::graphics::scene::scene_renderer::mesh) fn mesh_pipeline_shader_source_with_cache",
            "mesh_shader_module_cache_key",
            "&shader_source.source_hash",
            "source.cache_content_hashes.iter().map(String::as_str)",
            "#[path = \"ensure_pipeline/tests.rs\"]",
        ],
    );
    assert_contains_all(
        "base mesh pipeline cache tests stay child-owned",
        &mesh_cache_ensure_tests,
        &[
            "mesh_pipeline_template_source_hashes_feed_disk_and_module_keys",
            "runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss",
            "runtime_project_plugin_registry_shader_keys_use_staged_prewarm_without_compile_miss",
            "runtime_custom_geometry_descriptor_pipeline_uses_staged_prewarm_without_compile_miss",
            "ensure_custom_geometry_pass_pipeline",
        ],
    );
    assert_contains_all(
        "mesh pipeline cache stores non-base pipelines by variant id",
        &mesh_cache_state,
        &[
            "velocity_mesh_pipelines:\n        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>",
            "shadow_mesh_pipelines:\n        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>",
            "taa_reactive_mask_mesh_pipelines:\n        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>",
            "taa_reactive_material_mask_mesh_pipelines:\n        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>",
            "pub(crate) fn pipeline_and_shader_key_for_variant",
            "Option<(MeshPassPipelineKind, PipelineKey, ShaderVariantKey)>",
        ],
    );
    assert!(
        !mesh_cache_state.contains("HashMap<PipelineKey, wgpu::RenderPipeline>"),
        "mesh pipeline cache should not keep pass-specific WGPU pipeline maps keyed only by PipelineKey"
    );
    assert!(
        !mesh_cache_state.contains("pub(crate) fn pipeline_key_for_variant"),
        "non-base pass pipeline lookup should use pipeline_and_shader_key_for_variant so ShaderVariantKey remains live"
    );
    assert_contains_all(
        "velocity pipeline cache consumes variant id and shader variant identity",
        &mesh_cache_velocity,
        &[
            "pipeline_and_shader_key_for_variant",
            "self.ensure_velocity_pipeline(",
            "variant_id",
            "&pipeline_key",
            "&shader_variant_key",
            "mesh_pipeline_velocity_template_source_for_geometry",
            "mesh_pipeline_shader_source_with_cache",
            "geometry_source_descriptor_for_variant(shader_variant_key)",
            "&geometry_source",
            "velocity_mesh_shader_key",
            "variant_key.canonical_string()",
            "&shader_source.source_hash",
            "velocity_mesh_pipelines.contains_key(&variant_id)",
            "velocity_mesh_pipelines.insert(variant_id, pipeline)",
            "velocity_mesh_pipelines.get(&variant_id)",
            "velocity_mesh_shader_key_includes_shader_variant_identity_and_source_hash",
        ],
    );
    assert!(
        !mesh_cache_velocity.contains("FALLBACK_MESH_SHADER"),
        "velocity pass should consume template source owner instead of direct fallback WGSL"
    );
    assert_contains_all(
        "taa reactive pipeline cache consumes variant id and shader variant identity",
        &mesh_cache_taa,
        &[
            "pipeline_and_shader_key_for_variant",
            "mesh_pipeline_taa_reactive_mask_template_source_for_geometry",
            "mesh_pipeline_shader_source_with_cache",
            "geometry_source_descriptor_for_variant(shader_variant_key)",
            "&geometry_source",
            "taa_reactive_mask_mesh_shader_key",
            "variant_key.canonical_string()",
            "&shader_source.source_hash",
            "taa_reactive_mask_mesh_pipelines",
            ".contains_key(&variant_id)",
            ".get(&variant_id)",
            "taa_reactive_material_mask_mesh_pipelines",
            "taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash",
        ],
    );
    assert!(
        !mesh_cache_taa.contains("FALLBACK_MESH_SHADER"),
        "TAA reactive mask pass should consume template source owner instead of direct fallback WGSL"
    );
    assert_contains_all(
        "shadow pipeline cache consumes variant id and shader variant identity",
        &mesh_cache_shadow,
        &[
            "pipeline_and_shader_key_for_variant",
            "mesh_pipeline_shadow_template_source_for_geometry",
            "mesh_pipeline_shader_source_with_cache",
            "geometry_source_descriptor_for_variant(shader_variant_key)",
            "&geometry_source",
            "shadow_mesh_shader_key",
            "variant_key.canonical_string()",
            "&shader_source.source_hash",
            "shadow_mesh_pipelines",
            "shadow_mesh_pipelines.contains_key(&variant_id)",
            "shadow_mesh_pipelines.insert(variant_id, pipeline)",
            "shadow_mesh_pipelines.get(&variant_id)",
            "ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask",
            "shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash",
        ],
    );
    let legacy_shadow_shader_symbol = ["SHADOW", "_MAP", "_SHADER"].concat();
    assert!(
        !mesh_cache_shadow.contains(legacy_shadow_shader_symbol.as_str()),
        "shadow pass should consume template source owner instead of the deleted shadow_map shader body"
    );
    assert!(
        !mesh_cache_ensure.contains("FALLBACK_MESH_SHADER"),
        "base mesh ensure_pipeline.rs should no longer consume the legacy monolithic fallback shader source directly"
    );
    for forbidden in [
        "builtin_geometry_source_descriptor",
        "GEOMETRY_SOURCE_ID_STATIC_MESH",
        "assemble_material_shader_template",
        "standard_material_surface_source_for_features",
    ] {
        assert!(
            !mesh_cache_ensure.contains(forbidden),
            "base mesh ensure_pipeline.rs should delegate template source assembly to shader_source.rs, but still contains {forbidden}"
        );
    }
}
