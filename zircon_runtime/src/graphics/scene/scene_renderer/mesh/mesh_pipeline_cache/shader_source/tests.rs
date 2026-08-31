#[path = "tests/runtime_shading_model_sources.rs"]
mod runtime_shading_model_sources;

use crate::core::framework::render::{
    GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_PLUGIN_ID_START, GeometrySourceBindingKind,
    GeometrySourceBindingRequirement, GeometrySourceDescriptor, GeometrySourceId,
    GeometrySourceVertexAttribute, RenderShaderDefinitionValue, ShaderFeatureBits, ShaderPassType,
};
use crate::graphics::scene::resources::default_pipeline_key;

use super::{
    MESH_SHADER_TEMPLATE_REVISION, MeshPipelineShaderSource,
    mesh_pipeline_deferred_gbuffer_template_source_for_geometry,
    mesh_pipeline_depth_prepass_template_source_for_geometry,
    mesh_pipeline_shadow_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source,
    mesh_pipeline_standard_material_template_source_for_geometry,
    mesh_pipeline_standard_material_template_source_for_geometry_descriptor,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_taa_reactive_mask_template_source_for_geometry,
    mesh_pipeline_velocity_template_source_for_geometry,
};

#[test]
fn mesh_shader_source_build_profiles_assembly_hash_and_validation_separately() {
    let source = include_str!("../shader_source.rs");

    assert!(source.contains("\"shader_pipeline\", \"mesh_source_build\""));
    assert!(source.contains("\"shader_pipeline\", \"source_hash\""));
    assert!(source.contains("\"shader_pipeline\", \"naga_validation\""));
    assert!(source.contains("mesh_shader_source_bytes"));
    assert!(source.contains("mesh_shader_assembly_segment_count"));
}

#[test]
fn mesh_pipeline_standard_material_template_source_assembles_forward_base_source() {
    let mut key = default_pipeline_key();
    key.alpha_mask = true;
    key.alpha_cutoff_bits = Some(0.5f32.to_bits());
    key.double_sided = true;
    key.receive_shadows = false;

    let source = match mesh_pipeline_standard_material_template_source(&key) {
        Ok(source) => source,
        Err(error) => panic!("standard material template assembly failed: {error:?}"),
    };

    assert!(
        source
            .wgsl_source
            .contains("// include: zr_scene_runtime.wgsl")
    );
    assert!(source.wgsl_source.contains("// include: zr_gpu_scene.wgsl"));
    assert!(
        source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl")
    );
    assert!(source.wgsl_source.contains("// include: zr_shadow.wgsl"));
    assert!(source.wgsl_source.contains("fn vs_main("));
    assert!(source.wgsl_source.contains("fn fs_main("));
    assert!(!source.wgsl_source.contains("fn fs_oit("));
    assert!(
        !source
            .wgsl_source
            .contains("var<storage, read_write> oit_layers")
    );
    assert!(source.wgsl_source.contains("fn zr_material_surface("));
    assert!(
        source
            .wgsl_source
            .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;")
    );
    assert!(
        source
            .wgsl_source
            .contains("const ZR_FEATURE_ALPHA_TEST: bool = true;")
    );
    assert!(
        source
            .wgsl_source
            .contains("const ZR_FEATURE_RECEIVE_SHADOWS: bool = false;")
    );
    assert!(
        source
            .wgsl_source
            .contains("const ZR_FEATURE_DOUBLE_SIDED: bool = true;")
    );
    assert!(
        source
            .wgsl_source
            .contains("const ZR_FEATURE_HAS_NORMAL_TEXTURE: bool = false;")
    );
    assert!(
        source
            .wgsl_source
            .contains("standard_material_shading_model_id")
    );
    assert!(
        source
            .wgsl_source
            .contains("surface.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID")
    );
    assert!(
        key.shader_feature_bits()
            .contains(ShaderFeatureBits::ALPHA_TEST)
    );
    assert!(!source.segments.is_empty());
    assert_ne!(source.validation_cache_key(), source.source_hash);
}

#[test]
fn mesh_pipeline_shader_source_remaps_invalid_wgsl_for_background_diagnostics() {
    let source = MeshPipelineShaderSource::from_raw_wgsl("fn invalid_wgsl(");

    let error = source
        .validate_wgsl(&source.wgsl_source)
        .expect_err("invalid WGSL must retain its remapped diagnostic for the validation worker");

    assert!(!error.is_empty());
    assert_eq!(source.validation_cache_key(), source.source_hash);
}

