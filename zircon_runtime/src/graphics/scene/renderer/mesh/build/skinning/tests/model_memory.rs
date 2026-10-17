use super::*;
use crate::asset::{MeshSdfAsset, MeshVertex, VirtualGeometryAsset};
use crate::core::framework::render::RenderMeshBounds;
use crate::core::math::Vec2;
use crate::memory_validation::{live_bytes, measure};

#[test]
fn memory_profile_borrowed_model_skinning() {
    let mut geometry = VirtualGeometryAsset::default();
    geometry.cluster_page_data = vec![vec![17; 8 * 1024 * 1024]];
    let mut vertex = MeshVertex::new(Vec3::X, Vec3::Z, Vec2::ZERO);
    vertex.joint_weights = [1.0, 0.0, 0.0, 0.0];
    let source = ModelPrimitiveAsset {
        vertices: vec![vertex; 65_536],
        indices: vec![0; 131_072],
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
    };
    let palette = SkinnedMeshJointPalette {
        joint_matrices: vec![Mat4::IDENTITY],
        storage: Some(Arc::new(
            SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY]).unwrap(),
        )),
    };
    let baseline = live_bytes();
    let (legacy, old) = measure(|| prepare_skinned_model_primitive_with_palette(&source, &palette));
    old.report(
        "model_skinning_with_sdf_and_virtual_geometry",
        "unused_source_clone_baseline",
    );
    let (shared, new) = measure(|| Arc::new(skin_model_primitive_with_palette(&source, &palette)));
    new.report(
        "model_skinning_with_sdf_and_virtual_geometry",
        "borrowed_source_shared_fallback",
    );
    assert_eq!(legacy.primitive.vertices, shared.vertices);
    assert_eq!(legacy.primitive.indices, shared.indices);
    assert_eq!(shared.vertices, source.vertices);
    assert!(shared.mesh_sdf.is_none());
    assert!(shared.virtual_geometry.is_none());
    assert_eq!(
        source.mesh_sdf.as_ref().unwrap().voxels.len(),
        256 * 256 * 256
    );
    assert_eq!(
        source.virtual_geometry.as_ref().unwrap().cluster_page_data[0].len(),
        8 * 1024 * 1024
    );
    assert!(new.live_bytes < old.live_bytes / 4);
    drop((legacy, shared));
    assert_eq!(live_bytes(), baseline);
}
