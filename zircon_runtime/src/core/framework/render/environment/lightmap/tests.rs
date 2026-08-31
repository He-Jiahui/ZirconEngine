use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

use super::*;

fn slot() -> LightmapInstanceSlot {
    LightmapInstanceSlot {
        atlas_page: 0,
        uv_rect: Vec4::new(0.5, 0.5, 0.25, 0.25),
    }
}

fn atlas_descriptor() -> LightmapAtlasDescriptor {
    LightmapAtlasDescriptor {
        page_size: 4,
        page_count: 1,
        format: LightmapAtlasFormat::Rgba16Float,
    }
}

fn lightmaps(generation: u64) -> LightmapConsumeContract {
    LightmapConsumeContract::new(
        generation,
        ResourceId::from_stable_label("res://lighting/test.lightmap-array"),
        atlas_descriptor(),
        vec![(7, slot())],
    )
}

fn bake_request() -> LightmapBakeRequest {
    LightmapBakeRequest {
        contract_version: LIGHTMAP_CONSUME_CONTRACT_VERSION,
        request_id: 41,
        scene_revision: 9,
        light_set_generation: 3,
        static_instance_ids: vec![7],
        scene_snapshot: LightmapBakeSceneSnapshot {
            format_version: LIGHTMAP_SCENE_SNAPSHOT_VERSION,
            content_hash: [7; 32],
            payload: vec![1, 2, 3, 4],
        },
        atlas_budget: LightmapAtlasBudget {
            page_size: 4,
            max_pages: 2,
        },
        texel_density: 16.0,
        probe_bounds_min: Vec3::splat(-1.0),
        probe_bounds_max: Vec3::splat(1.0),
        probe_cell_size: Vec3::ONE,
    }
}

fn bake_output() -> LightmapBakeOutput {
    LightmapBakeOutput {
        contract_version: LIGHTMAP_CONSUME_CONTRACT_VERSION,
        request_id: 41,
        scene_revision: 9,
        light_set_generation: 3,
        atlas: atlas_descriptor(),
        atlas_pages: vec![LightmapAtlasPage {
            page_index: 0,
            texels_rgba16f_le: vec![0; 4 * 4 * RGBA16F_TEXEL_SIZE_BYTES],
        }],
        slots: vec![(7, slot())],
        probe_grid: Some(probe_grid(3)),
    }
}

fn probe_grid(generation: u64) -> LightProbeGridData {
    LightProbeGridData {
        light_set_generation: generation,
        bounds_min: Vec3::splat(-1.0),
        cell_size: Vec3::ONE,
        dims: [2, 1, 1],
        sh: vec![ShL2Rgb::default(); 2],
    }
}

#[test]
fn render_env_lightmap_bake_dto_serde_roundtrip() {
    let request = bake_request();
    let output = bake_output();

    request.validate().expect("request fixture should be valid");
    output
        .validate_against(&request)
        .expect("fixture should belong to the request");
    let json = serde_json::to_string(&(request.clone(), output.clone()))
        .expect("contracts should serialize");
    let decoded: (LightmapBakeRequest, LightmapBakeOutput) =
        serde_json::from_str(&json).expect("contract should deserialize");

    assert_eq!(decoded, (request, output));
}

#[test]
fn bake_output_rejects_stale_scene_and_unrequested_instances() {
    let request = bake_request();
    let mut stale = bake_output();
    stale.scene_revision += 1;
    assert_eq!(
        stale.validate_against(&request),
        Err(LightmapContractValidationError::BakeRequestMismatch)
    );

    let mut unexpected = bake_output();
    unexpected.slots.push((99, slot()));
    assert_eq!(
        unexpected.validate_against(&request),
        Err(LightmapContractValidationError::UnexpectedBakedInstanceId { instance_id: 99 })
    );
}

#[test]
fn bake_output_imports_a_validated_rgba16f_array_contract() {
    let output = bake_output();
    let (contract, probe_grid) = output
        .into_consume_contract(ResourceId::from_stable_label(
            "res://lighting/imported.lightmap-array",
        ))
        .expect("valid bake output should become a consumption contract after import");

    assert_eq!(contract.atlas_descriptor, atlas_descriptor());
    assert_eq!(contract.slots(), &[(7, slot())]);
    assert!(probe_grid.is_some());
}

