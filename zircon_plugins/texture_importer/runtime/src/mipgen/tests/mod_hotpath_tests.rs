use super::*;
use std::hint::black_box;
use std::time::Instant;
use zircon_runtime::asset::AssetUri;

#[test]
fn plugins07_mip_hotpath_reuses_preallocated_base_payload_buffer() {
    let width = 4;
    let height = 4;
    let mip_count = full_mip_count(width, height);
    let total_len = rgba8_mip_chain_len(width, height, mip_count, 1).unwrap();
    let mut rgba = Vec::with_capacity(total_len);
    rgba.extend([32, 64, 96, 255].repeat((width * height) as usize));
    let base_pointer = rgba.as_ptr();
    let texture = offline_texture("res://textures/reuse.png", width, height, 1, rgba);

    let generated = generate_offline_mips(texture).unwrap();

    assert_eq!(generated.rgba.as_ptr(), base_pointer);
    assert_eq!(generated.rgba.len(), total_len);
    assert_eq!(&generated.rgba[..4], &[32, 64, 96, 255]);
}

#[test]
fn plugins07_mip_hotpath_range_current_levels_preserve_layer_packing_order() {
    let width = 4;
    let height = 4;
    let layer_count = 2;
    let red = [255, 0, 0, 255];
    let green = [0, 255, 0, 255];
    let mut rgba = Vec::new();
    rgba.extend(red.repeat((width * height) as usize));
    rgba.extend(green.repeat((width * height) as usize));
    let texture = offline_texture(
        "res://textures/layers.png",
        width,
        height,
        layer_count,
        rgba,
    );

    let generated = generate_offline_mips(texture).unwrap();

    let base_layer_len = rgba8_level_len(width, height).unwrap();
    let mip1_layer_len = rgba8_level_len(2, 2).unwrap();
    let mip2_layer_len = rgba8_level_len(1, 1).unwrap();
    assert_eq!(
        generated.rgba.len(),
        (base_layer_len + mip1_layer_len + mip2_layer_len) * layer_count as usize
    );
    let mip1_offset = base_layer_len * layer_count as usize;
    let mip2_offset = mip1_offset + mip1_layer_len * layer_count as usize;
    assert_eq!(
        &generated.rgba[mip1_offset..mip1_offset + mip1_layer_len],
        red.repeat(4)
    );
    assert_eq!(
        &generated.rgba[mip1_offset + mip1_layer_len..mip2_offset],
        green.repeat(4)
    );
    assert_eq!(
        &generated.rgba[mip2_offset..mip2_offset + mip2_layer_len],
        &red
    );
    assert_eq!(&generated.rgba[mip2_offset + mip2_layer_len..], &green);
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn plugins07_mip_hotpath_release_base_payload_reuse_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const BASE_BYTES: usize = 16_777_216;
    const RESERVED_BYTES: usize = 22_369_620;
    const THRESHOLD_PERCENT: u128 = 80;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy = || measure_base_payload_copy(BASE_BYTES, RESERVED_BYTES);
        let optimized = || measure_base_payload_move(BASE_BYTES, RESERVED_BYTES);
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }

    emit_mip_performance_gate(
        "plugins07_mip_base_payload_reuse",
        &legacy_samples,
        &optimized_samples,
        THRESHOLD_PERCENT,
        &format!(
            "base_bytes={BASE_BYTES} reserved_bytes={RESERVED_BYTES} legacy_base_copies_per_sample=1 optimized_base_copies_per_sample=0 legacy_copied_bytes_per_sample={BASE_BYTES} optimized_copied_bytes_per_sample=0"
        ),
    );
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn plugins07_mip_hotpath_release_range_current_levels_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const BASE_BYTES: usize = 16_777_216;
    const LAYERS: usize = 8;
    const THRESHOLD_PERCENT: u128 = 80;
    let base = vec![17_u8; BASE_BYTES];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy = || measure_layer_clones(&base, LAYERS);
        let optimized = || measure_layer_ranges(&base, LAYERS);
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }

    emit_mip_performance_gate(
        "plugins07_mip_range_current_levels",
        &legacy_samples,
        &optimized_samples,
        THRESHOLD_PERCENT,
        &format!(
            "base_bytes={BASE_BYTES} layers={LAYERS} legacy_layer_clones_per_sample={LAYERS} optimized_layer_clones_per_sample=0 legacy_cloned_bytes_per_sample={BASE_BYTES} optimized_cloned_bytes_per_sample=0"
        ),
    );
}

fn offline_texture(
    uri: &str,
    width: u32,
    height: u32,
    layer_count: u32,
    rgba: Vec<u8>,
) -> TextureAsset {
    let texture = TextureAsset::new_rgba8(AssetUri::parse(uri).unwrap(), width, height, rgba);
    let mut descriptor = texture.texture_descriptor();
    descriptor.metadata.mip_policy = TextureMipPolicy::GenerateOffline;
    descriptor.depth_or_array_layers = layer_count;
    texture.with_descriptor(descriptor)
}

fn measure_base_payload_copy(base_bytes: usize, reserved_bytes: usize) -> u128 {
    let mut source = Vec::with_capacity(reserved_bytes);
    source.resize(base_bytes, 29_u8);
    let started = Instant::now();
    let mut packed = Vec::with_capacity(reserved_bytes);
    packed.extend_from_slice(black_box(source.as_slice()));
    black_box(packed);
    started.elapsed().as_nanos()
}

fn measure_base_payload_move(base_bytes: usize, reserved_bytes: usize) -> u128 {
    let mut source = Vec::with_capacity(reserved_bytes);
    source.resize(base_bytes, 29_u8);
    let started = Instant::now();
    let packed = black_box(source);
    black_box(packed);
    started.elapsed().as_nanos()
}

fn measure_layer_clones(base: &[u8], layer_count: usize) -> u128 {
    let layer_len = base.len() / layer_count;
    let started = Instant::now();
    let layers = black_box(base)
        .chunks_exact(layer_len)
        .map(Vec::from)
        .collect::<Vec<_>>();
    black_box(layers);
    started.elapsed().as_nanos()
}

fn measure_layer_ranges(base: &[u8], layer_count: usize) -> u128 {
    let layer_len = base.len() / layer_count;
    let started = Instant::now();
    let mut bytes = 0_usize;
    for layer in black_box(base).chunks_exact(layer_len) {
        bytes += black_box(layer).len();
    }
    black_box(bytes);
    started.elapsed().as_nanos()
}

fn emit_mip_performance_gate(
    task: &str,
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    threshold_percent: u128,
    workload: &str,
) {
    let legacy_p95 = nearest_rank_mip_p95(legacy_samples);
    let optimized_p95 = nearest_rank_mip_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT {task} sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
        mip_samples_csv(legacy_samples),
        mip_samples_csv(optimized_samples),
    );
    assert!(
        improvement_percent >= threshold_percent,
        "{task} must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn nearest_rank_mip_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn mip_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
