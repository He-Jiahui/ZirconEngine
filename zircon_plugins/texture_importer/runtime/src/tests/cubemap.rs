use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use image::{Rgba, RgbaImage};
use zircon_runtime::asset::{
    AssetImportContext, AssetUri, ImportedAsset, TexturePayload, ZCUBE_SOURCE_CUBEMAP_FORMAT,
    encode_source_cubemap_zcube_rgba16f_mips_owned,
};
use zircon_runtime::core::framework::render::RenderImageDimension;

use crate::array::contiguous_rgba_layer_bytes;
use crate::cubemap::sample_equirect_bilinear;
use crate::plugin_registration;

#[test]
fn binary_source_cubemap_container_imports_without_manifest_decode_or_f32_expansion() {
    let root = test_root("binary-source-cubemap");
    let source_rgba16f = vec![0x3c; 6 * (2 * 2 + 1) * 8];
    let encoded = encode_source_cubemap_zcube_rgba16f_mips_owned(2, 2, source_rgba16f).unwrap();

    let texture = import_cubemap_bytes(&root, "captured.zcube", encoded.clone());

    assert_cube_descriptor(&texture, 2);
    let TexturePayload::Container {
        format,
        bytes,
        mip_count,
        array_layers,
    } = &texture.payload
    else {
        panic!("binary source cubemap must remain a container");
    };
    assert_eq!(format, ZCUBE_SOURCE_CUBEMAP_FORMAT);
    assert_eq!(bytes, &encoded);
    assert_eq!(*mip_count, 2);
    assert_eq!(*array_layers, 6);
}

#[test]
fn six_file_cubemap_manifest_imports_in_wgpu_face_order() {
    let root = test_root("six-files");
    let sources = (0_u8..6)
        .map(|face| {
            let name = format!("face_{face}.png");
            save_image(&root.join(&name), solid_image(2, 2, [face, 0, 0, 255]));
            format!("\"{name}\"")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let manifest = format!("layout = \"six_files\"\nsources = [{sources}]\n");

    let texture = import_manifest(&root, "six.zcube", &manifest);

    assert_cube_descriptor(&texture, 2);
    for face in 0_usize..6 {
        assert_eq!(texture.rgba[face * 2 * 2 * 4], face as u8);
    }
}

#[test]
fn horizontal_cross_cubemap_manifest_uses_cmft_tile_layout() {
    let root = test_root("horizontal-cross");
    let mut cross = RgbaImage::new(8, 6);
    let offsets = [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1), (3, 1)];
    for (face, (column, row)) in offsets.into_iter().enumerate() {
        fill_tile(&mut cross, column, row, [face as u8, 0, 0, 255]);
    }
    save_image(&root.join("cross.png"), cross);
    let manifest = "layout = \"horizontal_cross\"\nsources = [\"cross.png\"]\n";

    let texture = import_manifest(&root, "horizontal.zcube", manifest);

    assert_cube_descriptor(&texture, 2);
    for face in 0_usize..6 {
        assert_eq!(texture.rgba[face * 2 * 2 * 4], face as u8);
    }
}

#[test]
fn vertical_cross_cubemap_manifest_rotates_negative_z_like_cmft() {
    let root = test_root("vertical-cross");
    let mut cross = RgbaImage::new(6, 8);
    let offsets = [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1)];
    for (face, (column, row)) in offsets.into_iter().enumerate() {
        fill_tile(&mut cross, column, row, [face as u8, 0, 0, 255]);
    }
    let negative_z = [[1_u8, 2_u8], [3_u8, 4_u8]];
    for (y, row) in negative_z.into_iter().enumerate() {
        for (x, value) in row.into_iter().enumerate() {
            cross.put_pixel(2 + x as u32, 6 + y as u32, Rgba([value, 0, 0, 255]));
        }
    }
    save_image(&root.join("cross.png"), cross);
    let manifest = "layout = \"vertical_cross\"\nsources = [\"cross.png\"]\n";

    let texture = import_manifest(&root, "vertical.zcube", manifest);

    assert_cube_descriptor(&texture, 2);
    let negative_z_offset = 5 * 2 * 2 * 4;
    let values = (0..4)
        .map(|pixel| texture.rgba[negative_z_offset + pixel * 4])
        .collect::<Vec<_>>();
    assert_eq!(values, vec![4, 3, 2, 1]);
}