#[test]
fn mesh_pipeline_oit_source_is_a_dedicated_fragment_store_variant() {
    let key = default_pipeline_key();
    let base = mesh_pipeline_standard_material_template_source(&key)
        .expect("standard material template assembly");
    let base_hash = base.source_hash.clone();
    let base_segment_count = base.segments.len();
    let oit = base
        .into_oit_fragment_store_source()
        .expect("template forward source should support OIT specialization");

    assert_ne!(oit.source_hash, base_hash);
    assert!(oit.template_revision.ends_with("+oit-fragment-store-v1"));
    assert!(oit.wgsl_source.contains("fn fs_oit("));
    assert!(
        oit.wgsl_source
            .contains("oit_draw(input.clip_position, zr_fs_main_impl(input, front_facing));")
    );
    assert!(
        oit.wgsl_source
            .contains("@builtin(front_facing) front_facing: bool")
    );
    assert!(
        oit.wgsl_source
            .contains("@group(4) @binding(0) var<storage, read_write> oit_layers")
    );
    assert_eq!(oit.segments.len(), base_segment_count + 2);
    assert!(
        oit.segments
            .iter()
            .any(|segment| segment.module_id == "zircon::oit::draw")
    );
    assert!(
        oit.segments
            .iter()
            .any(|segment| segment.module_id == "zircon::oit::fragment_store_entry")
    );
}

#[test]
fn mesh_pipeline_standard_material_template_source_derives_normal_texture_define() {
    let mut key = default_pipeline_key();
    key.has_normal_texture = true;

    let source = match mesh_pipeline_standard_material_template_source(&key) {
        Ok(source) => source,
        Err(error) => panic!("standard material template assembly failed: {error:?}"),
    };

    assert!(
        source
            .wgsl_source
            .contains("const ZR_FEATURE_HAS_NORMAL_TEXTURE: bool = true;")
    );
    assert!(
        key.shader_feature_bits()
            .contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE)
    );
}

#[test]
fn mesh_pipeline_depth_prepass_template_source_uses_depth_only_template() {
    let mut key = default_pipeline_key();
    key.alpha_mask = true;
    key.alpha_cutoff_bits = Some(0.5f32.to_bits());

    let source = match mesh_pipeline_depth_prepass_template_source_for_geometry(
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
    ) {
        Ok(source) => source,
        Err(error) => panic!("depth prepass template source assembly failed: {error:?}"),
    };

    assert!(
        source
            .wgsl_source
            .contains("// include: zr_template_depth_alpha.wgsl")
    );
    assert!(source.wgsl_source.contains("zr_geometry_skinned.wgsl"));
    assert!(source.wgsl_source.contains("fn vs_main("));
    assert!(source.wgsl_source.contains("fn fs_main("));
    assert!(source.wgsl_source.contains("fn zr_material_surface("));
    assert!(source.wgsl_source.contains("zr_surface_fails_alpha_clip"));
    assert!(!source.wgsl_source.contains("surface.normal_ws * 0.5"));
    assert!(
        !source
            .wgsl_source
            .contains("// include: zr_template_gbuffer.wgsl")
    );
    assert!(
        !source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl")
    );
    assert!(!source.wgsl_source.contains("// include: zr_shadow.wgsl"));
}

