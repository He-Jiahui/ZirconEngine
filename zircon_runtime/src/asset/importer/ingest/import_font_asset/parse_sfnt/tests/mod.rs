mod fixtures;

use std::hint::black_box;
use std::time::Instant;

use ttf_parser::Face;
use ttf2woff2::{BrotliQuality, encode};

use super::*;
use crate::asset::assets::{DecodedFontSource, FontSourceBudgetError, decode_font_source};

use fixtures::{fira_regular, patch_os2_weight, ttc_from_fonts, variable_font};

const SAMPLE_PAIRS: usize = 21;
const FVAR_PARSES_PER_SAMPLE: usize = 512;
const BENCHMARK_AXIS_COUNT: usize = 8;
const BENCHMARK_INSTANCE_COUNT: usize = 64;

fn decoded(bytes: Vec<u8>) -> DecodedFontSource {
    decode_font_source(bytes).expect("font fixture should decode")
}

#[test]
fn text_font_cmap_coverage_bitset_matches_face() {
    let bytes = fira_regular();
    let source = decoded(bytes.clone());
    let metadata = parse_font_metadata(&source).unwrap();
    let face = Face::parse(&bytes, 0).unwrap();
    let coverage = &metadata.faces[0].cmap;

    assert!(coverage.codepoint_count > 0);
    for ch in ['A', 'a', '0'] {
        assert_eq!(
            coverage.contains_codepoint(ch as u32),
            face.glyph_index(ch).is_some()
        );
    }
}

#[test]
fn text_font_cmap_coverage_bitmap_deduplicates_and_compacts_cross_word_ranges() {
    let mut coverage = UnicodeScalarCoverage::default();
    for codepoint in [62, 63, 64, 64, 128] {
        coverage.insert(codepoint);
    }

    let coverage = coverage
        .into_asset_coverage(0, usize::MAX)
        .expect("small bitmap coverage should stay below its range budget");

    assert_eq!(coverage.codepoint_count, 4);
    assert_eq!(
        coverage.ranges,
        vec![
            FontAssetCodepointRange { start: 62, end: 64 },
            FontAssetCodepointRange {
                start: 128,
                end: 128,
            },
        ]
    );
}

#[test]
fn text_font_cmap_coverage_rejects_excessive_serialized_ranges() {
    let mut coverage = UnicodeScalarCoverage::default();
    for codepoint in [1, 3, 5] {
        coverage.insert(codepoint);
    }

    let error = coverage.into_asset_coverage(7, 2).unwrap_err();

    assert_eq!(
        error,
        FontSourceBudgetError::CmapRangeCount {
            face_index: 7,
            limit: 2,
            observed_at_least: 3,
        }
    );
}

#[test]
fn text_font_static_face_reports_no_variable_axes() {
    let metadata = parse_font_metadata(&decoded(fira_regular())).unwrap();

    assert_eq!(metadata.face_count, 1);
    assert!(metadata.faces[0].variation_axes.is_empty());
    assert!(metadata.faces[0].named_instances.is_empty());
}

#[test]
fn text_font_parse_ttf_extracts_os2_name_metadata() {
    let original = fira_regular();
    let expected = Face::parse(&original, 0).unwrap();
    let metadata = parse_font_metadata(&decoded(original.clone())).unwrap();
    let face = &metadata.faces[0];

    assert_eq!(metadata.source_format, FontAssetSourceFormat::Sfnt);
    assert_eq!(face.face_index, 0);
    assert!(
        face.family
            .as_deref()
            .is_some_and(|family| family.contains("Fira"))
    );
    assert_eq!(face.weight, 400);
    assert_eq!(face.width_class, 5);
    assert_eq!(face.style, FontAssetFaceStyle::Normal);
    assert_eq!(face.metrics.units_per_em, expected.units_per_em());
    assert_eq!(face.metrics.ascender, expected.ascender());
    assert_eq!(face.metrics.descender, expected.descender());
    assert_eq!(face.metrics.line_gap, expected.line_gap());
    assert_eq!(
        face.metrics.underline.map(|metrics| metrics.position),
        expected.underline_metrics().map(|metrics| metrics.position)
    );
    assert_eq!(
        face.metrics.strikeout.map(|metrics| metrics.thickness),
        expected
            .strikeout_metrics()
            .map(|metrics| metrics.thickness)
    );
}

