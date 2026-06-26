use super::{sources::RenderShaderTemplateAssemblySources, *};

pub(super) fn assert_render_shader_template_assembly_is_folder_backed(
    sources: &RenderShaderTemplateAssemblySources,
) {
    let RenderShaderTemplateAssemblySources {
        shader_mod,
        template_mod,
        assemble,
        include_registry,
        material_surface,
        pass_specialization,
        taa_reactive_template,
        validation,
        tests,
        variant_cache_prewarm,
        pipeline_key,
        mesh_cache_mod,
        mesh_cache_state,
        mesh_cache_ensure,
        mesh_cache_velocity,
        mesh_cache_taa,
        mesh_cache_shadow,
        mesh_cache_source,
        mesh_pipeline_mod,
        mesh_pipeline_test_support,
        mesh_pipeline_velocity,
        mesh_pipeline_taa,
        mesh_pipeline_shadow,
        shadow_processor,
        non_material_rebuild,
        shadow_renderer,
        shadow_mod,
        graph_gpu_context,
        graph_stage_execution,
    } = sources;
    let mesh_pipeline_shadow_production = mesh_pipeline_shadow
        .split("#[cfg(test)]")
        .next()
        .expect("shadow mesh pipeline production section");
    assert_contains_all(
        "graphics shader module mounts template owner",
        &shader_mod,
        &[
            "pub(crate) mod template;",
            "assemble_material_shader_template",
            "assemble_taa_reactive_mask_shader_template",
            "MaterialShaderTemplateRequest",
            "TaaReactiveMaskShaderTemplateRequest",
            "ShaderTemplateAssemblyError",
        ],
    );

    assert_contains_all(
        "template module stays folder-backed",
        &template_mod,
        &[
            "mod assemble;",
            "mod include_registry;",
            "mod material_surface;",
            "mod pass_specialization;",
            "mod taa_reactive_mask;",
            "mod validation;",
            "mod tests;",
            "pub(crate) use assemble::{",
            "standard_material_surface_source",
            "StandardMaterialSurfaceSource",
            "assemble_taa_reactive_mask_shader_template",
            "TaaReactiveMaskShaderTemplateRequest",
            "validate_material_shader_template_wgsl",
            "validate_shader_variant_prewarm_wgsl",
            "ShaderTemplateValidationError",
        ],
    );

    assert_contains_all(
        "template assembler owns request/output contract",
        &assemble,
        &[
            "pub(crate) struct MaterialShaderTemplateRequest",
            "pub(crate) struct MaterialShaderTemplateAssembly",
            "pub(crate) enum ShaderTemplateAssemblyError",
            "pub(crate) fn assemble_material_shader_template",
            "GeometrySourceDescriptor",
            "ShaderFeatureBits",
            "ShaderPassType",
            "rename_material_surface_entry",
            "RESERVED_MATERIAL_SYMBOL_PREFIXES",
            "MATERIAL_SHADER_TEMPLATE_REVISION",
            "ZR_FEATURE_ALPHA_TEST",
            "ZR_FEATURE_RECEIVE_SHADOWS",
            "ZR_FEATURE_DOUBLE_SIDED",
        ],
    );

    assert_contains_all(
        "taa reactive mask template assembler owns auxiliary template source assembly",
        &taa_reactive_template,
        &[
            "pub(crate) struct TaaReactiveMaskShaderTemplateRequest",
            "pub(crate) fn assemble_taa_reactive_mask_shader_template",
            "format_defines_header",
            "push_include_chunk",
            "rename_material_surface_entry",
            "scene_runtime_include",
            "gpu_scene_include",
            "surface_types_include",
            "geometry_source_include_for",
            "zr_template_taa_reactive_mask.wgsl",
            "MATERIAL_SHADER_TEMPLATE_REVISION",
        ],
    );
    assert!(
        !taa_reactive_template.contains("light_grid_include"),
        "TAA reactive mask auxiliary template should not pull Forward lighting includes"
    );
    assert!(
        !taa_reactive_template.contains("shadow_include"),
        "TAA reactive mask auxiliary template should not pull Forward shadow includes"
    );

    assert_contains_all(
        "template include registry owns include_str and hashing",
        &include_registry,
        &[
            "pub(crate) struct ShaderTemplateInclude",
            "pub(crate) struct ShaderTemplateIncludeRegistry",
            "blake3::hash",
            "geometry_source_include_for",
            "scene_runtime_include",
            "gpu_scene_include",
            "light_grid_include",
            "shadow_include",
            "zr_scene_runtime.wgsl",
            "zr_gpu_scene.wgsl",
            "zr_light_grid.wgsl",
            "zr_shadow.wgsl",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH",
            "include_str!(\"../wgsl/zr_geometry_static.wgsl\")",
            "include_str!(\"../wgsl/zr_scene_runtime.wgsl\")",
            "include_str!(\"../../scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl\")",
            "include_str!(\"../../scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl\")",
            "include_str!(\"../../scene/scene_renderer/shadow/shaders/zr_shadow.wgsl\")",
            "include_str!(\"../wgsl/zr_surface_types.wgsl\")",
        ],
    );

    assert_contains_all(
        "standard material surface source owns descriptor projection",
        &material_surface,
        &[
            "pub(crate) struct StandardMaterialSurfaceSource",
            "pub(crate) fn standard_material_surface_source",
            "pub(crate) fn standard_material_surface_source_for_features",
            "StandardMaterialDescriptor",
            "RenderMaterialAlphaMode::Mask",
            "ShaderFeatureBits::ALPHA_TEST",
            "ShaderFeatureBits::RECEIVE_SHADOWS",
            "ShaderFeatureBits::DOUBLE_SIDED",
            "standard_material_surface",
            "standard_material_sampled_normal",
            "standard_material_normal_tex",
            "standard_material_metallic_roughness_tex",
            "standard_material_alpha_cutoff",
            "standard_material_shading_model_id",
            "standard_material_properties.data8.z",
            "input.tint * input.color",
            "input.tangent_handedness",
            "input.uv1",
        ],
    );

    assert_contains_all(
        "pipeline key exposes runtime shader feature bits for template cutover",
        &pipeline_key,
        &[
            "pub(crate) fn shader_feature_bits",
            "ShaderFeatureBits::ALPHA_TEST",
            "ShaderFeatureBits::RECEIVE_SHADOWS",
            "ShaderFeatureBits::DOUBLE_SIDED",
            "pipeline_key_can_disable_receive_shadow_shader_feature",
        ],
    );

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
            "GEOMETRY_SOURCE_ID_SKINNED_MESH",
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
            "mesh_pipeline_standard_material_template_source_assembles_forward_base_source",
            "mesh_pipeline_standard_material_template_source_uses_requested_geometry_source",
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
            "shader_variant_key.geometry_source",
            "MeshPipelineShaderSource",
            "mesh_pipeline_shader_source_with_cache",
            "pub(in crate::graphics::scene::scene_renderer::mesh) fn mesh_pipeline_shader_source_with_cache",
            "mesh_shader_module_cache_key",
            "&shader_source.source_hash",
            "source.cache_content_hashes.iter().map(String::as_str)",
            "mesh_pipeline_template_source_hashes_feed_disk_and_module_keys",
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
            "ensure_velocity_pipeline(device, variant_id, &pipeline_key, &shader_variant_key)",
            "mesh_pipeline_velocity_template_source_for_geometry",
            "mesh_pipeline_shader_source_with_cache",
            "shader_variant_key.geometry_source",
            "velocity_mesh_shader_key",
            "variant_key.canonical_string()",
            "&shader_source.source_hash",
            "velocity_mesh_pipelines.contains_key(&variant_id)",
            "velocity_mesh_pipelines.insert(variant_id, pipeline)",
            "velocity_mesh_pipelines\n            .get(&variant_id)",
            "velocity_mesh_shader_key_includes_shader_variant_identity_and_source_hash",
        ],
    );
    assert!(
        !mesh_cache_velocity.contains("FALLBACK_MESH_SHADER"),
        "velocity pass should consume template source owner instead of direct fallback WGSL"
    );
    assert_contains_all(
        "velocity mesh pipeline consumes template entry names",
        &mesh_pipeline_velocity,
        &[
            "entry_point: Some(\"vs_main\")",
            "entry_point: Some(\"fs_main\")",
            "GpuMeshVertex::previous_position_layout()",
            "velocity_mesh_pipeline_declares_template_entries_and_previous_position_vertex_slot",
            "velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "create_standard_mesh_pipeline_layout",
        ],
    );
    assert_contains_all(
        "mesh pipeline WGPU test support owns shared standard layout fixture",
        &mesh_pipeline_test_support,
        &[
            "pub(crate) fn create_standard_mesh_pipeline_layout",
            "create_test_scene_layout",
            "create_empty_shadow_receiver_layout",
            "create_test_material_layout",
            "GPU_MATERIAL_UNIFORM_MIN_SIZE",
            "GpuScene::new",
        ],
    );
    assert_contains_all(
        "mesh pipeline root mounts shared WGPU test support only for tests",
        &mesh_pipeline_mod,
        &["#[cfg(test)]", "mod test_support;"],
    );
    assert_contains_all(
        "taa reactive pipeline cache consumes variant id and shader variant identity",
        &mesh_cache_taa,
        &[
            "pipeline_and_shader_key_for_variant",
            "mesh_pipeline_taa_reactive_mask_template_source_for_geometry",
            "mesh_pipeline_shader_source_with_cache",
            "shader_variant_key.geometry_source",
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
            "shader_variant_key.geometry_source",
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
    assert_contains_all(
        "shadow mesh pipeline consumes template entry names",
        &mesh_pipeline_shadow,
        &[
            "entry_point: Some(\"vs_main\")",
            "entry_point: Some(\"fs_main\")",
            "targets: &[]",
            "GpuMeshVertex::layout()",
            "SHADOW_DEPTH_BIAS_CONSTANT",
            "shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias",
            "shadow_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "GpuScene::new",
        ],
    );
    assert!(
        !mesh_pipeline_shadow_production.contains("GpuMeshVertex::previous_position_layout()"),
        "shadow mesh pipeline should keep the static mesh vertex ABI and not consume the Velocity-only previous-position slot"
    );
    assert_contains_all(
        "shadow replay resolves cache-backed variants at atlas execution time",
        &shadow_renderer,
        &[
            "ensure_shadow_pipeline_for_variant",
            "command.pipeline_variant_id",
            "bind_standard_material_if_needed",
            "record_depth_only_pass",
            "MeshPipelineCache",
            "record_atlas_commands_with_attachment_ops",
        ],
    );
    let forbidden_shadow_renderer_tokens = [
        ["SHADOW", "_MAP", "_SHADER"].concat(),
        ["fixed", "_shadow", "_variant"].concat(),
        ["alpha", "_mask", "_pipeline"].concat(),
        ["fs", "_alpha", "_mask"].concat(),
    ];
    for forbidden in &forbidden_shadow_renderer_tokens {
        assert!(
            !shadow_renderer.contains(forbidden.as_str()),
            "shadow renderer should not retain the legacy inline shadow shader path token {forbidden}"
        );
    }
    let legacy_shadow_source_module = ["shadow", "_map", "_shader", "_source"].concat();
    assert!(
        !shadow_mod.contains(legacy_shadow_source_module.as_str()),
        "shadow module should not mount the deleted inline shadow-map shader source owner"
    );
    assert_contains_all(
        "shadow command producers resolve real variant ids",
        &shadow_processor,
        &["context.pipeline_variant_id(pipeline_kind, batch)"],
    );
    assert!(
        !shadow_processor.contains("MeshPipelineVariantId::new(0)"),
        "shadow pass processor should not assign the fixed base variant id"
    );
    assert_contains_all(
        "pre-mesh shadow rebuild resolves real variant ids",
        &non_material_rebuild,
        &[
            "context.pipeline_variant_id(pipeline_kind, batch)",
            "rebuilds_opaque_shadow_command_without_material_handles",
        ],
    );
    assert_contains_all(
        "shadow graph execution carries mesh pipeline context",
        &graph_gpu_context,
        &[
            "record_shadow_atlas_to_resources",
            "shadow atlas graph executor for pass `{pass_name}` requires mesh pipeline context",
            "record_atlas_commands_with_attachment_ops",
            "self.device",
            "mesh_pipelines,",
        ],
    );
    assert_contains_all(
        "early shadow graph stage receives mesh pipeline context",
        &graph_stage_execution,
        &[
            "let uses_mesh_pipeline_context = is_depth_prepass || is_shadow;",
            "stage_streamer = uses_mesh_pipeline_context.then_some(streamer)",
            "stage_mesh_pipelines = if uses_mesh_pipeline_context",
            "uses_mesh_pipeline_context.then_some(mesh_draw_lists)",
            "is_shadow.then_some(&self.shadow_map_renderer)",
            "is_shadow.then_some(shadow_frame_plan)",
        ],
    );

    assert_contains_all(
        "taa reactive mask mesh pipeline consumes template entry names",
        &mesh_pipeline_taa,
        &[
            "entry_point: Some(\"vs_main\")",
            "\"fs_taa_reactive_mask\"",
            "\"fs_taa_reactive_material_mask\"",
            "GpuMeshVertex::layout()",
            "taa_reactive_mask_pipeline_declares_template_entries_and_static_vertex_layout",
            "taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "create_standard_mesh_pipeline_layout",
        ],
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

    assert_contains_all(
        "pass specialization owns pass template selection",
        &pass_specialization,
        &[
            "MATERIAL_SHADER_TEMPLATE_REVISION",
            "pub(crate) fn pass_template_for",
            "ShaderPassType::Forward",
            "ShaderPassType::GBuffer",
            "ShaderPassType::DepthPrepass",
            "ShaderPassType::Shadow",
            "ShaderPassType::Velocity",
            "ShaderFeatureBits::ALPHA_TEST",
            "zr_template_depth_alpha.wgsl",
            "zr_template_shadow_alpha.wgsl",
            "zr_template_velocity_alpha.wgsl",
            "VELOCITY_ALPHA_TEMPLATE",
            "support_includes",
            "requires_material_surface: alpha_test",
            "uses_previous_position: true",
        ],
    );

    assert_contains_all(
        "template validation owns naga parse and validation",
        &validation,
        &[
            "pub(crate) struct MaterialShaderTemplateValidation",
            "pub(crate) enum ShaderTemplateValidationError",
            "pub(crate) fn validate_material_shader_template_wgsl",
            "pub(crate) fn validate_shader_variant_prewarm_wgsl",
            "naga::front::wgsl::parse_str",
            "naga::valid::Validator::new",
            "naga::valid::ValidationFlags::all()",
            "entry_points",
        ],
    );

    assert_contains_all(
        "shader variant prewarm validates WGSL before disk writes",
        &variant_cache_prewarm,
        &[
            "validate_shader_variant_prewarm_wgsl",
            "shader variant WGSL validation failed",
            "continue;",
            "render_shader_variant_prewarm_rejects_invalid_wgsl_before_disk_write",
            "ShaderVariantCacheDiskLookup::Miss",
        ],
    );

    assert_contains_all(
        "template unit tests cover geometry and pass dimensions",
        &tests,
        &[
            "render_shader_template_assembles_static_and_skinned_geometry_sources",
            "render_shader_template_assembles_standard_material_surface_source",
            "standard_material_surface_source_can_be_built_from_runtime_features",
            "render_shader_template_validates_standard_material_wgsl_with_naga",
            "render_shader_template_clips_alpha_for_masked_standard_material_passes",
            "render_shader_template_specializes_depth_and_velocity_passes",
            "render_shader_template_rejects_reserved_material_symbols",
            "ZR_STANDARD_MATERIAL_ALPHA_CUTOFF",
            "standard_material_properties.data8.z",
            "zr_template_depth_alpha.wgsl",
            "zr_template_shadow_alpha.wgsl",
            "zr_template_velocity_alpha.wgsl",
            "zr_geometry_static.wgsl",
            "zr_geometry_skinned.wgsl",
            "zr_scene_runtime.wgsl",
            "zr_gpu_scene.wgsl",
            "zr_light_grid.wgsl",
            "zr_shadow.wgsl",
            "zr_world_from_local(instance_index)",
            "scene.view_proj * position_ws",
            "zr_build_shading_context(input)",
            "ZR_FEATURE_RECEIVE_SHADOWS && ctx.shadow_params.z > 0.5",
            "zr_skinned_joint_matrix(v.joints.x)",
            "fn zr_vs_main_impl(",
            "fn zr_fs_main_impl(",
            "fn vs_main(",
            "fn fs_main(",
            "struct ZrVelocityVertexInput",
            "@location(8) previous_position",
            "scene.previous_view_proj_unjittered * previous_world",
            "fetch_tangent(v, instance_index)",
            "zr_build_vertex_output(\n        instance_index,",
            "@location(3) uv1: vec2<f32>",
        ],
    );

    for (path, source) in [
        ("graphics/shader/mod.rs", shader_mod.as_str()),
        ("graphics/shader/template/mod.rs", template_mod.as_str()),
        ("graphics/shader/template/assemble.rs", assemble.as_str()),
        (
            "graphics/shader/template/include_registry.rs",
            include_registry.as_str(),
        ),
        (
            "graphics/shader/template/material_surface.rs",
            material_surface.as_str(),
        ),
        (
            "graphics/shader/template/pass_specialization.rs",
            pass_specialization.as_str(),
        ),
        (
            "graphics/shader/template/taa_reactive_mask.rs",
            taa_reactive_template.as_str(),
        ),
        (
            "graphics/shader/template/validation.rs",
            validation.as_str(),
        ),
        ("graphics/shader/template/tests.rs", tests.as_str()),
        (
            "graphics/shader/variant_cache/prewarm.rs",
            variant_cache_prewarm.as_str(),
        ),
        (
            "graphics/scene/resources/pipeline/pipeline_key.rs",
            pipeline_key.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs",
            mesh_cache_ensure.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs",
            mesh_cache_velocity.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs",
            mesh_cache_taa.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs",
            mesh_cache_shadow.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
            mesh_cache_state.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            mesh_cache_source.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/mod.rs",
            mesh_pipeline_mod.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs",
            mesh_pipeline_test_support.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs",
            mesh_pipeline_velocity.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs",
            mesh_pipeline_taa.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs",
            mesh_pipeline_shadow.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pass/processors/shadow.rs",
            shadow_processor.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs",
            non_material_rebuild.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs",
            shadow_renderer.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
            graph_gpu_context.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs",
            graph_stage_execution.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below R4.3 production/test owner budget; got {line_count}"
        );
    }
}
