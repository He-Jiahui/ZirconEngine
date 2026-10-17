use super::*;
use crate::asset::artifact::cache_payload::{from_imported_for_write, ArtifactCacheAsset};
use crate::asset::ImportedAsset;
use crate::core::framework::render::RenderMeshBounds;
use crate::core::math::{Vec2, Vec3};
use crate::memory_validation::{live_bytes, measure};
use std::io::{self, Write};
use zircon_runtime_interface::serialization::PositionalEncoding;
#[derive(Default)]
struct DigestSink {
    hash: blake3::Hasher,
    bytes: usize,
}
impl Write for DigestSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.hash.update(bytes);
        self.bytes += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn stream<T: Serialize>(value: &T) -> (usize, blake3::Hash) {
    let mut encoder = zstd::stream::Encoder::new(DigestSink::default(), 3).unwrap();
    PositionalEncoding::FixedLittleEndian
        .encode_into(&mut encoder, value, u64::MAX)
        .unwrap();
    let sink = encoder.finish().unwrap();
    (sink.bytes, sink.hash.finalize())
}
#[test]
fn memory_profile_model_stream() {
    let mut geometry = VirtualGeometryAsset::default();
    geometry.cluster_page_data = vec![vec![17; 8 * 1024 * 1024]];
    let model = ImportedAsset::Model(ModelAsset {
        uri: AssetUri::parse("res://memory/large.model").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: vec![MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO); 512 * 1024],
            indices: vec![0; 1024 * 1024],
            mesh: None,
            mesh_sdf: Some(MeshSdfAsset {
                schema_version: 1,
                source_hash: [7; 32],
                local_bounds: RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
                dimensions: [256; 3],
                voxel_size: [1.0 / 128.0; 3],
                distance_range: [-1.0, 1.0],
                encoding: Default::default(),
                cook_settings: Default::default(),
                voxels: vec![3; 256 * 256 * 256],
            }),
            virtual_geometry: Some(geometry),
        }],
    });
    // Warm encoder/serializer paths before each measured interval.
    stream(&from_imported_for_write(&model).unwrap());
    let baseline = live_bytes();
    let ((owned, old_digest), old) = measure(|| {
        let dto = ArtifactCacheAsset::from_imported(&model).unwrap();
        let digest = stream(&dto);
        (dto, digest)
    });
    old.report(
        "large_model_vertices_indices_sdf_virtual_geometry_stream",
        "owned_model_baseline",
    );
    drop(owned);
    assert_eq!(live_bytes(), baseline);
    let (new_digest, new) = measure(|| stream(&from_imported_for_write(&model).unwrap()));
    assert_eq!(old_digest, new_digest);
    assert!(new.peak_bytes < old.peak_bytes / 4);
    assert_eq!(new.live_bytes, 0);
    new.report(
        "large_model_vertices_indices_sdf_virtual_geometry_stream",
        "borrowed_model_view",
    );
    assert_eq!(live_bytes(), baseline);
}
