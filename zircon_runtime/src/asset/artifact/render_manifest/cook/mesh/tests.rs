use std::collections::BTreeMap;
use std::sync::Arc;

use crate::asset::{AssetUri, MeshAsset, MeshAttributeValues, MeshIndices};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use super::*;

fn resource() -> UntypedResourceHandle {
    UntypedResourceHandle::new(
        ResourceId::from_stable_label("render-mesh-cook/mesh"),
        ResourceKind::Mesh,
    )
}

fn settings() -> RenderArtifactMeshCookSettings {
    RenderArtifactMeshCookSettings::new(Arc::from("windows-dx12-sm6"), 256)
}

fn mesh(
    topology: RenderMeshTopology,
    positions: Vec<[f32; 3]>,
    indices: Option<MeshIndices>,
) -> MeshAsset {
    let mut attributes = BTreeMap::new();
    attributes.insert(
        crate::asset::MESH_ATTRIBUTE_POSITION.to_owned(),
        MeshAttributeValues::Float32x3(positions),
    );
    MeshAsset::new(
        AssetUri::parse("res://meshes/cooked.zrmesh")
            .unwrap_or_else(|error| panic!("mesh cook URI failed: {error}")),
        topology,
        attributes,
        indices,
    )
    .unwrap_or_else(|error| panic!("mesh cook fixture failed: {error}"))
}

#[test]
fn render_mesh_cook_packs_final_static_vertex_bytes_and_converts_u16_indices_once() {
    let mesh = mesh(
        RenderMeshTopology::TriangleList,
        vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
        Some(MeshIndices::U16(vec![2, 0, 1])),
    );

    let output = cook_mesh_render_artifact(resource(), 3, mesh, settings())
        .unwrap_or_else(|error| panic!("mesh cook failed: {error}"));
    let layout = output
        .manifest()
        .mesh_lod_layout(crate::asset::artifact::RenderSubresourceId::MeshLod { lod: 0 })
        .unwrap_or_else(|| panic!("cooked mesh LOD layout missing"));
    let bytes = output.blocks()[0].bytes();

    assert_eq!(layout.vertex_range(), 0..288);
    assert_eq!(layout.index_range(), 288..300);
    assert_eq!(bytes.len(), 300);
    assert_eq!(&bytes[0..4], &1.0_f32.to_le_bytes());
    assert_eq!(&bytes[4..8], &2.0_f32.to_le_bytes());
    assert_eq!(&bytes[8..12], &3.0_f32.to_le_bytes());
    assert_eq!(&bytes[20..24], &1.0_f32.to_le_bytes());
    assert_eq!(&bytes[56..60], &1.0_f32.to_le_bytes());
    assert_eq!(&bytes[68..72], &1.0_f32.to_le_bytes());
    assert_eq!(&bytes[72..76], &1.0_f32.to_le_bytes());
    assert_eq!(&bytes[84..88], &1.0_f32.to_le_bytes());
    assert_eq!(&bytes[288..292], &2_u32.to_le_bytes());
    assert_eq!(&bytes[292..296], &0_u32.to_le_bytes());
    assert_eq!(&bytes[296..300], &1_u32.to_le_bytes());
}

#[test]
fn render_mesh_cook_materializes_missing_or_empty_triangle_indices_in_the_final_block() {
    for indices in [None, Some(MeshIndices::U32(Vec::new()))] {
        let mesh = mesh(
            RenderMeshTopology::TriangleList,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices,
        );

        let output = cook_mesh_render_artifact(resource(), 4, mesh, settings())
            .unwrap_or_else(|error| panic!("implicit-index mesh cook failed: {error}"));
        let bytes = output.blocks()[0].bytes();

        assert_eq!(&bytes[288..292], &0_u32.to_le_bytes());
        assert_eq!(&bytes[292..296], &1_u32.to_le_bytes());
        assert_eq!(&bytes[296..300], &2_u32.to_le_bytes());
    }
}

#[test]
fn render_mesh_cook_rejects_topology_not_supported_by_the_current_gpu_pipeline() {
    let mesh = mesh(
        RenderMeshTopology::LineList,
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        Some(MeshIndices::U32(vec![0, 1])),
    );

    assert!(matches!(
        cook_mesh_render_artifact(resource(), 5, mesh, settings()),
        Err(RenderArtifactMeshCookError::UnsupportedTopology {
            topology: RenderMeshTopology::LineList,
        })
    ));
}