#[test]
fn equirectangular_cubemap_manifest_resamples_all_six_faces() {
    let root = test_root("equirectangular");
    let image = RgbaImage::from_fn(8, 4, |x, y| Rgba([(x * 24) as u8, (y * 64) as u8, 0, 255]));
    save_image(&root.join("environment.png"), image);
    let manifest = "layout = \"equirectangular\"\nsources = [\"environment.png\"]\n";

    let texture = import_manifest(&root, "environment.zcube", manifest);

    assert_cube_descriptor(&texture, 2);
    assert_eq!(texture.rgba.len(), 2 * 2 * 6 * 4);
    assert_ne!(&texture.rgba[0..16], &texture.rgba[16..32]);
}

#[test]
fn texture_array_manifest_imports_files_and_stacked_slice() {
    let root = test_root("array");
    save_image(&root.join("red.png"), solid_image(2, 2, [1, 0, 0, 255]));
    save_image(&root.join("blue.png"), solid_image(2, 2, [2, 0, 0, 255]));
    let files_manifest = "sources = [\"red.png\", \"blue.png\"]\n";

    let files = import_manifest(&root, "files.zarray", files_manifest);

    assert_eq!(files.render_image_descriptor().array_layer_count, 2);
    assert_eq!(files.rgba[0], 1);
    assert_eq!(files.rgba[16], 2);

    let stacked = RgbaImage::from_fn(2, 4, |_x, y| Rgba([if y < 2 { 3 } else { 4 }, 0, 0, 255]));
    save_image(&root.join("stacked.png"), stacked);
    let slice_manifest = "source = \"stacked.png\"\nrow_count = 2\n";

    let sliced = import_manifest(&root, "slice.zarray", slice_manifest);

    assert_eq!(sliced.render_image_descriptor().array_layer_count, 2);
    assert_eq!(sliced.rgba[0], 3);
    assert_eq!(sliced.rgba[16], 4);
}

#[test]
fn texture_hotpath_equirect_neighborhood_matches_channel_reference() {
    let image = patterned_image(17, 9);
    for uv in [
        [-0.001, 0.0],
        [0.0, 0.5],
        [0.999, 0.5],
        [1.001, 1.0],
        [0.371, 0.823],
    ] {
        assert_eq!(
            sample_equirect_bilinear(&image, uv),
            legacy_sample_equirect_bilinear(&image, uv),
            "bilinear neighborhood mismatch at {uv:?}"
        );
    }
}

#[test]
fn texture_hotpath_contiguous_array_slices_match_crop_reference() {
    let image = patterned_image(13, 15);

    let optimized = contiguous_rgba_layer_bytes(&image, 3).collect::<Vec<_>>();
    let legacy = legacy_crop_rgba_layers(&image, 3);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.len(), 5);
}

#[test]
#[ignore = "release performance gate"]
fn texture_hotpath_equirect_release_gate_reuses_pixel_neighborhood() {
    const SAMPLE_PAIRS: usize = 21;
    const UV_SAMPLES: usize = 8_192;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 40;

    let image = patterned_image(1_024, 512);
    let uv_samples = (0..UV_SAMPLES)
        .map(|index| {
            let x = index % 128;
            let y = index / 128;
            [(x as f32 + 0.37) / 128.0, (y as f32 + 0.63) / 64.0]
        })
        .collect::<Vec<_>>();
    for _ in 0..2 {
        black_box(measure_equirect_sampler(
            &image,
            &uv_samples,
            legacy_sample_equirect_bilinear,
        ));
        black_box(measure_equirect_sampler(
            &image,
            &uv_samples,
            sample_equirect_bilinear,
        ));
    }

    let (legacy_samples, optimized_samples) = alternating_samples(
        SAMPLE_PAIRS,
        || measure_equirect_sampler(&image, &uv_samples, legacy_sample_equirect_bilinear),
        || measure_equirect_sampler(&image, &uv_samples, sample_equirect_bilinear),
    );
    assert_performance_gate(
        "plugins07_equirect_neighborhood",
        &legacy_samples,
        &optimized_samples,
        REQUIRED_IMPROVEMENT_PERCENT,
        &format!(
            "uv_samples={UV_SAMPLES} legacy_pixel_fetches_per_sample={} optimized_pixel_fetches_per_sample={} source_width=1024 source_height=512",
            UV_SAMPLES * 16,
            UV_SAMPLES * 4
        ),
    );
}

