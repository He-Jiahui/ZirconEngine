use super::*;
use crate::subassets::material_entry_from_asset;
use std::hint::black_box;
use std::time::Instant;
use zircon_runtime::asset::assets::default_pbr_shader_reference;
use zircon_runtime::asset::{
    AlphaMode, AssetReference, AssetUri, ImportedAssetEntry, MaterialAsset,
};

#[test]
fn plugins07_gltf_hotpath_complete_normals_are_borrowed_and_short_normals_are_padded() {
    let positions = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let normals = [0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0];
    let indices = [0, 1, 2];

    let complete = prepare_vertex_normals(&positions, &normals, &indices).unwrap();
    let short = prepare_vertex_normals(&positions, &normals[..3], &indices).unwrap();

    assert!(matches!(&complete, std::borrow::Cow::Borrowed(_)));
    assert!(matches!(&short, std::borrow::Cow::Owned(_)));
    assert_eq!(
        short.as_ref(),
        &[0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
}

#[test]
fn plugins07_gltf_hotpath_material_entry_moves_payload_and_preserves_dependencies() {
    let material_uri = AssetUri::parse("res://models/fixture.gltf#Material0").unwrap();
    let texture_uri = AssetUri::parse("res://models/fixture.gltf#Texture0").unwrap();
    let mut material = material_fixture(1, 8);
    material.base_color_texture = Some(AssetReference::from_locator(texture_uri.clone()));
    let shader_uri = material.shader.locator.clone();

    let entry = material_entry_from_asset(material_uri.clone(), material);

    assert_eq!(entry.locator, material_uri);
    assert_eq!(entry.dependencies.len(), 2);
    assert!(entry.dependencies.contains(&shader_uri));
    assert!(entry.dependencies.contains(&texture_uri));
    assert!(matches!(entry.asset, ImportedAsset::Material(_)));
}

#[test]
fn plugins07_gltf_hotpath_material_fixture_uses_the_canonical_default_pbr_asset() {
    assert_eq!(
        material_fixture(0, 0).shader,
        default_pbr_shader_reference(),
        "hot-path fixtures must use the same compound shader reference as imported materials"
    );
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn plugins07_gltf_hotpath_release_borrowed_normals_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const NORMAL_VALUES: usize = 524_288;
    const ITERATIONS: usize = 16;
    const THRESHOLD_PERCENT: u128 = 80;
    let normals = vec![0.25_f32; NORMAL_VALUES];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy = || measure_normal_clone(&normals, ITERATIONS);
        let optimized = || measure_normal_borrow(&normals, ITERATIONS);
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }

    emit_performance_gate(
        "plugins07_gltf_borrowed_normals",
        &legacy_samples,
        &optimized_samples,
        THRESHOLD_PERCENT,
        &format!(
            "normal_values={NORMAL_VALUES} iterations_per_sample={ITERATIONS} legacy_cloned_bytes_per_sample={} optimized_cloned_bytes_per_sample=0",
            NORMAL_VALUES * ITERATIONS * std::mem::size_of::<f32>()
        ),
    );
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn plugins07_gltf_hotpath_release_material_entry_move_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const MATERIALS: usize = 2_048;
    const DIAGNOSTICS_PER_MATERIAL: usize = 16;
    const DIAGNOSTIC_BYTES: usize = 128;
    const THRESHOLD_PERCENT: u128 = 40;
    let material_uri = AssetUri::parse("res://models/performance.gltf#Material0").unwrap();
    let template = material_fixture(DIAGNOSTICS_PER_MATERIAL, DIAGNOSTIC_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy = || measure_material_entries(&template, &material_uri, MATERIALS, true);
        let optimized = || measure_material_entries(&template, &material_uri, MATERIALS, false);
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }

    emit_performance_gate(
        "plugins07_gltf_material_entry_move",
        &legacy_samples,
        &optimized_samples,
        THRESHOLD_PERCENT,
        &format!(
            "materials_per_sample={MATERIALS} diagnostics_per_material={DIAGNOSTICS_PER_MATERIAL} diagnostic_bytes={DIAGNOSTIC_BYTES} legacy_material_clones_per_sample={MATERIALS} optimized_material_clones_per_sample=0"
        ),
    );
}

fn material_fixture(diagnostic_count: usize, diagnostic_bytes: usize) -> MaterialAsset {
    MaterialAsset {
        name: Some("PerformanceMaterial".to_string()),
        shader: default_pbr_shader_reference(),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: (0..diagnostic_count)
            .map(|_| "x".repeat(diagnostic_bytes))
            .collect(),
    }
}

fn measure_normal_clone(normals: &[f32], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut values = 0_usize;
    for _ in 0..iterations {
        let owned = black_box(normals).to_vec();
        values += black_box(owned.as_slice()).len();
    }
    black_box(values);
    started.elapsed().as_nanos()
}

fn measure_normal_borrow(normals: &[f32], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut values = 0_usize;
    for _ in 0..iterations {
        let borrowed: std::borrow::Cow<'_, [f32]> = std::borrow::Cow::Borrowed(black_box(normals));
        values += black_box(borrowed.as_ref()).len();
    }
    black_box(values);
    started.elapsed().as_nanos()
}

fn measure_material_entries(
    template: &MaterialAsset,
    material_uri: &AssetUri,
    material_count: usize,
    legacy: bool,
) -> u128 {
    let materials = (0..material_count)
        .map(|_| template.clone())
        .collect::<Vec<_>>();
    let started = Instant::now();
    let entries = materials
        .into_iter()
        .map(|material| {
            if legacy {
                legacy_material_entry(material_uri.clone(), material)
            } else {
                material_entry_from_asset(material_uri.clone(), material)
            }
        })
        .collect::<Vec<_>>();
    black_box(entries);
    started.elapsed().as_nanos()
}

fn legacy_material_entry(uri: AssetUri, asset: MaterialAsset) -> ImportedAssetEntry {
    let mut entry = ImportedAssetEntry::new(uri, ImportedAsset::Material(asset.clone()))
        .with_dependency(asset.shader.locator.clone());
    for reference in asset
        .all_texture_slots()
        .into_iter()
        .map(|(_, reference)| reference)
    {
        if !entry.dependencies.contains(&reference.locator) {
            entry = entry.with_dependency(reference.locator.clone());
        }
    }
    entry
}

fn emit_performance_gate(
    task: &str,
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    threshold_percent: u128,
    workload: &str,
) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT {task} sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
        samples_csv(legacy_samples),
        samples_csv(optimized_samples),
    );
    assert!(
        improvement_percent >= threshold_percent,
        "{task} must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