#[test]
fn mesh_pipeline_standard_material_shader_pass_source_keeps_depth_only_contract() {
    let opaque_key = default_pipeline_key();
    let opaque_source = match mesh_pipeline_standard_material_template_source_for_shader_pass(
        &opaque_key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ShaderPassType::DepthPrepass,
    ) {
        Ok(source) => source,
        Err(error) => panic!("opaque depth-only template source failed: {error:?}"),
    };

    assert!(
        opaque_source
            .wgsl_source
            .contains("// include: zr_template_depth.wgsl")
    );
    assert!(
        opaque_source
            .wgsl_source
            .contains("// include: zr_geometry_skinned.wgsl")
    );
    assert!(opaque_source.wgsl_source.contains("fn vs_main("));
    assert!(!opaque_source.wgsl_source.contains("fn fs_main("));
    assert!(!opaque_source.wgsl_source.contains("zr_material_surface"));
    assert!(
        !opaque_source
            .wgsl_source
            .contains("surface.normal_ws * 0.5")
    );
    assert!(
        !opaque_source
            .wgsl_source
            .contains("// include: zr_template_gbuffer.wgsl")
    );
    assert!(
        !opaque_source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl")
    );
    assert!(
        !opaque_source
            .wgsl_source
            .contains("// include: zr_shadow.wgsl")
    );
    assert!(
        opaque_source
            .cache_content_hashes
            .contains(&opaque_source.source_hash)
    );

    let mut alpha_key = default_pipeline_key();
    alpha_key.alpha_mask = true;
    alpha_key.alpha_cutoff_bits = Some(0.5f32.to_bits());
    let alpha_source = match mesh_pipeline_standard_material_template_source_for_shader_pass(
        &alpha_key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ShaderPassType::DepthPrepass,
    ) {
        Ok(source) => source,
        Err(error) => panic!("alpha depth-only template source failed: {error:?}"),
    };

    assert!(
        alpha_source
            .wgsl_source
            .contains("// include: zr_template_depth_alpha.wgsl")
    );
    assert!(alpha_source.wgsl_source.contains("fn fs_main("));
    assert!(alpha_source.wgsl_source.contains("zr_material_surface"));
    assert!(
        alpha_source
            .wgsl_source
            .contains("zr_apply_alpha_clip(surface);")
    );
    assert!(!alpha_source.wgsl_source.contains("surface.normal_ws * 0.5"));
    assert!(
        !alpha_source
            .wgsl_source
            .contains("// include: zr_template_gbuffer.wgsl")
    );
    assert_ne!(opaque_source.wgsl_source, alpha_source.wgsl_source);
    assert_ne!(
        opaque_source.cache_content_hashes,
        alpha_source.cache_content_hashes
    );
    assert_eq!(alpha_source.template_revision, "zr-material-template-v1");
}

#[test]
fn mesh_pipeline_deferred_gbuffer_template_source_writes_albedo_and_material_targets() {
    let mut key = default_pipeline_key();
    key.alpha_mask = true;
    key.alpha_cutoff_bits = Some(0.5f32.to_bits());

    let source = match mesh_pipeline_deferred_gbuffer_template_source_for_geometry(
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
    ) {
        Ok(source) => source,
        Err(error) => panic!("deferred gbuffer template source assembly failed: {error:?}"),
    };

    assert!(
        source
            .wgsl_source
            .contains("// include: zr_template_deferred_gbuffer.wgsl")
    );
    assert!(
        source
            .wgsl_source
            .contains("// include: zr_gbuffer_encode_standard_pbr.wgsl")
    );
    assert!(
        source
            .wgsl_source
            .contains("// include: zr_geometry_skinned.wgsl")
    );
    assert!(source.wgsl_source.contains("fn vs_main("));
    assert!(source.wgsl_source.contains("fn fs_main("));
    assert!(source.wgsl_source.contains("ZrDeferredGBufferOutput"));
    assert!(source.wgsl_source.contains("@location(0) albedo"));
    assert!(source.wgsl_source.contains("@location(1) normal"));
    assert!(source.wgsl_source.contains("@location(2) material"));
    assert!(source.wgsl_source.contains("@location(3) emissive"));
    assert!(
        source
            .wgsl_source
            .contains("zr_gpu_scene_has_lightmap(input.instance_index)")
    );
    assert!(
        source
            .wgsl_source
            .contains("vec4<f32>(max(surface.emissive, vec3<f32>(0.0)), 1.0)")
    );
    assert!(source.wgsl_source.contains("surface.normal_ws * 0.5"));
    assert!(
        source
            .wgsl_source
            .contains("zr_deferred_encode_material_flags(surface.shading_model_id")
    );
    assert!(source.wgsl_source.contains("zr_surface_fails_alpha_clip"));
    assert!(
        !source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl")
    );
    assert!(!source.wgsl_source.contains("// include: zr_shadow.wgsl"));
    assert_eq!(source.template_revision, "zr-material-template-v1");
}

#[test]
fn mesh_pipeline_template_source_hashes_include_template_revision() {
    let key = default_pipeline_key();

    let source = match mesh_pipeline_standard_material_template_source(&key) {
        Ok(source) => source,
        Err(error) => panic!("standard material template assembly failed: {error:?}"),
    };

    assert!(source.cache_content_hashes.len() > 1);
    assert!(source.cache_content_hashes.contains(&source.source_hash));
    assert_ne!(source.template_revision, MESH_SHADER_TEMPLATE_REVISION);
}