#[test]
fn text_font_variable_axes_roundtrip() {
    let source = decoded(variable_font());
    let metadata = parse_font_metadata(&source).unwrap();
    let face = &metadata.faces[0];

    assert_eq!(face.variation_axes.len(), 1);
    let axis = &face.variation_axes[0];
    assert_eq!(axis.tag, "wght");
    assert_eq!((axis.min, axis.default, axis.max), (100.0, 400.0, 900.0));
    assert_eq!(face.named_instances.len(), 1);
    assert_eq!(face.named_instances[0].coordinates.len(), 1);
    assert_eq!(face.named_instances[0].coordinates[0].tag, "wght");
    assert_eq!(face.named_instances[0].coordinates[0].value, 650.0);
}

#[test]
fn text_font_parse_ttc_enumerates_faces() {
    let regular = fira_regular();
    let mut second_face = regular.clone();
    patch_os2_weight(&mut second_face, 700);
    let collection = ttc_from_fonts(&[regular.as_slice(), second_face.as_slice()]);

    let metadata = parse_font_metadata(&decoded(collection)).unwrap();

    assert_eq!(
        metadata.source_format,
        FontAssetSourceFormat::TrueTypeCollection
    );
    assert_eq!(metadata.face_count, 2);
    assert_eq!(metadata.faces[0].face_index, 0);
    assert_eq!(metadata.faces[1].face_index, 1);
    assert_eq!(metadata.faces[0].weight, 400);
    assert_eq!(metadata.faces[1].weight, 700);
}

#[test]
fn text_font_woff2_decodes_to_sfnt() {
    let original = fira_regular();
    let woff2 = encode(&original, BrotliQuality::default()).unwrap();
    assert!(woff2.starts_with(b"wOF2"));

    let source = decoded(woff2);
    assert_eq!(source.source_format(), FontAssetSourceFormat::Woff2);
    assert!(!source.bytes().starts_with(b"wOF2"));

    let metadata = parse_font_metadata(&source).unwrap();
    assert_eq!(metadata.source_format, FontAssetSourceFormat::Woff2);
    assert_eq!(metadata.face_count, 1);
    assert!(
        metadata.faces[0]
            .family
            .as_deref()
            .is_some_and(|family| family.contains("Fira"))
    );
    assert!(Face::parse(source.bytes(), 0).is_ok());
}

