use zircon_runtime::asset::AssetUri;

use super::*;

#[test]
fn editor571_packed_locations_use_dense_source_index_storage() {
    let production = include_str!("../packer.rs");
    let dense_storage = ["locations: ", "Vec<PackedSourceLocation>"].concat();
    let tree_storage = ["locations: ", "BTreeMap<usize, PackedSourceLocation>"].concat();

    assert!(production.contains(&dense_storage));
    assert!(!production.contains(&tree_storage));
}

#[test]
#[ignore = "release-only dense sprite placement lookup benchmark"]
fn editor571_dense_sprite_placement_lookup_release_benchmark_evidence() {
    use std::hint::black_box;
    use std::time::Instant;

    const ENTRY_COUNT: usize = 4_096;
    const ROUNDS: usize = 128;
    const SAMPLE_PAIRS: usize = 21;

    fn map_lookup(locations: &BTreeMap<usize, PackedSourceLocation>) -> u64 {
        let mut checksum = 0_u64;
        for _ in 0..ROUNDS {
            for index in 0..ENTRY_COUNT {
                let location = locations.get(&black_box(index)).unwrap();
                checksum = checksum.wrapping_add(u64::from(location.x));
            }
        }
        checksum
    }

    fn dense_lookup(locations: &[PackedSourceLocation]) -> u64 {
        let mut checksum = 0_u64;
        for _ in 0..ROUNDS {
            for index in 0..ENTRY_COUNT {
                let location = locations.get(black_box(index)).unwrap();
                checksum = checksum.wrapping_add(u64::from(location.x));
            }
        }
        checksum
    }

    fn measure(operation: impl FnOnce() -> u64) -> (u128, u64) {
        let started = Instant::now();
        let checksum = operation();
        (started.elapsed().as_nanos().max(1), checksum)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let dense_locations = (0..ENTRY_COUNT)
        .map(|index| PackedSourceLocation {
            x: index as u32,
            y: 0,
            width: 1,
            height: 1,
        })
        .collect::<Vec<_>>();
    let tree_locations = dense_locations
        .iter()
        .copied()
        .enumerate()
        .collect::<BTreeMap<_, _>>();

    let (_, tree_checksum) = measure(|| map_lookup(&tree_locations));
    let (_, dense_checksum) = measure(|| dense_lookup(&dense_locations));
    assert_eq!(tree_checksum, dense_checksum);

    for _ in 0..4 {
        black_box(measure(|| map_lookup(&tree_locations)));
        black_box(measure(|| dense_lookup(&dense_locations)));
    }

    let mut tree_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut dense_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            tree_samples.push(measure(|| map_lookup(&tree_locations)).0);
            dense_samples.push(measure(|| dense_lookup(&dense_locations)).0);
        } else {
            dense_samples.push(measure(|| dense_lookup(&dense_locations)).0);
            tree_samples.push(measure(|| map_lookup(&tree_locations)).0);
        }
    }

    let tree_p95_ns = percentile(&tree_samples, 95);
    let dense_p95_ns = percentile(&dense_samples, 95);
    println!(
        "EDITOR571_DENSE_PLACEMENT_LOOKUP_BENCH_V1 entry_count={ENTRY_COUNT} \
             rounds={ROUNDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_tree_even \
             tree_p95_ns={tree_p95_ns} dense_p95_ns={dense_p95_ns} tree_raw_ns={} \
             dense_raw_ns={}",
        raw(&tree_samples),
        raw(&dense_samples),
    );
    assert!(
        dense_p95_ns.saturating_mul(4) <= tree_p95_ns,
        "dense placement lookup must reduce P95 by at least 75%: \
             tree={tree_p95_ns}ns dense={dense_p95_ns}ns"
    );
}

fn source(name: &str, width: u32, height: u32, color: [u8; 4]) -> SpriteAtlasSourceImage {
    let mut rgba = Vec::new();
    for _ in 0..(width * height) {
        rgba.extend_from_slice(&color);
    }
    SpriteAtlasSourceImage {
        name: name.to_string(),
        source_uri: Some(AssetUri::parse(&format!("res://ui/{name}.png")).unwrap()),
        width,
        height,
        rgba,
    }
}

