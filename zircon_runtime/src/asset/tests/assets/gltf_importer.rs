use std::fs;

use super::gltf_external_fixtures::{write_external_texture_gltf, write_missing_buffer_gltf};
use super::gltf_primitive_fixtures::{
    write_line_gltf, write_node_animation_gltf, write_skinned_triangle_gltf,
    write_tangent_color_triangle_gltf, write_texture_transform_triangle_gltf, write_triangle_gltf,
    write_two_primitive_gltf, write_uv_channel_triangle_gltf,
};
use super::gltf_scene_fixtures::write_two_scene_gltf;
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::importer_with_first_wave_plugin_fixtures;
use crate::asset::{
    AssetImportOutcome, AssetImporter, AssetImporterCapabilityStatus, AssetUri, DataAssetFormat,
    ImportedAsset, ImportedAssetEntry, MaterialAsset, MeshAttributeValues, ModelPrimitiveAsset,
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
    MESH_ATTRIBUTE_UV1,
};
use crate::core::framework::animation::{AnimationChannelValueAsset, AnimationInterpolationAsset};

mod basic_import;
mod external_inputs;
mod labeled_subassets;
mod material_transforms;
mod multi_primitive;
mod multi_scene;
mod vertex_channels;
mod woc_required_extensions;

fn virtual_geometry_import_settings() -> toml::Table {
    toml::from_str(
        r#"
            [virtual_geometry]
            enabled = true
        "#,
    )
    .unwrap()
}

fn entry_for_label<'a>(
    outcome: &'a AssetImportOutcome,
    root_uri: &AssetUri,
    label: &str,
) -> &'a ImportedAssetEntry {
    let locator = label_uri(root_uri, label);
    entry_for_locator(outcome, &locator)
}

fn assert_cooked_virtual_geometry(primitive: &ModelPrimitiveAsset, source_hint: &str) {
    let virtual_geometry = primitive
        .virtual_geometry
        .as_ref()
        .expect("imported model primitive should carry cooked virtual geometry");
    assert!(!virtual_geometry.hierarchy_buffer.is_empty());
    assert!(!virtual_geometry.cluster_headers.is_empty());
    assert!(!virtual_geometry.cluster_page_headers.is_empty());
    assert!(!virtual_geometry.cluster_page_data.is_empty());
    assert!(!virtual_geometry.root_page_table.is_empty());
    assert_eq!(
        virtual_geometry.debug.source_hint.as_deref(),
        Some(source_hint)
    );
    assert_virtual_geometry_vertex_ordinals(primitive);
}

fn assert_virtual_geometry_vertex_ordinals(primitive: &ModelPrimitiveAsset) {
    let ordinals = primitive
        .vertices
        .iter()
        .map(|vertex| {
            ModelPrimitiveAsset::decode_virtual_geometry_vertex_ordinal(vertex.joint_indices)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ordinals,
        (0..primitive.vertices.len() as u32).collect::<Vec<_>>()
    );
}

fn identity_bind_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn assert_scene_entity(entry: &ImportedAssetEntry, expected_name: &str, root_uri: &AssetUri) {
    match &entry.asset {
        ImportedAsset::Scene(scene) => {
            assert_eq!(scene.entities.len(), 1);
            let entity = &scene.entities[0];
            assert_eq!(entity.name, expected_name);
            assert_eq!(entity.parent, None);
            let mesh = entity.mesh.as_ref().expect("scene entity mesh");
            assert_eq!(mesh.model.locator, label_uri(root_uri, "Mesh0"));
            assert_eq!(mesh.material.locator, label_uri(root_uri, "Material0"));
            assert_eq!(mesh.primitives.len(), 1);
            assert_eq!(
                mesh.primitives[0].mesh.locator,
                label_uri(root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                mesh.primitives[0].material.locator,
                label_uri(root_uri, "Material0")
            );
        }
        other => panic!("unexpected scene asset: {other:?}"),
    }
}

fn assert_texture_slot_transform(
    material: &MaterialAsset,
    root_uri: &AssetUri,
    slot: &str,
    expected_scale: [f32; 2],
    expected_offset: [f32; 2],
    expected_uv_channel: u32,
) {
    let value = material
        .texture_slots
        .get(slot)
        .unwrap_or_else(|| panic!("{slot} texture slot should be imported"));
    assert_eq!(
        value.reference.as_ref().unwrap().locator,
        label_uri(root_uri, "Texture0")
    );
    let transform = value.texture_transform();
    assert_vec2_near(transform.scale, expected_scale);
    assert_vec2_near(transform.offset, expected_offset);
    assert_eq!(value.texture_uv_channel(), expected_uv_channel);
}

fn assert_vec2_near(actual: [f32; 2], expected: [f32; 2]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 0.000_001,
            "expected {expected:?}, got {actual:?}"
        );
    }
}

fn entry_for_locator<'a>(
    outcome: &'a AssetImportOutcome,
    locator: &AssetUri,
) -> &'a ImportedAssetEntry {
    outcome
        .entries
        .iter()
        .find(|entry| entry.locator == *locator)
        .unwrap_or_else(|| panic!("missing gltf subasset {locator}"))
}

fn label_uri(root_uri: &AssetUri, label: &str) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}")).unwrap()
}
