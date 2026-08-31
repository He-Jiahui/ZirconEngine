use std::sync::Arc;

use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use super::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
    RenderArtifactLayout, RenderArtifactManifest, RenderArtifactManifestError,
    RenderArtifactResidencyClass, RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout,
    RenderSubresourceId,
};

#[path = "tests/mesh_layout.rs"]
mod mesh_layout;
use mesh_layout::mesh_test_layout;

const TEST_BLOCK_ALIGNMENT: u32 = 256;
const TEST_TEXTURE_WIDTH: u32 = 16;
const TEST_TEXTURE_HEIGHT: u32 = 8;

fn resource(label: &str, kind: ResourceKind) -> UntypedResourceHandle {
    UntypedResourceHandle::new(ResourceId::from_stable_label(label), kind)
}

fn content_id(seed: u8) -> RenderArtifactContentId {
    RenderArtifactContentId::from_bytes([seed; 32])
}

fn block(
    seed: u8,
    subresource: RenderSubresourceId,
    residency: RenderArtifactResidencyClass,
    format: &str,
    dependencies: Vec<RenderSubresourceId>,
) -> RenderArtifactBlockDescriptor {
    RenderArtifactBlockDescriptor::new(
        subresource,
        content_id(seed),
        RenderArtifactBlockCodec::Raw,
        4_096,
        4_096,
        TEST_BLOCK_ALIGNMENT,
        Arc::from(format),
        residency,
        dependencies,
    )
}

fn texture_block(mip: u32, layer: u32) -> RenderArtifactBlockDescriptor {
    let residency = if mip >= 2 {
        RenderArtifactResidencyClass::Bootstrap
    } else {
        RenderArtifactResidencyClass::Streamable
    };
    let dependencies = (mip < 3)
        .then_some(RenderSubresourceId::TextureMipLayer {
            mip: mip + 1,
            layer,
        })
        .into_iter()
        .collect();
    let decoded_bytes = bc7_mip_bytes(mip);
    RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip, layer },
        content_id((mip * 2 + layer + 1) as u8),
        RenderArtifactBlockCodec::Raw,
        decoded_bytes,
        decoded_bytes,
        TEST_BLOCK_ALIGNMENT,
        Arc::from("bc7-rgba-unorm-srgb"),
        residency,
        dependencies,
    )
}

fn bc7_texture_layout(
    mip_count: u32,
    array_layer_count: u32,
    bootstrap_first_mip: u32,
) -> RenderArtifactLayout {
    RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
        RenderArtifactTextureBlockFormat::new(Arc::from("bc7-rgba-unorm-srgb"), 4, 4, 16),
        TEST_TEXTURE_WIDTH,
        TEST_TEXTURE_HEIGHT,
        mip_count,
        array_layer_count,
        bootstrap_first_mip,
    ))
}

fn rgba8_texture_layout(width: u32, height: u32) -> RenderArtifactLayout {
    RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
        RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
        width,
        height,
        1,
        1,
        0,
    ))
}

fn bc7_mip_bytes(mip: u32) -> u64 {
    let width = (TEST_TEXTURE_WIDTH >> mip).max(1);
    let height = (TEST_TEXTURE_HEIGHT >> mip).max(1);
    u64::from(width.div_ceil(4)) * u64::from(height.div_ceil(4)) * 16
}

#[test]
fn render_texture_manifest_canonicalizes_every_mip_layer_and_marks_bootstrap_tail() {
    let texture = resource("render-artifact/texture/canonical", ResourceKind::Texture);
    let mut blocks = Vec::new();
    for mip in (0..4).rev() {
        for layer in (0..2).rev() {
            blocks.push(texture_block(mip, layer));
        }
    }

    let manifest = RenderArtifactManifest::new(
        texture,
        7,
        Arc::from("windows-dx12-sm6"),
        bc7_texture_layout(4, 2, 2),
        Vec::new(),
        blocks,
    )
    .unwrap_or_else(|error| panic!("texture manifest failed: {error}"));

    assert_eq!(manifest.blocks().len(), 8);
    assert_eq!(
        manifest.blocks()[0].subresource(),
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 }
    );
    assert_eq!(
        manifest.blocks()[7].subresource(),
        RenderSubresourceId::TextureMipLayer { mip: 3, layer: 1 }
    );
    assert_eq!(manifest.bootstrap_blocks().count(), 4);
    assert_eq!(manifest.streamable_blocks().count(), 4);
    assert_eq!(
        manifest
            .block(RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 })
            .map(RenderArtifactBlockDescriptor::dependencies),
        Some([RenderSubresourceId::TextureMipLayer { mip: 2, layer: 0 }].as_slice())
    );
}