#[test]
fn mesh_pipeline_standard_material_template_source_uses_requested_geometry_source() {
    let key = default_pipeline_key();

    let source = match mesh_pipeline_standard_material_template_source_for_geometry(
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
    ) {
        Ok(source) => source,
        Err(error) => panic!("skinned standard material template assembly failed: {error:?}"),
    };

    assert!(
        source
            .wgsl_source
            .contains("// include: zr_geometry_skinned.wgsl")
    );
    assert!(
        source
            .wgsl_source
            .contains("const ZR_GEOMETRY_SOURCE_SKINNED_MESH: bool = true;")
    );
    assert!(
        source
            .wgsl_source
            .contains("zr_skinned_joint_matrix(v.joints.x)")
    );
    assert_eq!(source.template_revision, "zr-material-template-v1");
}

#[test]
fn mesh_pipeline_velocity_template_source_uses_previous_position_vertex_input() {
    let mut key = default_pipeline_key();
    key.alpha_mask = true;
    key.alpha_cutoff_bits = Some(0.5f32.to_bits());

    let source = match mesh_pipeline_velocity_template_source_for_geometry(
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
    ) {
        Ok(source) => source,
        Err(error) => panic!("skinned velocity template assembly failed: {error:?}"),
    };

    assert!(
        source
            .wgsl_source
            .contains("// include: zr_template_velocity_alpha.wgsl")
    );
    assert!(source.wgsl_source.contains("struct ZrVelocityVertexInput"));
    assert!(
        source
            .wgsl_source
            .contains("@location(8) previous_position")
    );
    assert!(
        source
            .wgsl_source
            .contains("let previous_input = zr_velocity_vertex_input(v, v.previous_position);")
    );
    assert!(
        source
            .wgsl_source
            .contains("fetch_prev_position(previous_input, instance_index)")
    );
    assert!(
        source
            .wgsl_source
            .contains("scene.previous_view_proj_unjittered * previous_world")
    );
    assert!(source.wgsl_source.contains("fn fs_main("));
    assert!(
        source
            .wgsl_source
            .contains("zr_velocity_apply_alpha_clip(input);")
    );
    assert!(source.wgsl_source.contains("zr_material_surface"));
    assert_eq!(source.template_revision, "zr-material-template-v1");
}

#[test]
fn mesh_pipeline_taa_reactive_mask_template_source_uses_material_surface_without_lighting() {
    let mut key = default_pipeline_key();
    key.alpha_mask = true;
    key.alpha_cutoff_bits = Some(0.5f32.to_bits());

    let source = match mesh_pipeline_taa_reactive_mask_template_source_for_geometry(
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
    ) {
        Ok(source) => source,
        Err(error) => panic!("skinned TAA reactive mask template assembly failed: {error:?}"),
    };

    assert!(
        source
            .wgsl_source
            .contains("// include: zr_template_taa_reactive_mask.wgsl")
    );
    assert!(
        source
            .wgsl_source
            .contains("// include: zr_geometry_skinned.wgsl")
    );
    assert!(source.wgsl_source.contains("fn vs_main("));
    assert!(source.wgsl_source.contains("fn fs_taa_reactive_mask("));
    assert!(
        source
            .wgsl_source
            .contains("fn fs_taa_reactive_material_mask(")
    );
    assert!(source.wgsl_source.contains("fn zr_material_surface("));
    assert!(source.wgsl_source.contains("surface.custom0.x"));
    assert!(
        source
            .wgsl_source
            .contains("const ZR_FEATURE_ALPHA_TEST: bool = true;")
    );
    assert!(
        !source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl")
    );
    assert!(!source.wgsl_source.contains("// include: zr_shadow.wgsl"));
    assert_eq!(source.template_revision, "zr-material-template-v1");
}

