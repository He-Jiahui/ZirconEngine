use crate::core::resource::ResourceLocator;

use super::*;

#[test]
fn shader_ide_module_stub_path_maps_logical_modules_to_files() {
    assert_eq!(
        shader_ide_relative_path_string(&shader_ide_module_stub_relative_path(
            "myproj::cloth::common"
        )),
        "modules/myproj/cloth/common.wgsl"
    );
    assert_eq!(
        shader_ide_relative_path_string(&shader_ide_module_stub_relative_path(
            "zr_surface_types.wgsl"
        )),
        "modules/builtin/zr_surface_types.wgsl"
    );
}

#[test]
fn shader_ide_generated_material_stub_path_is_scoped_by_source_uri() {
    let uri = ResourceLocator::parse("res://shaders/hero_cloth").unwrap();

    assert_eq!(
        shader_ide_relative_path_string(&shader_ide_generated_material_stub_relative_path(&uri)),
        "generated/res_shaders_hero_cloth.material.wgsl"
    );
}

#[test]
fn shader_ide_preview_paths_are_scoped_by_source_uri_and_variant() {
    let uri = ResourceLocator::parse("res://shaders/hero_cloth").unwrap();
    let variant = ShaderIdePreviewVariant::new(ShaderPassType::GBuffer, 1);

    assert_eq!(
        shader_ide_relative_path_string(&shader_ide_preview_relative_path(&uri, "default")),
        "preview/res_shaders_hero_cloth.default.wgsl"
    );
    assert_eq!(
        shader_ide_relative_path_string(&shader_ide_preview_segments_relative_path(
            &uri, "default"
        )),
        "preview/res_shaders_hero_cloth.default.segments.json"
    );
    assert_eq!(variant.name, "gbuffer_options_0x00000001");
    assert_eq!(
        shader_ide_relative_path_string(&shader_ide_preview_relative_path(&uri, &variant.name)),
        "preview/res_shaders_hero_cloth.gbuffer_options_0x00000001.wgsl"
    );
}