fn patterned_source(name: &str, width: u32, height: u32) -> SpriteAtlasSourceImage {
    let mut rgba = Vec::new();
    for y in 0..height {
        for x in 0..width {
            rgba.extend_from_slice(&[x as u8, y as u8, (x + y) as u8, 255]);
        }
    }
    SpriteAtlasSourceImage {
        name: name.to_string(),
        source_uri: Some(AssetUri::parse(&format!("res://ui/{name}.png")).unwrap()),
        width,
        height,
        rgba,
    }
}

#[test]
fn sprite_atlas_packer_packs_sources_in_deterministic_entry_order() {
    let config = SpriteAtlasBuildConfig {
        initial_size: (8, 8),
        max_size: (8, 8),
        ..Default::default()
    };
    let sources = vec![
        source("search", 2, 2, [255, 0, 0, 255]),
        source("close", 2, 2, [0, 255, 0, 255]),
        source("play", 2, 2, [0, 0, 255, 255]),
    ];

    let packed = pack_sprite_atlas_sources(&config, &sources).expect("sources should pack");

    let names = packed
        .atlas
        .entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["search", "close", "play"]);
    assert_eq!(packed.diagnostics.source_count, 3);
    assert_eq!(packed.diagnostics.packed_count, 3);
    assert_eq!(packed.diagnostics.atlas_width, 8);
    assert_eq!(packed.diagnostics.atlas_height, 8);
    assert_eq!(
        packed.atlas.atlas_texture.to_string(),
        "lib://editor-sprite-atlases/editor-atlas.png"
    );
}

#[test]
fn sprite_atlas_packer_copies_source_rgba_rows_into_atlas() {
    let config = SpriteAtlasBuildConfig {
        padding: (0, 0),
        initial_size: (4, 4),
        max_size: (4, 4),
        ..Default::default()
    };
    let sources = vec![patterned_source("search", 2, 2)];

    let packed = pack_sprite_atlas_sources(&config, &sources).expect("source should pack");
    let rect = packed.atlas.entries[0].pixel_rect;
    let atlas_width = packed.atlas.width as usize;

    for y in rect.y..(rect.y + rect.height) {
        for x in rect.x..(rect.x + rect.width) {
            let offset = ((y as usize * atlas_width) + x as usize) * RGBA8_BYTES_PER_PIXEL;
            let local_x = x - rect.x;
            let local_y = y - rect.y;
            assert_eq!(
                &packed.rgba[offset..offset + 4],
                &[local_x as u8, local_y as u8, (local_x + local_y) as u8, 255]
            );
        }
    }
}

#[test]
fn sprite_atlas_packer_derives_uvs_from_pixel_rect_without_padding() {
    let config = SpriteAtlasBuildConfig {
        padding: (1, 1),
        initial_size: (4, 4),
        max_size: (4, 4),
        ..Default::default()
    };
    let sources = vec![source("search", 2, 2, [255, 0, 0, 255])];

    let packed = pack_sprite_atlas_sources(&config, &sources).expect("source should pack");
    let entry = &packed.atlas.entries[0];

    assert_eq!(entry.pixel_rect.x, 0);
    assert_eq!(entry.pixel_rect.y, 0);
    assert_eq!(entry.pixel_rect.width, 2);
    assert_eq!(entry.pixel_rect.height, 2);
    assert_eq!(entry.uv_rect.min, [0.0, 0.0]);
    assert_eq!(entry.uv_rect.max, [0.5, 0.5]);
}

#[test]
fn sprite_atlas_packer_keeps_padding_out_of_pixel_rects() {
    let config = SpriteAtlasBuildConfig {
        padding: (2, 3),
        initial_size: (8, 8),
        max_size: (8, 8),
        ..Default::default()
    };
    let sources = vec![source("search", 2, 2, [255, 0, 0, 255])];

    let packed = pack_sprite_atlas_sources(&config, &sources).expect("source should pack");

    assert_eq!(packed.atlas.padding.x, 2);
    assert_eq!(packed.atlas.padding.y, 3);
    assert_eq!(packed.atlas.entries[0].pixel_rect.width, 2);
    assert_eq!(packed.atlas.entries[0].pixel_rect.height, 2);
}