#[test]
fn mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked() {
    let mut key = default_pipeline_key();
    let opaque = match mesh_pipeline_shadow_template_source_for_geometry(
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
    ) {
        Ok(source) => source,
        Err(error) => panic!("skinned shadow template assembly failed: {error:?}"),
    };

    assert!(
        opaque
            .wgsl_source
            .contains("// include: zr_template_shadow.wgsl")
    );
    assert!(opaque.wgsl_source.contains("fn vs_main("));
    assert!(!opaque.wgsl_source.contains("fn fs_main("));
    assert!(!opaque.wgsl_source.contains("fn zr_material_surface("));
    assert!(
        !opaque
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl")
    );
    assert!(!opaque.wgsl_source.contains("// include: zr_shadow.wgsl"));

    key.alpha_mask = true;
    key.alpha_cutoff_bits = Some(0.5f32.to_bits());
    let alpha = match mesh_pipeline_shadow_template_source_for_geometry(
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
    ) {
        Ok(source) => source,
        Err(error) => panic!("skinned alpha shadow template assembly failed: {error:?}"),
    };

    assert!(
        alpha
            .wgsl_source
            .contains("// include: zr_template_shadow_alpha.wgsl")
    );
    assert!(alpha.wgsl_source.contains("fn vs_main("));
    assert!(alpha.wgsl_source.contains("fn fs_main("));
    assert!(alpha.wgsl_source.contains("fn zr_material_surface("));
    assert!(alpha.wgsl_source.contains("zr_surface_fails_alpha_clip"));
    assert!(
        alpha
            .wgsl_source
            .contains("const ZR_FEATURE_ALPHA_TEST: bool = true;")
    );
    assert!(!alpha.wgsl_source.contains("// include: zr_light_grid.wgsl"));
    assert!(!alpha.wgsl_source.contains("// include: zr_shadow.wgsl"));
    assert_eq!(alpha.template_revision, "zr-material-template-v1");
}

#[test]
fn mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings() {
    let key = default_pipeline_key();
    let descriptor = virtual_geometry_source_descriptor();

    let source = match mesh_pipeline_standard_material_template_source_for_geometry_descriptor(
        &key,
        &descriptor,
    ) {
        Ok(source) => source,
        Err(error) => panic!("virtual geometry template source assembly failed: {error:?}"),
    };

    assert!(
        source
            .wgsl_source
            .contains("// include: zr_geometry_virtual_geometry.wgsl")
    );
    assert!(
        source
            .wgsl_source
            .contains("@group(3) @binding(9) var<storage, read> zr_virtual_geometry_pages")
    );
    assert!(
        source
            .wgsl_source
            .contains("@group(3) @binding(10) var<storage, read> zr_virtual_geometry_clusters")
    );
    assert!(
        source
            .wgsl_source
            .contains("fn zr_gpu_scene_virtual_geometry_page_count()")
    );
    assert!(
        source
            .wgsl_source
            .contains("fn zr_gpu_scene_virtual_geometry_cluster_word_count()")
    );
    assert!(source.wgsl_source.contains(
        "zr_gpu_scene_valid_payload_slot(payload_slot, zr_gpu_scene_virtual_geometry_page_count())"
    ));
    assert!(
        source.wgsl_source.contains(
            "let cluster_word_count = zr_gpu_scene_virtual_geometry_cluster_word_count()"
        )
    );
    assert!(!source.wgsl_source.contains(
        "zr_gpu_scene_valid_payload_slot(payload_slot, arrayLength(&zr_virtual_geometry_pages))"
    ));
    assert!(
        source
            .wgsl_source
            .contains("fn zr_virtual_geometry_vertex_word_index(")
    );
    assert!(
        source
            .wgsl_source
            .contains("v.joints.x | (v.joints.y << ZR_VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT)")
    );
    assert!(
        source
            .wgsl_source
            .contains("zr_gpu_scene_primitive_for_instance(instance_index).payload_slot")
    );
    assert!(descriptor.requires_binding(GeometrySourceBindingKind::VirtualGeometryPages));
    assert!(descriptor.requires_binding(GeometrySourceBindingKind::VirtualGeometryClusters));
}

fn virtual_geometry_source_descriptor() -> GeometrySourceDescriptor {
    GeometrySourceDescriptor {
        id: GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START),
        token: "custom:virtual_geometry".to_string(),
        wgsl_include: "zr_geometry_virtual_geometry.wgsl".to_string(),
        vertex_attributes: vec![
            GeometrySourceVertexAttribute::Position,
            GeometrySourceVertexAttribute::Normal,
            GeometrySourceVertexAttribute::Tangent,
            GeometrySourceVertexAttribute::Uv0,
        ],
        required_bindings: vec![
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryPages,
                "virtual_geometry.pages",
            ),
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryClusters,
                "virtual_geometry.clusters",
            ),
        ],
        shader_defines: vec![RenderShaderDefinitionValue::bool(
            "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY",
            true,
        )],
    }
}