#[test]
fn render_env_lightmap_uv_rect_transform_roundtrip() {
    let slot = slot();
    let uv2 = crate::core::math::Vec2::new(0.25, 0.75);

    let atlas_uv = slot
        .transform_uv2(uv2)
        .expect("valid UV2 should transform into the atlas page");
    let restored = slot
        .inverse_transform_uv2(atlas_uv)
        .expect("atlas UV should invert to UV2");

    assert_eq!(atlas_uv, crate::core::math::Vec2::new(0.375, 0.625));
    assert!((restored - uv2).abs().max_element() <= f32::EPSILON);
}

#[test]
fn optimization_wave_20260824rs_runtime09f2_resolves_stable_instance_slot() {
    let contract = lightmaps(5);

    assert_eq!(contract.slot_for_instance(7), Some(slot()));
    assert_eq!(contract.slot_for_instance(8), None);
}

#[test]
fn optimization_wave_20260824rs_runtime09f2_slot_lookup_tracks_generation_replacement() {
    let replacement = LightmapInstanceSlot {
        atlas_page: 0,
        uv_rect: Vec4::new(0.25, 0.25, 0.5, 0.5),
    };
    let contract = LightmapConsumeContract::new(
        6,
        ResourceId::from_stable_label("res://lighting/replaced.lightmap-array"),
        atlas_descriptor(),
        vec![(9, replacement)],
    );

    assert_eq!(contract.slot_for_instance(7), None);
    assert_eq!(contract.slot_for_instance(9), Some(replacement));
}

#[test]
fn optimization_wave_20260824rs_runtime09f2_normalizes_slot_order_and_preserves_first_duplicate() {
    let first = slot();
    let replacement = LightmapInstanceSlot {
        atlas_page: 0,
        uv_rect: Vec4::new(0.25, 0.25, 0.5, 0.5),
    };
    let contract = LightmapConsumeContract::new(
        5,
        ResourceId::from_stable_label("res://lighting/sorted.lightmap-array"),
        atlas_descriptor(),
        vec![(9, replacement), (7, first), (7, replacement)],
    );

    assert_eq!(
        contract
            .slots()
            .iter()
            .map(|(instance_id, _)| *instance_id)
            .collect::<Vec<_>>(),
        vec![7, 7, 9]
    );
    assert_eq!(contract.slot_for_instance(7), Some(first));
}

#[test]
fn optimization_wave_20260824rs_runtime09f2_deserialization_normalizes_slot_order() {
    let contract = LightmapConsumeContract::new(
        5,
        ResourceId::from_stable_label("res://lighting/decoded.lightmap-array"),
        atlas_descriptor(),
        vec![(7, slot()), (9, slot())],
    );
    let mut encoded = serde_json::to_value(contract).expect("contract should serialize");
    encoded["slots"]
        .as_array_mut()
        .expect("slots should serialize as an array")
        .reverse();

    let decoded: LightmapConsumeContract =
        serde_json::from_value(encoded).expect("contract should deserialize");

    assert_eq!(
        decoded
            .slots()
            .iter()
            .map(|(instance_id, _)| *instance_id)
            .collect::<Vec<_>>(),
        vec![7, 9]
    );
}

#[test]
#[ignore = "performance evidence; run in the managed Windows release lane"]
fn optimization_wave_20260824rs_runtime09f2_lightmap_slot_lookup_evidence() {
    const SLOT_COUNT: u64 = 100_000;
    const QUERY_COUNT: u64 = 10_000;
    const MAX_ELAPSED: Duration = Duration::from_millis(500);

    let slots = (0..SLOT_COUNT)
        .rev()
        .map(|instance_id| (instance_id, slot()))
        .collect();
    let contract = LightmapConsumeContract::new(
        5,
        ResourceId::from_stable_label("res://lighting/lookup-bench.lightmap-array"),
        atlas_descriptor(),
        slots,
    );
    let started = Instant::now();
    for _ in 0..QUERY_COUNT {
        black_box(contract.slot_for_instance(black_box(SLOT_COUNT - 1)))
            .expect("last slot should resolve");
    }
    let elapsed = started.elapsed();

    let legacy_slot_comparisons = SLOT_COUNT * QUERY_COUNT;
    let comparisons_per_query_upper_bound = u64::from(u64::BITS - SLOT_COUNT.leading_zeros());
    let indexed_comparisons_upper_bound = comparisons_per_query_upper_bound * QUERY_COUNT;
    let reduction_basis_points = (legacy_slot_comparisons - indexed_comparisons_upper_bound)
        * 10_000
        / legacy_slot_comparisons;
    assert!(elapsed <= MAX_ELAPSED, "indexed lookup took {elapsed:?}");
    println!(
        "RUNTIME09F2_LIGHTMAP_SLOT_BENCH_V1 slots={} queries={} legacy_slot_comparisons={} indexed_comparisons_upper_bound={} reduction_basis_points={} elapsed_ns={}",
        SLOT_COUNT,
        QUERY_COUNT,
        legacy_slot_comparisons,
        indexed_comparisons_upper_bound,
        reduction_basis_points,
        elapsed.as_nanos()
    );
}

