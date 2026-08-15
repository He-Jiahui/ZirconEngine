use super::*;

#[test]
fn importer_preserves_gltf_texture_transform_on_standard_material_slots() {
    let root = unique_temp_project_root("texture_transform_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_texture_transform_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/texture_transform_triangle.gltf").unwrap();

    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    match &entry_for_label(&outcome, &root_uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            assert_texture_slot_transform(
                material,
                &root_uri,
                "base_color",
                [0.3, 0.4],
                [0.1, 0.2],
                1,
            );
            assert_texture_slot_transform(material, &root_uri, "normal", [0.5, 0.6], [0.3, 0.4], 0);
            assert_texture_slot_transform(
                material,
                &root_uri,
                "metallic_roughness",
                [0.4, 0.5],
                [0.2, 0.3],
                0,
            );
            assert_texture_slot_transform(
                material,
                &root_uri,
                "occlusion",
                [0.6, 0.7],
                [0.4, 0.5],
                1,
            );
            assert_texture_slot_transform(
                material,
                &root_uri,
                "emissive",
                [0.7, 0.8],
                [0.5, 0.6],
                1,
            );

            let descriptor = material.standard_material_descriptor();
            assert_vec2_near(descriptor.base_color_texture_transform.scale, [0.3, 0.4]);
            assert_vec2_near(descriptor.base_color_texture_transform.offset, [0.1, 0.2]);
            assert_eq!(descriptor.base_color_texture_uv_channel, 1);
            assert_vec2_near(descriptor.normal_texture_transform.scale, [0.5, 0.6]);
            assert_vec2_near(descriptor.normal_texture_transform.offset, [0.3, 0.4]);
            assert_eq!(descriptor.normal_texture_uv_channel, 0);
            assert_vec2_near(
                descriptor.metallic_roughness_texture_transform.scale,
                [0.4, 0.5],
            );
            assert_vec2_near(
                descriptor.metallic_roughness_texture_transform.offset,
                [0.2, 0.3],
            );
            assert_eq!(descriptor.metallic_roughness_texture_uv_channel, 0);
            assert_vec2_near(descriptor.occlusion_texture_transform.scale, [0.6, 0.7]);
            assert_vec2_near(descriptor.occlusion_texture_transform.offset, [0.4, 0.5]);
            assert_eq!(descriptor.occlusion_texture_uv_channel, 1);
            assert_eq!(material.occlusion_strength(), 0.25);
            assert_eq!(descriptor.occlusion_strength, 0.25);
            assert_vec2_near(descriptor.emissive_texture_transform.scale, [0.7, 0.8]);
            assert_vec2_near(descriptor.emissive_texture_transform.offset, [0.5, 0.6]);
            assert_eq!(descriptor.emissive_texture_uv_channel, 1);
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}
