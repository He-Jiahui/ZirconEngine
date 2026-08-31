use std::sync::Arc;

use crate::asset::artifact::{
    RenderArtifactLayout, RenderArtifactManifest, RenderArtifactManifestError,
    RenderArtifactMeshBounds, RenderArtifactMeshIndexFormat, RenderArtifactMeshLayout,
    RenderArtifactMeshLodLayout, RenderArtifactMeshVertexFormat,
};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::resource::ResourceKind;

use super::{RenderArtifactResidencyClass, RenderSubresourceId, block, resource};

pub(super) fn mesh_test_layout(
    platform_format: &str,
    lod_count: u16,
    bootstrap_first_lod: u16,
) -> RenderArtifactMeshLayout {
    RenderArtifactMeshLayout::new(
        Arc::from(platform_format),
        RenderArtifactMeshVertexFormat::StaticMeshV1,
        RenderArtifactMeshIndexFormat::Uint32,
        bootstrap_first_lod,
        (0..lod_count)
            .map(|lod| {
                RenderArtifactMeshLodLayout::new(
                    lod,
                    RenderMeshTopology::TriangleList,
                    32,
                    256,
                    3_072,
                    RenderArtifactMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
                )
            })
            .collect(),
    )
}

#[test]
fn render_mesh_manifest_exposes_upload_ready_vertex_index_ranges_and_bounds() {
    let mesh = resource("render-artifact/mesh/layout", ResourceKind::Mesh);
    let manifest = RenderArtifactManifest::new(
        mesh,
        1,
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::mesh(mesh_test_layout("zr-static-mesh-v1", 1, 0)),
        Vec::new(),
        vec![block(
            17,
            RenderSubresourceId::MeshLod { lod: 0 },
            RenderArtifactResidencyClass::Bootstrap,
            "zr-static-mesh-v1",
            Vec::new(),
        )],
    )
    .unwrap_or_else(|error| panic!("mesh upload layout manifest failed: {error}"));

    let layout = manifest
        .mesh_lod_layout(RenderSubresourceId::MeshLod { lod: 0 })
        .unwrap_or_else(|| panic!("missing mesh LOD upload layout"));

    assert_eq!(layout.vertex_range(), 0..3_072);
    assert_eq!(layout.index_range(), 3_072..4_096);
    assert_eq!(layout.bounds().min(), [-1.0; 3]);
    assert_eq!(layout.bounds().max(), [1.0; 3]);
}

#[test]
fn render_mesh_manifest_rejects_block_bytes_that_do_not_match_lod_layout() {
    let mesh = resource("render-artifact/mesh/decoded-size", ResourceKind::Mesh);
    let mut mismatched = block(
        19,
        RenderSubresourceId::MeshLod { lod: 0 },
        RenderArtifactResidencyClass::Bootstrap,
        "zr-static-mesh-v1",
        Vec::new(),
    );
    mismatched = crate::asset::artifact::RenderArtifactBlockDescriptor::new(
        mismatched.subresource(),
        mismatched.content_id(),
        mismatched.codec(),
        mismatched.encoded_bytes() - 4,
        mismatched.decoded_bytes() - 4,
        mismatched.alignment(),
        Arc::from(mismatched.platform_format()),
        mismatched.residency(),
        Vec::new(),
    );

    let result = RenderArtifactManifest::new(
        mesh,
        1,
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::mesh(mesh_test_layout("zr-static-mesh-v1", 1, 0)),
        Vec::new(),
        vec![mismatched],
    );

    assert!(matches!(
        result,
        Err(RenderArtifactManifestError::MeshLodDecodedSizeMismatch {
            lod: 0,
            expected: 4_096,
            actual: 4_092,
        })
    ));
}