#[test]
fn render_texture_manifest_derives_tight_upload_layout_for_each_semantic_block() {
    let texture = resource(
        "render-artifact/texture/upload-layout",
        ResourceKind::Texture,
    );
    let manifest = RenderArtifactManifest::new(
        texture,
        1,
        Arc::from("windows-dx12-sm6"),
        bc7_texture_layout(4, 1, 2),
        Vec::new(),
        (0..4).map(|mip| texture_block(mip, 0)).collect(),
    )
    .unwrap_or_else(|error| panic!("texture upload-layout manifest failed: {error}"));

    let layout = manifest
        .texture_subresource_layout(RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 })
        .unwrap_or_else(|| panic!("missing texture subresource upload layout"));

    assert_eq!(layout.width(), 8);
    assert_eq!(layout.height(), 4);
    assert_eq!(layout.bytes_per_row(), 32);
    assert_eq!(layout.block_rows(), 1);
    assert_eq!(layout.decoded_bytes(), 32);
}

#[test]
fn render_texture_manifest_rejects_decoded_bytes_that_do_not_match_tight_upload_layout() {
    let texture = resource(
        "render-artifact/texture/decoded-size",
        ResourceKind::Texture,
    );
    let mut invalid = texture_block(0, 0);
    invalid = RenderArtifactBlockDescriptor::new(
        invalid.subresource(),
        invalid.content_id(),
        RenderArtifactBlockCodec::Raw,
        invalid.encoded_bytes() + 16,
        invalid.decoded_bytes() + 16,
        invalid.alignment(),
        Arc::from(invalid.platform_format()),
        invalid.residency(),
        invalid.dependencies().to_vec(),
    );

    let result = RenderArtifactManifest::new(
        texture,
        1,
        Arc::from("windows-dx12-sm6"),
        bc7_texture_layout(1, 1, 0),
        Vec::new(),
        vec![invalid],
    );

    assert!(matches!(
        result,
        Err(
            RenderArtifactManifestError::TextureBlockDecodedSizeMismatch {
                expected: 128,
                actual: 144,
                ..
            }
        )
    ));
}

#[test]
fn render_texture_manifest_rejects_a_mip_chain_longer_than_its_extent() {
    let texture = resource("render-artifact/texture/mip-range", ResourceKind::Texture);
    let result = RenderArtifactManifest::new(
        texture,
        1,
        Arc::from("windows-dx12-sm6"),
        bc7_texture_layout(6, 1, 5),
        Vec::new(),
        (0..6).map(|mip| texture_block(mip, 0)).collect(),
    );

    assert!(matches!(
        result,
        Err(RenderArtifactManifestError::TextureMipCountOutOfRange {
            mip_count: 6,
            max_mip_count: 5,
            ..
        })
    ));
}

#[test]
fn render_texture_manifest_rejects_a_missing_semantic_mip_layer() {
    let texture = resource("render-artifact/texture/missing", ResourceKind::Texture);
    let blocks = (0..4)
        .flat_map(|mip| (0..2).map(move |layer| (mip, layer)))
        .filter(|&(mip, layer)| (mip, layer) != (1, 1))
        .map(|(mip, layer)| texture_block(mip, layer))
        .collect();

    let result = RenderArtifactManifest::new(
        texture,
        1,
        Arc::from("windows-dx12-sm6"),
        bc7_texture_layout(4, 2, 2),
        Vec::new(),
        blocks,
    );

    assert!(matches!(
        result,
        Err(RenderArtifactManifestError::BlockCountMismatch {
            expected: 8,
            actual: 7
        })
    ));
}

#[test]
fn render_manifest_rejects_duplicate_subresource_owners() {
    let texture = resource("render-artifact/texture/duplicate", ResourceKind::Texture);
    let duplicate = texture_block(0, 0);
    let result = RenderArtifactManifest::new(
        texture,
        1,
        Arc::from("windows-dx12-sm6"),
        bc7_texture_layout(1, 1, 0),
        Vec::new(),
        vec![duplicate.clone(), duplicate],
    );

    assert!(matches!(
        result,
        Err(RenderArtifactManifestError::DuplicateSubresource {
            subresource: RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 }
        })
    ));
}

