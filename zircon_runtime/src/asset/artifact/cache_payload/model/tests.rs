use super::*;
use crate::asset::artifact::cache_payload::{from_imported_for_write, ArtifactCacheAsset};
use crate::asset::ImportedAsset;
use crate::core::framework::render::RenderMeshBounds;
use crate::core::math::{Vec2, Vec3};
use zircon_runtime_interface::serialization::PositionalEncoding;

#[test]
fn borrowed_model_encoding_matches_owned_wire_and_reads_existing_bytes() {
    let encoding = PositionalEncoding::FixedLittleEndian;
    let mut geometry = VirtualGeometryAsset::default();
    geometry.cluster_page_data = vec![vec![17; 4096]];
    geometry.cluster_page_headers = vec![crate::asset::VirtualGeometryClusterPageHeaderAsset {
        page_id: 0,
        start_offset: 0,
        payload_size_bytes: 4096,
    }];
    for primitives in [
        Vec::new(),
        vec![ModelPrimitiveAsset {
            vertices: vec![MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO); 1024],
            indices: vec![0, 1, 2],
            mesh: None,
            mesh_sdf: Some(MeshSdfAsset {
                schema_version: 1,
                source_hash: [7; 32],
                local_bounds: RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
                dimensions: [4; 3],
                voxel_size: [0.5; 3],
                distance_range: [-1.0, 1.0],
                encoding: Default::default(),
                cook_settings: Default::default(),
                voxels: vec![3; 64],
            }),
            virtual_geometry: Some(geometry),
        }],
        vec![ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh: Some(AssetReference::from_locator(
                AssetUri::parse("res://memory/shared.mesh").unwrap(),
            )),
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    ] {
        let asset = ImportedAsset::Model(ModelAsset {
            uri: AssetUri::parse("res://memory/model.obj").unwrap(),
            primitives,
        });
        let old = ArtifactCacheAsset::from_imported(&asset).unwrap();
        let view = from_imported_for_write(&asset).unwrap();
        let expected = encoding.encode(&old).unwrap();
        let actual = encoding.encode(&view).unwrap();
        assert_eq!(actual, expected);
        let decoded: ArtifactCacheAsset = encoding.decode(&actual).unwrap();
        assert_eq!(decoded, old);
        let ArtifactCacheAsset::Model(wire) = decoded else {
            panic!("expected a model DTO");
        };
        let buffers = wire
            .primitives
            .iter()
            .map(|primitive| {
                (
                    primitive.vertices.as_ptr(),
                    primitive.indices.as_ptr(),
                    primitive.mesh_sdf.as_ref().map(|sdf| sdf.voxels.as_ptr()),
                    primitive
                        .virtual_geometry
                        .as_ref()
                        .and_then(|geometry| geometry.cluster_page_data.first())
                        .map(|page| page.as_ptr()),
                )
            })
            .collect::<Vec<_>>();
        let model = wire.into_asset();
        for (primitive, (vertices, indices, sdf, page)) in model.primitives.iter().zip(buffers) {
            assert_eq!(primitive.vertices.as_ptr(), vertices);
            assert_eq!(primitive.indices.as_ptr(), indices);
            assert_eq!(
                primitive.mesh_sdf.as_ref().map(|sdf| sdf.voxels.as_ptr()),
                sdf
            );
            assert_eq!(
                primitive
                    .virtual_geometry
                    .as_ref()
                    .and_then(|geometry| geometry.cluster_page_data.first())
                    .map(|page| page.as_ptr()),
                page
            );
        }
        assert_eq!(ImportedAsset::Model(model), asset);
    }
}
