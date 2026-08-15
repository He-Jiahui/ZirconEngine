use crate::asset::{
    cook_mesh_sdf_from_mesh, cook_mesh_sdf_or_fallback, MeshSdfCookBudget, MeshSdfCookError,
    MeshSdfCookSettings, MeshVertex,
};
use crate::core::math::{Vec2, Vec3};

#[test]
fn cube_cook_is_deterministic_and_self_validating() {
    let (vertices, indices) = cube_geometry();
    let settings = MeshSdfCookSettings {
        max_dimension: 16,
        max_voxel_count: 4096,
        max_payload_bytes: 16 * 1024,
        surface_band_voxels: 4,
        two_sided: false,
    };

    let first = cook_mesh_sdf_from_mesh(&vertices, &indices, settings).unwrap();
    let second = cook_mesh_sdf_from_mesh(&vertices, &indices, settings).unwrap();

    assert_eq!(first, second);
    assert!(first.validate_for_source(&vertices, &indices).is_ok());
    assert!(first.voxels.iter().any(|distance| *distance < 0));
    assert!(first.voxels.iter().any(|distance| *distance > 0));
}

#[test]
fn selected_layout_stays_within_voxel_and_byte_budgets() {
    let (vertices, indices) = cube_geometry();
    let settings = MeshSdfCookSettings {
        max_dimension: 64,
        max_voxel_count: 1000,
        max_payload_bytes: 2300,
        surface_band_voxels: 3,
        two_sided: true,
    };

    let asset = cook_mesh_sdf_from_mesh(&vertices, &indices, settings).unwrap();

    assert!(asset.voxel_count().unwrap() <= settings.max_voxel_count);
    assert!(asset.encoded_size_bytes().unwrap() <= settings.max_payload_bytes);
    assert!(asset
        .dimensions
        .into_iter()
        .all(|dimension| dimension <= 64));
}

#[test]
fn source_hash_rejects_geometry_changes() {
    let (vertices, indices) = cube_geometry();
    let asset =
        cook_mesh_sdf_from_mesh(&vertices, &indices, MeshSdfCookSettings::default()).unwrap();
    let mut moved = vertices.clone();
    moved[0].position[0] -= 0.25;

    assert!(asset.validate_for_source(&moved, &indices).is_err());
}

#[test]
fn source_triangle_limit_rejects_before_bvh_allocation() {
    let (vertices, _) = cube_geometry();
    let triangle_count = super::budget::MAX_MESH_SDF_SOURCE_TRIANGLE_COUNT + 1;
    let indices = [0_u32, 1, 2]
        .into_iter()
        .cycle()
        .take(triangle_count as usize * 3)
        .collect::<Vec<_>>();

    assert!(matches!(
        cook_mesh_sdf_from_mesh(&vertices, &indices, MeshSdfCookSettings::default()),
        Err(MeshSdfCookError::SourceTriangleBudgetExceeded { actual, budget })
            if actual == triangle_count && budget == super::budget::MAX_MESH_SDF_SOURCE_TRIANGLE_COUNT
    ));
}

#[test]
fn cumulative_import_budget_rejects_before_unbounded_model_work() {
    let mut budget = MeshSdfCookBudget::default();
    budget
        .reserve(super::budget::MAX_MESH_SDF_IMPORT_VOXEL_COUNT, 1, 1)
        .unwrap();

    assert!(matches!(
        budget.reserve(1, 1, 1),
        Err(MeshSdfCookError::ImportVoxelBudgetExceeded { .. })
    ));
}

#[test]
fn budget_exhaustion_degrades_to_a_typed_missing_mesh_sdf() {
    let (vertices, indices) = cube_geometry();
    let mut budget = MeshSdfCookBudget::default();
    budget
        .reserve(super::budget::MAX_MESH_SDF_IMPORT_VOXEL_COUNT, 1, 1)
        .unwrap();

    let cooked = cook_mesh_sdf_or_fallback(
        &vertices,
        &indices,
        MeshSdfCookSettings::default(),
        &mut budget,
    )
    .unwrap();

    assert!(cooked.is_none());
}

fn cube_geometry() -> (Vec<MeshVertex>, Vec<u32>) {
    let positions = [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ];
    let vertices = positions
        .into_iter()
        .map(|position| MeshVertex::new(Vec3::from_array(position), Vec3::Z, Vec2::ZERO))
        .collect();
    let indices = vec![
        0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7, 6,
        3, 0, 4, 3, 4, 7,
    ];
    (vertices, indices)
}
