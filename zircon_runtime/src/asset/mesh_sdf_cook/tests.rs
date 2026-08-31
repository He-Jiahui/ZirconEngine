use crate::asset::{
    cook_mesh_sdf_from_mesh, cook_mesh_sdf_from_mesh_with_executor, cook_mesh_sdf_or_fallback,
    MeshSdfCookBudget, MeshSdfCookError, MeshSdfCookSettings, MeshVertex,
};
use crate::core::math::{Vec2, Vec3};
use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
use std::hint::black_box;
use std::time::Instant;

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
fn explicit_executor_cook_matches_the_serial_payload() {
    let (vertices, indices) = cube_geometry();
    let settings = MeshSdfCookSettings {
        max_dimension: 16,
        max_voxel_count: 4096,
        max_payload_bytes: 16 * 1024,
        surface_band_voxels: 4,
        two_sided: false,
    };
    let serial = cook_mesh_sdf_from_mesh(&vertices, &indices, settings)
        .expect("serial Mesh SDF cook should succeed");
    let executor = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let parallel = cook_mesh_sdf_from_mesh_with_executor(&executor, &vertices, &indices, settings)
        .expect("explicit executor Mesh SDF cook should succeed");

    assert_eq!(parallel, serial);
}

#[test]
fn runtime02_mesh_sdf_default_path_has_no_ambient_rayon_and_executor_path_is_explicit() {
    let source = include_str!("cook.rs");

    assert!(!source.contains("rayon::prelude"));
    assert!(source.contains("cook_mesh_sdf_from_mesh_with_executor"));
    assert!(source.contains("executor.parallel_map_indices"));
    assert!(source.contains("default cook entry points remain serial"));
}

#[test]
fn runtime02_mesh_sdf_bvh_build_reserves_filtered_triangle_storage() {
    let source = include_str!("acceleration.rs");

    assert!(source.contains("let triangle_capacity = indices.len() / 3;"));
    assert!(source.contains("Vec::with_capacity(triangle_capacity)"));
    assert!(source.contains("Vec::with_capacity(triangles.len())"));
}

#[test]
#[ignore = "managed Runtime02 Mesh SDF executor performance evidence"]
fn runtime02_mesh_sdf_explicit_executor_performance_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const WARMUP_ROUNDS: usize = 2;
    const MAX_EXECUTOR_P95_MULTIPLIER: u128 = 2;

    let (vertices, indices) = cube_geometry();
    let settings = MeshSdfCookSettings {
        max_dimension: 64,
        max_voxel_count: 262_144,
        max_payload_bytes: 1_000_000,
        surface_band_voxels: 4,
        two_sided: false,
    };
    let executor = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));

    for _ in 0..WARMUP_ROUNDS {
        black_box(
            cook_mesh_sdf_from_mesh(&vertices, &indices, settings)
                .expect("serial Mesh SDF warmup should succeed"),
        );
        black_box(
            cook_mesh_sdf_from_mesh_with_executor(&executor, &vertices, &indices, settings)
                .expect("executor Mesh SDF warmup should succeed"),
        );
    }

    let mut serial_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut executor_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (serial_ns, executor_ns) = if pair_index % 2 == 0 {
            (
                measure_serial(&vertices, &indices, settings),
                measure_executor(&executor, &vertices, &indices, settings),
            )
        } else {
            let executor_ns = measure_executor(&executor, &vertices, &indices, settings);
            (measure_serial(&vertices, &indices, settings), executor_ns)
        };
        serial_samples.push(serial_ns);
        executor_samples.push(executor_ns);
    }

    let serial_p95 = nearest_rank_p95(&serial_samples);
    let executor_p95 = nearest_rank_p95(&executor_samples);
    assert!(
        executor_p95 <= serial_p95.saturating_mul(MAX_EXECUTOR_P95_MULTIPLIER),
        "explicit executor p95 regressed beyond the bounded comparison guard"
    );
    println!(
        "RUNTIME02_MESH_SDF_EXECUTOR_BENCH_V1 sample_pairs={SAMPLE_PAIRS} warmup_rounds={WARMUP_ROUNDS} voxel_budget={} serial_p95_ns={serial_p95} executor_p95_ns={executor_p95} serial_samples={} executor_samples={} ambient_rayon_imports_before=1 ambient_rayon_imports_after=0 executor_capability=caller_owned order=alternating_serial_first_even",
        settings.max_voxel_count,
        csv(&serial_samples),
        csv(&executor_samples),
    );
}

fn measure_serial(vertices: &[MeshVertex], indices: &[u32], settings: MeshSdfCookSettings) -> u128 {
    let started = Instant::now();
    black_box(
        cook_mesh_sdf_from_mesh(vertices, indices, settings)
            .expect("serial Mesh SDF sample should succeed"),
    );
    started.elapsed().as_nanos().max(1)
}

fn measure_executor(
    executor: &TaskPool,
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
) -> u128 {
    let started = Instant::now();
    black_box(
        cook_mesh_sdf_from_mesh_with_executor(executor, vertices, indices, settings)
            .expect("executor Mesh SDF sample should succeed"),
    );
    started.elapsed().as_nanos().max(1)
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
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