#[test]
fn optimization_wave_20260824rs_runtime09f2_serde_roundtrip_preserves_slots() {
    let contract = LightmapConsumeContract::new(
        5,
        ResourceId::from_stable_label("res://lighting/test.lightmap-array"),
        atlas_descriptor(),
        vec![
            (7, slot()),
            (
                9,
                LightmapInstanceSlot {
                    atlas_page: 0,
                    uv_rect: Vec4::new(0.25, 0.25, 0.5, 0.5),
                },
            ),
        ],
    );
    let json = serde_json::to_string(&contract).expect("contract should serialize");
    let decoded: LightmapConsumeContract =
        serde_json::from_str(&json).expect("contract should deserialize");

    assert_eq!(decoded, contract);
    decoded
        .validate()
        .expect("deserialized contract should preserve a valid slot contract");
    assert_eq!(decoded.slots(), contract.slots());
    assert_eq!(decoded.slot_for_instance(7), Some(slot()));
    assert_eq!(
        decoded.slot_for_instance(9),
        Some(LightmapInstanceSlot {
            atlas_page: 0,
            uv_rect: Vec4::new(0.25, 0.25, 0.5, 0.5),
        })
    );
}

#[test]
fn optimization_wave_20260824rs_runtime09f2_rejects_duplicate_instances_and_out_of_bounds_uvs() {
    let duplicate = LightmapConsumeContract::new(
        1,
        ResourceId::from_stable_label("res://lighting/test.lightmap-array"),
        atlas_descriptor(),
        vec![(7, slot()), (7, slot())],
    );
    assert_eq!(
        duplicate.validate(),
        Err(LightmapContractValidationError::DuplicateInstanceId { instance_id: 7 })
    );
    assert_eq!(duplicate.slot_for_instance(7), Some(slot()));

    let invalid_slot = LightmapInstanceSlot {
        atlas_page: 0,
        uv_rect: Vec4::new(0.75, 0.75, 0.5, 0.5),
    };
    assert_eq!(
        invalid_slot.validate(),
        Err(LightmapContractValidationError::InvalidUvRect)
    );
}

#[test]
fn probe_grid_requires_exact_finite_sh9_population() {
    let mut grid = probe_grid(1);
    grid.sh.pop();
    assert_eq!(
        grid.validate(),
        Err(LightmapContractValidationError::ProbeCoefficientCount {
            expected: 2,
            actual: 1,
        })
    );

    let mut grid = probe_grid(1);
    grid.sh[1].0[0].x = f32::NAN;
    assert_eq!(
        grid.validate(),
        Err(LightmapContractValidationError::NonFiniteProbeCoefficient)
    );
}

#[test]
fn render_env_probe_grid_trilinear_center_equals_cell_average() {
    let mut grid = LightProbeGridData {
        light_set_generation: 1,
        bounds_min: Vec3::ZERO,
        cell_size: Vec3::ONE,
        dims: [2, 2, 2],
        sh: Vec::new(),
    };
    for value in 0..8 {
        let mut sh = ShL2Rgb::default();
        sh.0[0] = Vec3::splat(value as f32);
        grid.sh.push(sh);
    }

    let sample = grid
        .sample_trilinear(Vec3::splat(0.5))
        .expect("cell center should sample all eight probes");

    assert_eq!(sample.0[0], Vec3::splat(3.5));
}

#[test]
fn bake_output_rejects_mixed_light_set_generations() {
    let mut output = bake_output();
    output.probe_grid = Some(probe_grid(2));

    assert_eq!(
        output.validate(),
        Err(LightmapContractValidationError::GenerationMismatch)
    );
}