#[test]
fn sprite_atlas_packer_reports_pack_failure_when_max_size_is_too_small() {
    let config = SpriteAtlasBuildConfig {
        initial_size: (2, 2),
        max_size: (2, 2),
        ..Default::default()
    };
    let sources = vec![source("search", 4, 4, [255, 0, 0, 255])];

    let error = pack_sprite_atlas_sources(&config, &sources).expect_err("pack should fail");
    let SpriteAtlasBuildError::PackFailed {
        max_width,
        max_height,
        diagnostics,
    } = error
    else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(max_width, 2);
    assert_eq!(max_height, 2);
    assert_eq!(diagnostics.source_count, 1);
    assert_eq!(diagnostics.packed_count, 0);
    assert_eq!(diagnostics.packed_area, 25);
    assert_eq!(diagnostics.atlas_area, 4);
    assert_eq!(diagnostics.skipped_sources, vec!["search".to_string()]);
}

#[test]
fn sprite_atlas_packer_rejects_unsafe_output_stem() {
    let config = SpriteAtlasBuildConfig {
        output_stem: "../icons".to_string(),
        initial_size: (4, 4),
        max_size: (4, 4),
        ..Default::default()
    };
    let sources = vec![source("search", 2, 2, [255, 0, 0, 255])];

    assert_eq!(
        pack_sprite_atlas_sources(&config, &sources),
        Err(SpriteAtlasBuildError::InvalidConfig(
            "output_stem must be a single safe file stem".to_string()
        ))
    );
}

#[test]
fn sprite_atlas_packer_rejects_uri_metacharacter_output_stem() {
    let config = SpriteAtlasBuildConfig {
        output_stem: "icons#dark".to_string(),
        initial_size: (4, 4),
        max_size: (4, 4),
        ..Default::default()
    };
    let sources = vec![source("search", 2, 2, [255, 0, 0, 255])];

    assert_eq!(
        pack_sprite_atlas_sources(&config, &sources),
        Err(SpriteAtlasBuildError::InvalidConfig(
            "output_stem must be a single safe file stem".to_string()
        ))
    );
}

#[test]
fn sprite_atlas_packer_keeps_padded_sources_separate() {
    let config = SpriteAtlasBuildConfig {
        padding: (1, 0),
        initial_size: (6, 2),
        max_size: (6, 2),
        ..Default::default()
    };
    let sources = vec![
        source("left", 2, 2, [255, 0, 0, 255]),
        source("right", 2, 2, [0, 255, 0, 255]),
    ];

    let packed = pack_sprite_atlas_sources(&config, &sources).expect("sources should pack");
    let first = packed.atlas.entries[0].pixel_rect;
    let second = packed.atlas.entries[1].pixel_rect;
    let separated_horizontally = first.x + first.width + config.padding.0 <= second.x
        || second.x + second.width + config.padding.0 <= first.x;
    let separated_vertically = first.y + first.height + config.padding.1 <= second.y
        || second.y + second.height + config.padding.1 <= first.y;

    assert!(separated_horizontally || separated_vertically);
}

#[test]
fn sprite_atlas_packer_decodes_source_images_to_rgba8() {
    let image = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_pixel(
        2,
        1,
        image::Rgba([3, 5, 7, 255]),
    );
    let mut encoded = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();

    let source = decode_sprite_atlas_source_image(
        "decoded",
        Some(AssetUri::parse("res://ui/decoded.png").unwrap()),
        encoded.get_ref(),
    )
    .unwrap();

    assert_eq!(source.width, 2);
    assert_eq!(source.height, 1);
    assert_eq!(source.rgba, vec![3, 5, 7, 255, 3, 5, 7, 255]);
}