#[test]
#[ignore = "release performance gate"]
fn texture_hotpath_array_release_gate_uses_contiguous_layer_copies() {
    const SAMPLE_PAIRS: usize = 21;
    const WIDTH: u32 = 512;
    const LAYER_HEIGHT: u32 = 8;
    const LAYERS: u32 = 128;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 30;

    let image = patterned_image(WIDTH, LAYER_HEIGHT * LAYERS);
    for _ in 0..2 {
        black_box(measure_array_extraction(
            &image,
            LAYER_HEIGHT,
            legacy_crop_rgba_layers,
        ));
        black_box(measure_array_extraction(
            &image,
            LAYER_HEIGHT,
            optimized_contiguous_rgba_layers,
        ));
    }

    let (legacy_samples, optimized_samples) = alternating_samples(
        SAMPLE_PAIRS,
        || measure_array_extraction(&image, LAYER_HEIGHT, legacy_crop_rgba_layers),
        || measure_array_extraction(&image, LAYER_HEIGHT, optimized_contiguous_rgba_layers),
    );
    assert_performance_gate(
        "plugins07_contiguous_array_layers",
        &legacy_samples,
        &optimized_samples,
        REQUIRED_IMPROVEMENT_PERCENT,
        &format!(
            "width={WIDTH} layer_height={LAYER_HEIGHT} layers={LAYERS} legacy_generic_crops_per_sample={LAYERS} optimized_contiguous_copies_per_sample={LAYERS}"
        ),
    );
}

fn legacy_sample_equirect_bilinear(image: &RgbaImage, uv: [f32; 2]) -> Rgba<u8> {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let x = uv[0].rem_euclid(1.0) * width as f32 - 0.5;
    let y = uv[1].clamp(0.0, 1.0) * height as f32 - 0.5;
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let tx = x - x.floor();
    let ty = y - y.floor();
    let mut result = [0_u8; 4];
    for (channel, value) in result.iter_mut().enumerate() {
        let sample = |sx: i64, sy: i64| {
            let wrapped_x = sx.rem_euclid(width as i64) as u32;
            let clamped_y = sy.clamp(0, height as i64 - 1) as u32;
            image.get_pixel(wrapped_x, clamped_y)[channel] as f32
        };
        let top = sample(x0, y0) * (1.0 - tx) + sample(x0 + 1, y0) * tx;
        let bottom = sample(x0, y0 + 1) * (1.0 - tx) + sample(x0 + 1, y0 + 1) * tx;
        *value = (top * (1.0 - ty) + bottom * ty).round() as u8;
    }
    Rgba(result)
}

fn legacy_crop_rgba_layers(image: &RgbaImage, layer_height: u32) -> Vec<Vec<u8>> {
    (0..image.height() / layer_height)
        .map(|layer| {
            image::imageops::crop_imm(image, 0, layer * layer_height, image.width(), layer_height)
                .to_image()
                .into_raw()
        })
        .collect()
}

fn optimized_contiguous_rgba_layers(image: &RgbaImage, layer_height: u32) -> Vec<Vec<u8>> {
    contiguous_rgba_layer_bytes(image, layer_height).collect()
}

