use super::super::{assert_contains_all, sources::RenderShaderTemplateAssemblySources};

pub(super) fn assert_shader_template_contracts(sources: &RenderShaderTemplateAssemblySources) {
    let RenderShaderTemplateAssemblySources {
        shader_mod,
        template_mod,
        assemble,
        module_registry,
        material_surface,
        pass_specialization,
        taa_reactive_template,
        validation,
        tests,
        template_surface_module_tests,
        variant_cache_prewarm,
        pipeline_key,
        ..
    } = sources;

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
            "mod module_registry;",
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
            "ShadingModelDescriptor",
            "with_shading_model_descriptor",
            "shading_model_forward_include_sources",
            "with_shading_model_forward_include_source",
            "UnknownShadingInclude",
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
        &module_registry,
        &[
            "pub(crate) struct ShaderTemplateInclude",
            "pub(crate) struct ShaderTemplateIncludeRegistry",
            "blake3::hash",
            "HashSet<String>",
            "source_includes",
            "geometry_source_include_for",
            "scene_runtime_include",
            "gpu_scene_include",
            "light_grid_include",
            "shadow_include",
            "shading_model_forward_include_for",
            "shading_model_forward_include_token",
            "shading_model_gbuffer_include_for",
            "shading_model_gbuffer_include_token",
            "zr_scene_runtime.wgsl",
            "zr_gpu_scene.wgsl",
            "zr_light_grid.wgsl",
            "zr_shadow.wgsl",
            "zr_shading_standard_pbr.wgsl",
            "zr_gbuffer_encode_standard_pbr.wgsl",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH",
            "GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH",
            "include_str!(\"../wgsl/zr_geometry_static.wgsl\")",
            "include_str!(\"../wgsl/zr_gbuffer_encode_standard_pbr.wgsl\")",
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
            "mod surface_modules;",
            "render_shader_template_assembles_standard_material_surface_source",
            "standard_material_surface_source_can_be_built_from_runtime_features",
            "render_shader_template_validates_standard_material_wgsl_with_naga",
            "render_shader_template_clips_alpha_for_masked_standard_material_passes",
            "render_shader_template_specializes_depth_and_velocity_passes",
            "render_shader_template_uses_shading_model_descriptor_forward_include",
            "render_shader_template_rejects_unknown_shading_model_forward_include",
            "render_shader_template_uses_custom_shading_model_forward_include_source",
            "render_deferred_gbuffer_template_rejects_unknown_shading_model_gbuffer_include",
            "render_deferred_gbuffer_template_uses_custom_shading_model_gbuffer_include_source",
            "render_shader_template_rejects_reserved_material_symbols",
            "CUSTOM_TOON_FORWARD_INCLUDE",
            "CUSTOM_TOON_GBUFFER_INCLUDE",
            "with_shading_model_forward_include_source",
            "with_shading_model_gbuffer_include_source",
            "ZR_SHADING_TOON_DEBUG_ID",
            "ZR_GBUFFER_TOON_DEBUG_ID",
            "fn zr_toon_band",
            "custom shading include template WGSL should validate",
            "custom deferred GBuffer include template WGSL should validate",
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
            "zr_shading_toon.wgsl",
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
            "fn zr_build_vertex_output(",
            "@location(3) uv1: vec2<f32>",
        ],
    );
    assert_contains_all(
        "template surface module tests stay child-owned",
        &template_surface_module_tests,
        &[
            "render_shader_template_expands_declared_surface_modules_and_strips_directives",
            "render_shader_self_material_anchor_is_byte_identical_to_auto_injection",
            "render_shader_template_reports_unknown_surface_module",
            "ShaderTemplateInclude::new",
            "RenderShaderDefinitionValue::bool",
            "GENERATED_MATERIAL_MODULE_IMPORT_PATH",
        ],
    );
}
