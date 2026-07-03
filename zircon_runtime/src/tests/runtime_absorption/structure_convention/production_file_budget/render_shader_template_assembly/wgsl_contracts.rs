use super::*;

#[test]
fn runtime_15_render_shader_template_wgsl_contracts_are_child_owner() {
    let scene_runtime_wgsl = read_runtime_src("graphics/shader/wgsl/zr_scene_runtime.wgsl");
    let gpu_scene_wgsl =
        read_runtime_src("graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");
    let surface_types = read_runtime_src("graphics/shader/wgsl/zr_surface_types.wgsl");
    let static_wgsl = read_runtime_src("graphics/shader/wgsl/zr_geometry_static.wgsl");
    let skinned_wgsl = read_runtime_src("graphics/shader/wgsl/zr_geometry_skinned.wgsl");
    let forward_wgsl = read_runtime_src("graphics/shader/wgsl/zr_template_forward.wgsl");
    let gbuffer_wgsl = read_runtime_src("graphics/shader/wgsl/zr_template_gbuffer.wgsl");
    let depth_alpha_wgsl = read_runtime_src("graphics/shader/wgsl/zr_template_depth_alpha.wgsl");
    let shadow_alpha_wgsl = read_runtime_src("graphics/shader/wgsl/zr_template_shadow_alpha.wgsl");
    let velocity_wgsl = read_runtime_src("graphics/shader/wgsl/zr_template_velocity.wgsl");
    let velocity_alpha_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_template_velocity_alpha.wgsl");
    let taa_reactive_mask_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_template_taa_reactive_mask.wgsl");
    let standard_pbr_wgsl = read_runtime_src("graphics/shader/wgsl/zr_shading_standard_pbr.wgsl");
    let deferred_standard_pbr_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_shade_deferred_standard_pbr.wgsl");
    let deferred_blinn_phong_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_shade_deferred_blinn_phong.wgsl");
    let deferred_unlit_wgsl = read_runtime_src("graphics/shader/wgsl/zr_shade_deferred_unlit.wgsl");

    assert_contains_all(
        "scene runtime include exposes current scene uniform ABI",
        &scene_runtime_wgsl,
        &[
            "struct SceneUniform",
            "view_proj: mat4x4<f32>",
            "previous_view_proj_unjittered: mat4x4<f32>",
            "motion_params: vec4<f32>",
            "@group(0) @binding(0) var<uniform> scene: SceneUniform",
        ],
    );
    assert_contains_all(
        "gpu scene include exposes runtime transform and palette ABI",
        &gpu_scene_wgsl,
        &[
            "@group(3) @binding(0) var<storage, read> zr_primitive_data",
            "@group(3) @binding(1) var<storage, read> zr_instance_data",
            "@group(3) @binding(3) var<uniform> zr_skinned_joint_palette",
            "fn zr_world_from_local(instance_index: u32) -> mat4x4<f32>",
            "fn zr_previous_world_from_local(instance_index: u32) -> mat4x4<f32>",
            "fn zr_skinned_joint_matrix(joint_index: u32) -> mat4x4<f32>",
        ],
    );
    assert_contains_all(
        "surface types carry runtime-transformed material interpolation channels",
        &surface_types,
        &[
            "@location(2) uv0: vec2<f32>",
            "@location(3) joints: vec4<u32>",
            "@location(4) weights: vec4<f32>",
            "@location(5) tangent: vec4<f32>",
            "@location(6) color: vec4<f32>",
            "@location(7) uv1: vec2<f32>",
            "@location(3) uv1: vec2<f32>",
            "@location(4) tangent_ws: vec3<f32>",
            "@location(5) tangent_handedness: f32",
            "@location(7) tint: vec4<f32>",
            "@location(8) shadow_params: vec4<f32>",
            "struct ZrShadingContext",
            "fn zr_build_shading_context(input: ZrVertexOutput) -> ZrShadingContext",
            "instance_index: u32",
            "tangent_os: vec4<f32>",
            "let world_from_local = zr_world_from_local(instance_index);",
            "let position_ws = world_from_local * vec4<f32>(position_os, 1.0);",
            "output.clip_position = scene.view_proj * position_ws;",
            "output.position_ws = position_ws.xyz;",
            "output.normal_ws = zr_normalize_or_zero((world_from_local * vec4<f32>(normal_os, 0.0)).xyz);",
            "output.uv1 = uv1;",
            "output.tangent_ws = zr_normalize_or_zero((world_from_local * vec4<f32>(tangent_os.xyz, 0.0)).xyz);",
            "output.tangent_handedness = select(-1.0, 1.0, tangent_os.w >= 0.0);",
            "output.tint = zr_gpu_scene_tint(instance_index);",
            "output.shadow_params = zr_gpu_scene_shadow_params(instance_index);",
            "fn zr_normalize_or_zero(value: vec3<f32>) -> vec3<f32>",
            "alpha_cutoff: f32",
            "unlit: f32",
            "shading_model_id: u32",
            "fn zr_surface_fails_alpha_clip(surface: ZrSurfaceOutput) -> bool",
        ],
    );
    assert_contains_all(
        "static geometry include exposes fetch contract",
        &static_wgsl,
        &[
            "fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32>",
            "fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32>",
            "fn fetch_normal(v: ZrVertexInput, instance_index: u32) -> vec3<f32>",
            "fn fetch_tangent(v: ZrVertexInput, instance_index: u32) -> vec4<f32>",
            "fn fetch_uv0(v: ZrVertexInput) -> vec2<f32>",
            "fn fetch_uv1(v: ZrVertexInput) -> vec2<f32>",
        ],
    );
    assert_contains_all(
        "skinned geometry include reuses runtime gpu scene palette contract",
        &skinned_wgsl,
        &[
            "fn zr_template_skin_weight(joint_index: u32, weight: f32, joint_count: u32) -> f32",
            "fn zr_template_identity_mat4() -> mat4x4<f32>",
            "zr_skinned_joint_matrix(v.joints.x)",
            "zr_previous_skinned_joint_matrix(v.joints.x)",
            "zr_normalize_or_zero((zr_skin_matrix(v) * vec4<f32>(v.normal, 0.0)).xyz)",
            "fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32>",
        ],
    );
    assert_contains_all(
        "forward template passes uv and tangent fetches into surface output",
        &forward_wgsl,
        &[
            "fetch_tangent(v, instance_index),",
            "fetch_uv0(v),",
            "fetch_uv1(v),",
            "zr_build_vertex_output(",
            "instance_index,",
            "fn zr_vs_main_impl(",
            "fn zr_fs_main_impl(",
            "fn vs_main(",
            "fn fs_main(",
            "return zr_vs_main_impl(v, instance_index);",
            "return zr_fs_main_impl(input);",
            "fn zr_apply_alpha_clip(surface: ZrSurfaceOutput)",
            "discard;",
            "zr_apply_alpha_clip(surface);",
            "shade_forward(surface, zr_build_shading_context(input))",
        ],
    );
    assert_contains_all(
        "standard pbr shading include consumes runtime light grid and shadow inputs",
        &standard_pbr_wgsl,
        &[
            "fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32>",
            "fn zr_standard_pbr_gpu_light_lighting",
            "fn zr_standard_pbr_shade_gpu_light_index",
            "const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u",
            "const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u",
            "const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u",
            "fn zr_standard_pbr_shade_blinn_phong_light_vector",
            "surface.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID",
            "surface.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID",
            "zr_light_zbin_header",
            "zr_light_mask_word",
            "zr_gpu_light(light_index)",
            "ZR_FEATURE_RECEIVE_SHADOWS && ctx.shadow_params.z > 0.5",
            "zr_gpu_light_shadow_visibility(light, light_type, ctx.position_ws, view_z)",
            "scene.ambient_color.rgb * surface.occlusion",
        ],
    );
    assert_contains_all(
        "gbuffer template clips alpha-tested material output",
        &gbuffer_wgsl,
        &[
            "let surface = zr_material_surface(input);",
            "fn zr_fs_main_impl(",
            "fn fs_main(",
            "fn zr_apply_alpha_clip(surface: ZrSurfaceOutput)",
            "discard;",
            "zr_apply_alpha_clip(surface);",
        ],
    );
    assert_contains_all(
        "depth alpha template clips material surface alpha",
        &depth_alpha_wgsl,
        &[
            "@fragment",
            "let surface = zr_material_surface(input);",
            "fn zr_vs_main_impl(",
            "fn zr_fs_main_impl(",
            "fn vs_main(",
            "fn fs_main(",
            "fn zr_apply_alpha_clip(surface: ZrSurfaceOutput)",
            "discard;",
            "zr_apply_alpha_clip(surface);",
            "fetch_tangent(v, instance_index),",
            "fetch_uv1(v),",
        ],
    );
    assert_contains_all(
        "shadow alpha template clips material surface alpha",
        &shadow_alpha_wgsl,
        &[
            "@fragment",
            "let surface = zr_material_surface(input);",
            "fn zr_vs_main_impl(",
            "fn zr_fs_main_impl(",
            "fn vs_main(",
            "fn fs_main(",
            "fn zr_apply_alpha_clip(surface: ZrSurfaceOutput)",
            "discard;",
            "zr_apply_alpha_clip(surface);",
            "fetch_tangent(v, instance_index),",
            "fetch_uv1(v),",
        ],
    );
    assert_contains_all(
        "velocity template owns previous-position motion-vector output",
        &velocity_wgsl,
        &[
            "struct ZrVelocityVertexInput",
            "@location(8) previous_position",
            "let previous_input = zr_velocity_vertex_input(v, v.previous_position);",
            "fetch_prev_position(previous_input, instance_index)",
            "scene.view_proj_unjittered * current_world",
            "scene.previous_view_proj_unjittered * previous_world",
            "fn zr_vs_main_impl(",
            "fn vs_main(",
            "fn zr_velocity_clip_to_uv",
            "fn zr_velocity_apply_alpha_clip(input: ZrVelocityVertexOutput)",
            "fn fs_main(input: ZrVelocityVertexOutput)",
        ],
    );
    assert_contains_all(
        "velocity alpha template owns material alpha clip",
        &velocity_alpha_wgsl,
        &[
            "struct ZrVelocityVertexInput",
            "@location(8) previous_position",
            "fn zr_velocity_material_input(input: ZrVelocityVertexOutput) -> ZrVertexOutput",
            "let surface = zr_material_surface(zr_velocity_material_input(input));",
            "zr_surface_fails_alpha_clip(surface)",
            "discard;",
            "fn fs_main(input: ZrVelocityVertexOutput)",
        ],
    );
    assert_contains_all(
        "taa reactive mask template owns material reactive mask output",
        &taa_reactive_mask_wgsl,
        &[
            "fn zr_vs_main_impl(v: ZrVertexInput, instance_index: u32) -> ZrVertexOutput",
            "fetch_position(v, instance_index)",
            "fetch_tangent(v, instance_index)",
            "fetch_uv1(v)",
            "fn vs_main(v: ZrVertexInput",
            "fn fs_taa_reactive_mask(input: ZrVertexOutput) -> @location(0) f32",
            "let surface = zr_material_surface(input);",
            "surface.base_color.a",
            "surface.custom0.x",
            "fn fs_taa_reactive_material_mask(input: ZrVertexOutput) -> @location(0) f32",
            "let reactive_mask = clamp(surface.custom0.x, 0.0, 1.0);",
            "zr_discard_empty_taa_reactive_mask(reactive_mask)",
        ],
    );
    assert_contains_all(
        "deferred standard pbr include delegates to lit deferred shading",
        &deferred_standard_pbr_wgsl,
        &[
            "fn shade_deferred_standard_pbr",
            "ZR_SHADING_MODEL_STANDARD_PBR_ID",
            "shade_deferred_lit(position, coord, albedo, material, normal, ZR_SHADING_MODEL_STANDARD_PBR_ID)",
        ],
    );
    assert_contains_all(
        "deferred blinn phong include delegates to lit deferred shading",
        &deferred_blinn_phong_wgsl,
        &[
            "fn shade_deferred_blinn_phong",
            "ZR_SHADING_MODEL_BLINN_PHONG_ID",
            "shade_deferred_lit(position, coord, albedo, material, normal, ZR_SHADING_MODEL_BLINN_PHONG_ID)",
        ],
    );
    assert_contains_all(
        "deferred unlit include returns albedo directly",
        &deferred_unlit_wgsl,
        &["fn shade_deferred_unlit", "return albedo;"],
    );

    for (path, source) in [
        (
            "structure_convention/production_file_budget/render_shader_template_assembly.rs",
            include_str!("../render_shader_template_assembly.rs"),
        ),
        (
            "structure_convention/production_file_budget/render_shader_template_assembly/wgsl_contracts.rs",
            include_str!("wgsl_contracts.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below R4.3 production/test owner budget; got {line_count}"
        );
    }
}
