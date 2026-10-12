use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const MATERIALS_PER_SAMPLE: usize = 2_048;
const UNIQUE_TEXTURES: usize = 32;
const TEXTURE_REFERENCES: usize = 37;

#[test]
fn gltf_material_dependency_projection_is_unique_and_shader_first() {
    let (asset, expected) = material_dependency_fixture(8);

    let dependencies = material_dependencies(&asset);

    assert_eq!(dependencies, expected);
    assert_eq!(dependencies.first(), Some(&asset.shader.locator));
}

#[test]
fn gltf_normal_and_data_slots_reference_their_own_derived_texture_variants() {
    let root_uri = AssetUri::parse("res://models/shared_linear_texture.glb")
        .expect("fixture root URI must be valid");
    let gltf = gltf::Gltf::from_slice(
        br#"{
                "asset": { "version": "2.0" },
                "textures": [{ "source": 0 }],
                "materials": [{
                    "normalTexture": { "index": 0 },
                    "pbrMetallicRoughness": { "metallicRoughnessTexture": { "index": 0 } }
                }]
            }"#,
    )
    .expect("normal/data texture fixture must parse");
    let texture_usages = gltf_texture_color_space_usages(&gltf.document);

    let normal = texture_reference(
        &root_uri,
        0,
        GltfTextureColorSpace::Linear,
        TextureUsageHint::Normal,
        &texture_usages,
    );
    let data = texture_reference(
        &root_uri,
        0,
        GltfTextureColorSpace::Linear,
        TextureUsageHint::Data,
        &texture_usages,
    );

    assert_eq!(normal.locator, gltf_label_uri(&root_uri, "Texture0/Normal"));
    assert_eq!(data.locator, gltf_label_uri(&root_uri, "Texture0/Data"));
    assert_ne!(normal, data);
}

#[test]
fn gltf_material_projection_preserves_explicit_zero_roughness_factor() {
    let gltf = gltf::Gltf::from_slice(
        br#"{
                "asset": { "version": "2.0" },
                "materials": [{
                    "pbrMetallicRoughness": {
                        "metallicFactor": 1.0,
                        "roughnessFactor": 0.0
                    }
                }]
            }"#,
    )
    .expect("minimal glTF material must parse");
    let root_uri = AssetUri::parse("res://models/explicit_zero_roughness.gltf")
        .expect("fixture root URI must be valid");
    let material_uri = gltf_label_uri(&root_uri, "Material0");
    let material = gltf
        .document
        .materials()
        .next()
        .expect("fixture must contain one material");

    let asset = material_asset_from_gltf_material(&root_uri, material_uri, &material, &[]);

    assert_eq!(asset.roughness, 0.0);
    assert_eq!(asset.standard_material_descriptor().roughness, 0.0);
}

#[test]
fn gltf_default_and_explicit_materials_share_the_compound_default_pbr_reference() {
    let gltf = gltf::Gltf::from_slice(
        br#"{
                "asset": { "version": "2.0" },
                "materials": [{}]
            }"#,
    )
    .expect("minimal glTF material must parse");
    let root_uri = AssetUri::parse("res://models/default_pbr_reference.gltf")
        .expect("fixture root URI must be valid");
    let material = gltf
        .document
        .materials()
        .next()
        .expect("fixture must contain one material");
    let expected = default_pbr_shader_reference();

    let default_material = default_material_asset(gltf_label_uri(&root_uri, "DefaultMaterial"));
    let explicit_material = material_asset_from_gltf_material(
        &root_uri,
        gltf_label_uri(&root_uri, "Material0"),
        &material,
        &[],
    );

    assert_eq!(default_material.shader, expected);
    assert_eq!(explicit_material.shader, expected);
}

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_borrowed_gltf_material_dependency_projection() {
    let (asset, expected) = material_dependency_fixture(UNIQUE_TEXTURES);
    assert_eq!(material_dependencies(&asset), expected);
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_dependency_projection(
                legacy_material_dependencies,
                &asset,
            ));
            optimized_raw.push(measure_dependency_projection(material_dependencies, &asset));
        } else {
            optimized_raw.push(measure_dependency_projection(material_dependencies, &asset));
            legacy_raw.push(measure_dependency_projection(
                legacy_material_dependencies,
                &asset,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
        "borrowed glTF material dependency projection must improve P95 by at least 25%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "PERF_RESULT task=plugins07_borrowed_gltf_material_dependencies sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank materials_per_sample={MATERIALS_PER_SAMPLE} unique_textures={UNIQUE_TEXTURES} texture_references={TEXTURE_REFERENCES} legacy_temporary_slot_vec_allocations_per_material=2 optimized_temporary_slot_vec_allocations_per_material=0 legacy_temporary_slot_name_allocations_per_material={TEXTURE_REFERENCES} optimized_temporary_slot_name_allocations_per_material=0 legacy_locator_clones_per_material=71 optimized_locator_clones_per_material=33 threshold_percent=25 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

fn material_dependency_fixture(unique_textures: usize) -> (MaterialAsset, Vec<AssetUri>) {
    assert!(unique_textures >= 5);
    let mut asset =
        default_material_asset(AssetUri::parse("res://materials/dependency_fixture").unwrap());
    let references = (0..unique_textures)
        .map(|index| {
            AssetReference::from_locator(
                AssetUri::parse(&format!("res://textures/dependency_{index}")).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    asset.base_color_texture = Some(references[0].clone());
    asset.normal_texture = Some(references[1].clone());
    asset.metallic_roughness_texture = Some(references[2].clone());
    asset.occlusion_texture = Some(references[3].clone());
    asset.emissive_texture = Some(references[4].clone());
    for (index, reference) in references.iter().enumerate() {
        asset.texture_slots.insert(
            format!("custom_{index:02}"),
            MaterialTextureSlotValue::new(reference.clone()),
        );
    }
    let expected = std::iter::once(asset.shader.locator.clone())
        .chain(references.into_iter().map(|reference| reference.locator))
        .collect();
    (asset, expected)
}

fn legacy_material_dependencies(asset: &MaterialAsset) -> Vec<AssetUri> {
    let mut dependencies = vec![asset.shader.locator.clone()];
    let mut dependency_index = HashSet::from([asset.shader.locator.clone()]);
    for reference in black_box(asset.all_texture_slots())
        .into_iter()
        .map(|(_, reference)| reference)
    {
        if dependency_index.insert(reference.locator.clone()) {
            dependencies.push(reference.locator.clone());
        }
    }
    black_box(dependencies)
}

fn measure_dependency_projection(
    projection: fn(&MaterialAsset) -> Vec<AssetUri>,
    asset: &MaterialAsset,
) -> u64 {
    let started = Instant::now();
    let mut dependency_count = 0;
    for _ in 0..MATERIALS_PER_SAMPLE {
        dependency_count += black_box(projection(black_box(asset))).len();
    }
    black_box(dependency_count);
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u64]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
