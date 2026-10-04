use super::{fragment_entry_points, UI_MATERIAL_SHADER, UI_SURFACE_BLEND};

#[test]
fn ui_material_shader_exposes_surface_entry_points_and_material_helpers() {
    for entry_point in [
        "damage_clear_vs_main",
        "damage_clear_fs_main",
        "solid_vs_main",
        "solid_instance_vs_main",
        "solid_fs_linear_target",
        "solid_fs_byte_target",
        "solid_instance_fs_linear_target",
        "solid_instance_fs_byte_target",
        "image_vs_main",
        "image_fs_linear_target",
        "image_fs_byte_target",
    ] {
        assert!(
            UI_MATERIAL_SHADER.contains(entry_point),
            "ui_material.wgsl must expose `{entry_point}`"
        );
    }

    for helper in [
        "material_tint",
        "premultiply_alpha",
        "srgb_to_linear",
        "linear_to_srgb",
        "rounded_box_distance",
        "rounded_box_alpha",
        "material_solid_color",
        "material_image_color",
    ] {
        assert!(
            UI_MATERIAL_SHADER.contains(helper),
            "ui_material.wgsl must keep the Material UI helper `{helper}`"
        );
    }
}

#[test]
fn fragment_entry_points_follow_the_target_transfer_function() {
    let linear = fragment_entry_points(wgpu::TextureFormat::Bgra8UnormSrgb);
    assert_eq!(linear.solid, "solid_fs_linear_target");
    assert_eq!(linear.solid_instance, "solid_instance_fs_linear_target");
    assert_eq!(linear.image, "image_fs_linear_target");

    let byte = fragment_entry_points(wgpu::TextureFormat::Bgra8Unorm);
    assert_eq!(byte.solid, "solid_fs_byte_target");
    assert_eq!(byte.solid_instance, "solid_instance_fs_byte_target");
    assert_eq!(byte.image, "image_fs_byte_target");
}

#[test]
fn analytic_solid_vertex_abi_carries_pixel_space_shape_parameters() {
    assert_eq!(std::mem::size_of::<super::SolidVertex>(), 64);
    assert_eq!(std::mem::offset_of!(super::SolidVertex, position), 0);
    assert_eq!(std::mem::offset_of!(super::SolidVertex, color), 8);
    assert_eq!(std::mem::offset_of!(super::SolidVertex, local_position), 24);
    assert_eq!(std::mem::offset_of!(super::SolidVertex, half_extent), 32);
    assert_eq!(std::mem::offset_of!(super::SolidVertex, corner_radius), 40);
    assert_eq!(std::mem::offset_of!(super::SolidVertex, border_width), 44);
    assert_eq!(std::mem::offset_of!(super::SolidVertex, fill_color), 48);
    assert!(UI_MATERIAL_SHADER.contains("fwidth(outer_distance)"));
    assert!(UI_MATERIAL_SHADER
        .contains("smoothstep(\n        distance_width * -0.5,\n        distance_width * 0.5,\n        signed_distance,\n    )"));
    assert!(UI_MATERIAL_SHADER.contains("array<vec2<f32>, 16>"));
    assert!(UI_MATERIAL_SHADER.contains("let subpixel_filter_scale = 0.25"));
    assert!(UI_MATERIAL_SHADER.contains("let coverage_guard = distance_width * 0.75"));
    assert!(UI_MATERIAL_SHADER.contains("let inner_coverage_guard = inner_distance_width * 0.75"));
    assert!(!UI_MATERIAL_SHADER.contains("fwidth(sample_outer_distance)"));
    assert!(UI_MATERIAL_SHADER
        .contains("return vec2<f32>(outer_coverage_sum, inner_coverage_sum) * 0.0625"));
    assert!(UI_MATERIAL_SHADER.contains("coverages.x - coverages.y"));
    assert!(UI_MATERIAL_SHADER.contains("return fill + border"));
}

#[test]
fn compact_solid_instance_abi_is_one_record_per_quad() {
    assert_eq!(std::mem::size_of::<super::SolidInstance>(), 32);
    assert_eq!(std::mem::offset_of!(super::SolidInstance, min_position), 0);
    assert_eq!(std::mem::offset_of!(super::SolidInstance, max_position), 8);
    assert_eq!(std::mem::offset_of!(super::SolidInstance, color), 16);
    assert!(UI_MATERIAL_SHADER.contains("@builtin(vertex_index) vertex_index: u32"));
    assert!(UI_MATERIAL_SHADER.contains("array<vec2<f32>, 6>"));
}

#[test]
fn ui_material_shader_routes_fragment_outputs_through_material_helpers() {
    assert_eq!(
        UI_SURFACE_BLEND,
        wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        "solid and image UI surfaces must blend premultiplied fragment output"
    );
    assert!(
        UI_MATERIAL_SHADER.contains("return material_solid_color(input.color, true);")
            && UI_MATERIAL_SHADER.contains("return material_solid_color(input.color, false);"),
        "flat solid fragment output must go through the Material solid color path"
    );
    assert!(
        UI_MATERIAL_SHADER.contains("input.color.a * coverage"),
        "analytic rounded solids must apply distance-field coverage before premultiplication"
    );
    assert!(
        UI_MATERIAL_SHADER
            .contains("premultiply_alpha(vec4<f32>(srgb_to_linear(tinted.rgb), tinted.a))"),
        "linear targets must decode solid sRGB before coverage alpha premultiplication"
    );
    assert!(
        UI_MATERIAL_SHADER
            .contains("return material_image_color(textureSample(source_texture, source_sampler, input.uv), true);")
            && UI_MATERIAL_SHADER.contains(
                "return material_image_color(textureSample(source_texture, source_sampler, input.uv), false);"
            ),
        "image fragment output must go through the Material image color path"
    );
    let image_helper = UI_MATERIAL_SHADER
        .split("fn material_image_color")
        .nth(1)
        .and_then(|source| source.split('}').next())
        .expect("image material helper must remain explicit");
    assert!(
        !image_helper.contains("premultiply_alpha"),
        "filtered image texels are already premultiplied and must not be multiplied twice"
    );
    assert!(
        UI_MATERIAL_SHADER.contains("linear_to_srgb(clamp(tinted.rgb / tinted.a")
            && UI_MATERIAL_SHADER.contains("straight_srgb * tinted.a"),
        "the byte-target fallback must encode straight linear RGB before restoring premultiplication"
    );
}