#[test]
fn render_manifest_rejects_invalid_codec_sizes_and_alignment() {
    let texture = resource(
        "render-artifact/texture/invalid-block",
        ResourceKind::Texture,
    );
    let invalid_sizes = RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        content_id(1),
        RenderArtifactBlockCodec::Raw,
        2_048,
        4_096,
        TEST_BLOCK_ALIGNMENT,
        Arc::from("rgba8unorm"),
        RenderArtifactResidencyClass::Bootstrap,
        Vec::new(),
    );
    let result = RenderArtifactManifest::new(
        texture,
        1,
        Arc::from("windows-dx12-sm6"),
        rgba8_texture_layout(32, 32),
        Vec::new(),
        vec![invalid_sizes],
    );
    assert!(matches!(
        result,
        Err(RenderArtifactManifestError::RawCodecSizeMismatch { .. })
    ));

    let invalid_alignment = RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        content_id(2),
        RenderArtifactBlockCodec::Zstd,
        2_048,
        4_096,
        384,
        Arc::from("rgba8unorm"),
        RenderArtifactResidencyClass::Bootstrap,
        Vec::new(),
    );
    let result = RenderArtifactManifest::new(
        texture,
        1,
        Arc::from("windows-dx12-sm6"),
        rgba8_texture_layout(32, 32),
        Vec::new(),
        vec![invalid_alignment],
    );
    assert!(matches!(
        result,
        Err(RenderArtifactManifestError::InvalidBlockAlignment { alignment: 384, .. })
    ));
}

#[test]
fn render_mesh_manifest_requires_all_lods_and_accepts_owned_cluster_pages() {
    let mesh = resource("render-artifact/mesh/lods", ResourceKind::Mesh);
    let material = resource(
        "render-artifact/material/dependency",
        ResourceKind::Material,
    );
    let shader = resource("render-artifact/shader/dependency", ResourceKind::Shader);
    let blocks = vec![
        block(
            5,
            RenderSubresourceId::MeshClusterPage { lod: 1, page: 4 },
            RenderArtifactResidencyClass::Streamable,
            "mesh-position-normal-uv-index32",
            vec![RenderSubresourceId::MeshLod { lod: 1 }],
        ),
        block(
            3,
            RenderSubresourceId::MeshLod { lod: 2 },
            RenderArtifactResidencyClass::Bootstrap,
            "mesh-position-normal-uv-index32",
            Vec::new(),
        ),
        block(
            1,
            RenderSubresourceId::MeshLod { lod: 0 },
            RenderArtifactResidencyClass::Streamable,
            "mesh-position-normal-uv-index32",
            vec![RenderSubresourceId::MeshLod { lod: 1 }],
        ),
        block(
            2,
            RenderSubresourceId::MeshLod { lod: 1 },
            RenderArtifactResidencyClass::Streamable,
            "mesh-position-normal-uv-index32",
            vec![RenderSubresourceId::MeshLod { lod: 2 }],
        ),
        block(
            4,
            RenderSubresourceId::MeshClusterPage { lod: 2, page: 0 },
            RenderArtifactResidencyClass::Bootstrap,
            "mesh-position-normal-uv-index32",
            vec![RenderSubresourceId::MeshLod { lod: 2 }],
        ),
    ];

    let manifest = RenderArtifactManifest::new(
        mesh,
        11,
        Arc::from("linux-vulkan-sm6"),
        RenderArtifactLayout::mesh(mesh_test_layout("mesh-position-normal-uv-index32", 3, 2)),
        vec![shader, material, shader],
        blocks,
    )
    .unwrap_or_else(|error| panic!("mesh manifest failed: {error}"));

    assert_eq!(manifest.asset_dependencies(), &[material, shader]);
    assert_eq!(manifest.blocks().len(), 5);
    assert_eq!(manifest.bootstrap_blocks().count(), 2);
    assert!(
        manifest
            .block(RenderSubresourceId::MeshClusterPage { lod: 2, page: 0 })
            .is_some()
    );
}

#[test]
fn render_manifest_rejects_cyclic_semantic_block_dependencies() {
    let mesh = resource("render-artifact/mesh/cycle", ResourceKind::Mesh);
    let blocks = vec![
        block(
            1,
            RenderSubresourceId::MeshLod { lod: 0 },
            RenderArtifactResidencyClass::Bootstrap,
            "mesh-position-index32",
            vec![RenderSubresourceId::MeshLod { lod: 1 }],
        ),
        block(
            2,
            RenderSubresourceId::MeshLod { lod: 1 },
            RenderArtifactResidencyClass::Bootstrap,
            "mesh-position-index32",
            vec![RenderSubresourceId::MeshLod { lod: 0 }],
        ),
    ];

    let result = RenderArtifactManifest::new(
        mesh,
        1,
        Arc::from("linux-vulkan-sm6"),
        RenderArtifactLayout::mesh(mesh_test_layout("mesh-position-index32", 2, 0)),
        Vec::new(),
        blocks,
    );

    assert!(matches!(
        result,
        Err(RenderArtifactManifestError::CyclicBlockDependencies)
    ));
}