fn measure_equirect_sampler(
    image: &RgbaImage,
    uv_samples: &[[f32; 2]],
    sampler: fn(&RgbaImage, [f32; 2]) -> Rgba<u8>,
) -> Duration {
    let started = Instant::now();
    let mut checksum = 0_u64;
    for &uv in uv_samples {
        let pixel = sampler(black_box(image), black_box(uv));
        checksum = checksum.wrapping_add(pixel.0.iter().map(|value| u64::from(*value)).sum());
    }
    black_box(checksum);
    started.elapsed()
}

fn measure_array_extraction(
    image: &RgbaImage,
    layer_height: u32,
    extractor: fn(&RgbaImage, u32) -> Vec<Vec<u8>>,
) -> Duration {
    let started = Instant::now();
    black_box(extractor(black_box(image), black_box(layer_height)));
    started.elapsed()
}

fn alternating_samples(
    sample_pairs: usize,
    mut legacy: impl FnMut() -> Duration,
    mut optimized: impl FnMut() -> Duration,
) -> (Vec<Duration>, Vec<Duration>) {
    let mut legacy_samples = Vec::with_capacity(sample_pairs);
    let mut optimized_samples = Vec::with_capacity(sample_pairs);
    for pair in 0..sample_pairs {
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn assert_performance_gate(
    marker: &str,
    legacy_samples: &[Duration],
    optimized_samples: &[Duration],
    threshold_percent: u128,
    workload: &str,
) {
    let legacy_p95 = nearest_rank_p95(legacy_samples).as_nanos();
    let optimized_p95 = nearest_rank_p95(optimized_samples).as_nanos();
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT {marker} sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
        durations_csv(legacy_samples),
        durations_csv(optimized_samples),
    );
    assert!(
        improvement_percent >= threshold_percent,
        "{marker} must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn nearest_rank_p95(samples: &[Duration]) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn durations_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn patterned_image(width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| {
        Rgba([
            x.wrapping_mul(17).wrapping_add(y.wrapping_mul(3)) as u8,
            x.wrapping_mul(5).wrapping_add(y.wrapping_mul(29)) as u8,
            x.wrapping_mul(11).wrapping_add(y.wrapping_mul(7)) as u8,
            255,
        ])
    })
}

fn import_manifest(root: &Path, name: &str, manifest: &str) -> zircon_runtime::asset::TextureAsset {
    import_cubemap_bytes(root, name, manifest.as_bytes().to_vec())
}

fn import_cubemap_bytes(
    root: &Path,
    name: &str,
    bytes: Vec<u8>,
) -> zircon_runtime::asset::TextureAsset {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(Path::new(name))
        .expect("manifest importer registration");
    let source_path = root.join(name);
    let context = AssetImportContext::new(
        source_path,
        AssetUri::parse(&format!("res://textures/{name}")).expect("valid manifest uri"),
        bytes,
        Default::default(),
    );
    let outcome = importer.import(&context).expect("manifest import");
    match &outcome.root_entry().expect("root texture entry").asset {
        ImportedAsset::Texture(texture) => texture.clone(),
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

fn assert_cube_descriptor(texture: &zircon_runtime::asset::TextureAsset, face_size: u32) {
    let descriptor = texture.render_image_descriptor();
    assert_eq!(texture.width, face_size);
    assert_eq!(texture.height, face_size);
    assert_eq!(descriptor.dimension, RenderImageDimension::Cube);
    assert_eq!(descriptor.array_layer_count, 6);
}

fn solid_image(width: u32, height: u32, color: [u8; 4]) -> RgbaImage {
    RgbaImage::from_pixel(width, height, Rgba(color))
}

fn fill_tile(image: &mut RgbaImage, column: u32, row: u32, color: [u8; 4]) {
    for y in 0..2 {
        for x in 0..2 {
            image.put_pixel(column * 2 + x, row * 2 + y, Rgba(color));
        }
    }
}

fn save_image(path: &Path, image: RgbaImage) {
    image.save(path).expect("save test image");
}

fn test_root(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-texture-importer-{label}-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("create test directory");
    root
}