#[test]
fn text_font_malformed_woff2_preserves_decode_failure() {
    let error = decode_font_source(b"wOF2invalid".to_vec()).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("WOFF2 font source decode failed")
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn text_font_fvar_capacity_preserves_all_instances_and_coordinates() {
    let fvar = benchmark_fvar_table();

    let instances = parse_named_instances(Some(&fvar), BENCHMARK_AXIS_COUNT, |_| None);

    assert_eq!(instances.len(), BENCHMARK_INSTANCE_COUNT);
    assert!(
        instances
            .iter()
            .all(|instance| instance.coordinates.len() == BENCHMARK_AXIS_COUNT)
    );
    assert_eq!(instances[0].coordinates[0].tag, "AX00");
    assert_eq!(instances[63].coordinates[7].tag, "AX07");
}

#[test]
fn text_font_fvar_capacity_is_bounded_by_available_records() {
    let mut fvar = benchmark_fvar_table();
    fvar[12..14].copy_from_slice(&u16::MAX.to_be_bytes());
    let instances_offset = 16 + BENCHMARK_AXIS_COUNT * 20;
    let instance_size = 4 + BENCHMARK_AXIS_COUNT * 4;
    fvar.truncate(instances_offset + instance_size);

    let instances = parse_named_instances(Some(&fvar), BENCHMARK_AXIS_COUNT, |_| None);

    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].coordinates.len(), BENCHMARK_AXIS_COUNT);
}

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_text_font_fvar_capacity_projection() {
    let fvar = benchmark_fvar_table();
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_fvar_parses(legacy_parse_named_instances, &fvar));
            optimized_raw.push(measure_fvar_parses(optimized_parse_named_instances, &fvar));
        } else {
            optimized_raw.push(measure_fvar_parses(optimized_parse_named_instances, &fvar));
            legacy_raw.push(measure_fvar_parses(legacy_parse_named_instances, &fvar));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
        "bounded fvar capacity projection must improve P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "PERF_RESULT task=plugins07_bounded_font_fvar_capacity sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank parses_per_sample={FVAR_PARSES_PER_SAMPLE} axes_per_font={BENCHMARK_AXIS_COUNT} instances_per_font={BENCHMARK_INSTANCE_COUNT} legacy_axis_tag_initial_capacity=0 optimized_axis_tag_reserved_capacity=8 legacy_instance_initial_capacity=0 optimized_instance_reserved_capacity=64 legacy_coordinate_initial_capacity=0 optimized_coordinate_reserved_capacity=8 threshold_percent=15 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

fn optimized_parse_named_instances(fvar: &[u8]) -> usize {
    black_box(parse_named_instances(
        Some(black_box(fvar)),
        BENCHMARK_AXIS_COUNT,
        |_| None,
    ))
    .len()
}

fn legacy_parse_named_instances(fvar: &[u8]) -> usize {
    let axis_count = BENCHMARK_AXIS_COUNT;
    let axes_array_offset = read_u16(fvar, 4).map(usize::from).unwrap_or(0);
    let axis_size = read_u16(fvar, 10).map(usize::from).unwrap_or(20);
    let instance_count = read_u16(fvar, 12).map(usize::from).unwrap_or(0);
    let instance_size = read_u16(fvar, 14).map(usize::from).unwrap_or(0);
    let instances_offset = axes_array_offset + axis_count * axis_size;
    let axis_tags = legacy_parse_axis_tags(fvar, axes_array_offset, axis_count, axis_size);
    let mut instances = Vec::new();
    for index in 0..instance_count {
        let offset = instances_offset + index * instance_size;
        let Some(subfamily_name_id) = read_u16(fvar, offset) else {
            break;
        };
        let mut coordinates = Vec::new();
        for axis_index in 0..axis_count {
            let Some(value) = read_fixed(fvar, offset + 4 + axis_index * 4) else {
                continue;
            };
            if let Some(tag) = axis_tags.get(axis_index) {
                coordinates.push(FontAssetVariationCoord {
                    tag: tag.clone(),
                    value,
                });
            }
        }
        black_box(subfamily_name_id);
        instances.push(FontAssetVariableInstance {
            name: None,
            post_script_name: None,
            coordinates,
        });
    }
    black_box(instances).len()
}

fn legacy_parse_axis_tags(
    fvar: &[u8],
    axes_array_offset: usize,
    axis_count: usize,
    axis_size: usize,
) -> Vec<String> {
    (0..axis_count)
        .filter_map(|axis_index| {
            let offset = axes_array_offset.checked_add(axis_index * axis_size)?;
            let tag = fvar.get(offset..offset + 4)?;
            Some(String::from_utf8_lossy(tag).into_owned())
        })
        .collect()
}

fn benchmark_fvar_table() -> Vec<u8> {
    let axis_count = BENCHMARK_AXIS_COUNT as u16;
    let instance_count = BENCHMARK_INSTANCE_COUNT as u16;
    let axis_size = 20u16;
    let instance_size = 4u16 + axis_count * 4;
    let mut table = Vec::with_capacity(
        16 + usize::from(axis_count) * usize::from(axis_size)
            + usize::from(instance_count) * usize::from(instance_size),
    );
    table.extend_from_slice(&1u16.to_be_bytes());
    table.extend_from_slice(&0u16.to_be_bytes());
    table.extend_from_slice(&16u16.to_be_bytes());
    table.extend_from_slice(&2u16.to_be_bytes());
    table.extend_from_slice(&axis_count.to_be_bytes());
    table.extend_from_slice(&axis_size.to_be_bytes());
    table.extend_from_slice(&instance_count.to_be_bytes());
    table.extend_from_slice(&instance_size.to_be_bytes());
    for axis_index in 0..BENCHMARK_AXIS_COUNT {
        table.extend_from_slice(&[
            b'A',
            b'X',
            b'0' + u8::try_from(axis_index / 10).unwrap(),
            b'0' + u8::try_from(axis_index % 10).unwrap(),
        ]);
        table.extend_from_slice(&(100i32 << 16).to_be_bytes());
        table.extend_from_slice(&(400i32 << 16).to_be_bytes());
        table.extend_from_slice(&(900i32 << 16).to_be_bytes());
        table.extend_from_slice(&0u16.to_be_bytes());
        table.extend_from_slice(&(256u16 + axis_index as u16).to_be_bytes());
    }
    for instance_index in 0..BENCHMARK_INSTANCE_COUNT {
        table.extend_from_slice(&(512u16 + instance_index as u16).to_be_bytes());
        table.extend_from_slice(&0u16.to_be_bytes());
        for axis_index in 0..BENCHMARK_AXIS_COUNT {
            let value = (300 + instance_index as i32 + axis_index as i32) << 16;
            table.extend_from_slice(&value.to_be_bytes());
        }
    }
    table
}

fn measure_fvar_parses(parser: fn(&[u8]) -> usize, fvar: &[u8]) -> u64 {
    let started = Instant::now();
    let mut instances = 0;
    for _ in 0..FVAR_PARSES_PER_SAMPLE {
        instances += parser(black_box(fvar));
    }
    black_box(instances);
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
