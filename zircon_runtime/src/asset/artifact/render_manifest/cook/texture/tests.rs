use std::sync::Arc;

use crate::asset::{AssetUri, TextureAsset, TextureAssetDescriptor, TextureUploadSupport};
use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use super::*;

fn texture_resource(label: &str) -> UntypedResourceHandle {
    UntypedResourceHandle::new(ResourceId::from_stable_label(label), ResourceKind::Texture)
}

fn texture_uri(path: &str) -> AssetUri {
    AssetUri::parse(path).unwrap_or_else(|error| panic!("invalid texture test URI: {error}"))
}

fn settings(bootstrap_first_mip: u32) -> RenderArtifactTextureCookSettings {
    RenderArtifactTextureCookSettings::new(
        Arc::from("windows-dx12-sm6"),
        bootstrap_first_mip,
        256,
        TextureUploadSupport::all_compressed(),
    )
}

#[test]
fn render_texture_cook_slices_rgba_mip_layers_into_canonical_blocks() {
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.mip_count = 2;
    descriptor.array_layer_count = 2;
    descriptor.depth_or_array_layers = 2;
    let mut rgba = Vec::new();
    for (value, byte_count) in [(11, 64), (13, 64), (17, 16), (19, 16)] {
        rgba.extend(std::iter::repeat(value).take(byte_count));
    }
    let texture = TextureAsset::new_rgba8(texture_uri("res://textures/array-mips"), 4, 4, rgba)
        .with_descriptor(descriptor);

    let output = cook_texture_render_artifact(
        texture_resource("render-cook/rgba-array"),
        3,
        texture,
        settings(1),
    )
    .unwrap_or_else(|error| panic!("rgba semantic cook failed: {error}"));

    assert_eq!(output.blocks().len(), 4);
    assert_eq!(output.blocks()[0].bytes(), &[11; 64]);
    assert_eq!(output.blocks()[1].bytes(), &[13; 64]);
    assert_eq!(output.blocks()[2].bytes(), &[17; 16]);
    assert_eq!(output.blocks()[3].bytes(), &[19; 16]);
    assert_eq!(output.manifest().bootstrap_blocks().count(), 2);
    assert_eq!(
        output.blocks()[0].descriptor().dependencies(),
        &[RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 }]
    );
}

#[test]
fn render_texture_cook_strips_astc_container_header_from_the_semantic_block() {
    let mut container = vec![0_u8; 16];
    container[..4].copy_from_slice(b"\x13\xAB\xA1\x5C");
    container[4..7].copy_from_slice(&[4, 4, 1]);
    container[7..10].copy_from_slice(&[4, 0, 0]);
    container[10..13].copy_from_slice(&[4, 0, 0]);
    container[13..16].copy_from_slice(&[1, 0, 0]);
    let gpu_block = [29_u8; 16];
    container.extend_from_slice(&gpu_block);
    let texture = TextureAsset::new_container(
        texture_uri("res://textures/block.astc"),
        4,
        4,
        "astc/4x4x1",
        container,
        1,
        1,
    );

    let output = cook_texture_render_artifact(
        texture_resource("render-cook/astc"),
        1,
        texture,
        settings(0),
    )
    .unwrap_or_else(|error| panic!("ASTC semantic cook failed: {error}"));

    assert_eq!(output.blocks().len(), 1);
    assert_eq!(output.blocks()[0].bytes(), gpu_block.as_slice());
    assert_eq!(output.blocks()[0].descriptor().decoded_bytes(), 16);
    assert_eq!(
        output.blocks()[0].descriptor().platform_format(),
        "astc/4x4x1"
    );
}

#[test]
fn render_texture_cook_rejects_a_bootstrap_mip_outside_the_payload_chain() {
    let texture = TextureAsset::new_rgba8(
        texture_uri("res://textures/bootstrap-range"),
        2,
        2,
        vec![1_u8; 16],
    );

    let result = cook_texture_render_artifact(
        texture_resource("render-cook/bootstrap-range"),
        1,
        texture,
        settings(1),
    );

    assert!(matches!(
        result,
        Err(RenderArtifactTextureCookError::Manifest(
            RenderArtifactManifestError::TextureBootstrapMipOutOfRange { .. }
        ))
    ));
}
