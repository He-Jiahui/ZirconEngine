use std::path::Path;
use std::sync::Arc;

use ttf_parser::Face;

use super::*;
use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

fn fixture_bytes() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf"))
        .expect("Fira Sans test fixture")
}

fn glyph_id(bytes: &[u8], character: char) -> u16 {
    Face::parse(bytes, 0)
        .expect("fixture face")
        .glyph_index(character)
        .expect("fixture glyph")
        .0
}

#[test]
fn text_msdf_dynamic_generation_is_deterministic() {
    let bytes = fixture_bytes();
    let glyph_id = glyph_id(&bytes, 'M');
    let params = SdfBakeParams::for_mode(SdfMode::Msdf);

    let first = generate_distance_field_glyph(&bytes, 0, glyph_id, params).unwrap();
    let second = generate_distance_field_glyph(&bytes, 0, glyph_id, params).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.mode, SdfMode::Msdf);
    assert_eq!(first.channels, 4);
    assert!(
        first
            .pixels
            .chunks_exact(4)
            .any(|pixel| { pixel[0] != pixel[1] || pixel[1] != pixel[2] })
    );
}

#[test]
fn text_sdf_generation_source_context_parses_once_and_batches_deterministically() {
    let bytes = Arc::<[u8]>::from(fixture_bytes());
    let face = Face::parse(bytes.as_ref(), 0).expect("fixture face");
    let a = face.glyph_index('A').expect("fixture A glyph").0;
    let m = face.glyph_index('M').expect("fixture M glyph").0;
    let handle = SdfGenerationSourceHandle::new(41);
    let context = SdfGenerationSourceContext::new(
        handle,
        Arc::clone(&bytes),
        0,
        Arc::new(crate::text::VariationCoords::default()),
    )
    .expect("parsed generation source");
    let params = SdfBakeParams::for_mode(SdfMode::Msdf);

    let first = context.generate_batch(params, &[m, a, m]);
    let second = context.generate_batch(params, &[a, m]);

    assert_eq!(context.handle(), handle);
    assert_ne!(
        SdfGenerationSourceHandle::for_generation(7, 0),
        SdfGenerationSourceHandle::for_generation(8, 0)
    );
    assert_eq!(context.source_hash(), sdf_font_source_hash(bytes.as_ref()));
    assert_eq!(context.report().face_parse_count, 1);
    assert_eq!(context.report().source_hash_count, 1);
    assert_eq!(first.report.requested_glyph_count, 3);
    assert_eq!(first.report.unique_glyph_count, 2);
    assert_eq!(first.report.duplicate_glyph_count, 1);
    assert_eq!(
        first
            .glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![a, m]
    );
    assert_eq!(first.glyphs, second.glyphs);
}

#[test]
fn text_sdf_generation_batch_reports_one_hundred_and_ten_thousand_requests() {
    let bytes = Arc::<[u8]>::from(fixture_bytes());
    let face = Face::parse(bytes.as_ref(), 0).expect("fixture face");
    let glyph_id = face.glyph_index('A').expect("fixture A glyph").0;
    let context = SdfGenerationSourceContext::new(
        SdfGenerationSourceHandle::new(9),
        bytes,
        0,
        Arc::new(crate::text::VariationCoords::default()),
    )
    .expect("parsed generation source");
    let params = SdfBakeParams::default();

    for request_count in [1, 100, 10_000] {
        let batch = context.generate_batch(params, &vec![glyph_id; request_count]);

        assert_eq!(batch.report.requested_glyph_count, request_count);
        assert_eq!(batch.report.unique_glyph_count, 1);
        assert_eq!(
            batch.report.duplicate_glyph_count,
            request_count.saturating_sub(1)
        );
        assert_eq!(batch.report.generated_glyph_count, 1);
        assert_eq!(batch.report.failed_glyph_count, 0);
    }
    assert_eq!(context.report().face_parse_count, 1);
    assert_eq!(context.report().source_hash_count, 1);
}

#[test]
fn text_sdf_generation_batch_is_identical_across_worker_counts() {
    let bytes = Arc::<[u8]>::from(fixture_bytes());
    let face = Face::parse(bytes.as_ref(), 0).expect("fixture face");
    let glyph_ids = ['g', 'A', 'M', 'A']
        .into_iter()
        .map(|glyph| face.glyph_index(glyph).expect("fixture glyph").0)
        .collect::<Vec<_>>();
    let context = SdfGenerationSourceContext::new(
        SdfGenerationSourceHandle::new(17),
        bytes,
        0,
        Arc::new(crate::text::VariationCoords::default()),
    )
    .expect("parsed generation source");
    let one_worker = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let four_workers = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(4));
    let params = SdfBakeParams::for_mode(SdfMode::Mtsdf);

    let serial = context.generate_batch_with_pool(&one_worker, params, &glyph_ids);
    let parallel = context.generate_batch_with_pool(&four_workers, params, &glyph_ids);

    assert_eq!(serial, parallel);
    assert_eq!(serial.report.requested_glyph_count, 4);
    assert_eq!(serial.report.unique_glyph_count, 3);
    assert_eq!(serial.report.duplicate_glyph_count, 1);
}

#[test]
fn text_msdf_dynamic_generation_reports_missing_outline() {
    let bytes = fixture_bytes();
    let space = glyph_id(&bytes, ' ');

    assert_eq!(
        generate_distance_field_glyph(&bytes, 0, space, SdfBakeParams::for_mode(SdfMode::Mtsdf),),
        Err(SdfGlyphGenerationError::MissingGlyphOutline(space))
    );
    assert_eq!(
        generate_distance_field_glyph(&bytes, 99, space, SdfBakeParams::for_mode(SdfMode::Msdf),),
        Err(SdfGlyphGenerationError::InvalidFaceIndex(99))
    );
}

#[test]
fn packaged_runtime_last_resort_notdef_has_a_real_sdf_outline() {
    let bytes = include_bytes!("../../../../assets/fonts/ZirconDefaultComposite-subset.ttc");

    let glyph = generate_distance_field_glyph(bytes, 0, 0, SdfBakeParams::for_mode(SdfMode::Sdf))
        .expect("the packaged last-resort notdef glyph must have a rasterizable outline");

    assert!(glyph.size.x > 0);
    assert!(glyph.size.y > 0);
    assert!(glyph.pixels.iter().any(|sample| *sample > 127));
}

#[test]
fn text_msdf_mtsdf_true_distance_channel_is_monotonic_across_an_edge() {
    let bytes = fixture_bytes();
    let glyph = generate_distance_field_glyph(
        &bytes,
        0,
        glyph_id(&bytes, 'A'),
        SdfBakeParams::for_mode(SdfMode::Mtsdf),
    )
    .unwrap();
    let alpha = glyph
        .pixels
        .chunks_exact(4)
        .map(|pixel| pixel[3])
        .collect::<Vec<_>>();

    assert!(alpha.iter().copied().min().unwrap() < 96);
    assert!(alpha.iter().copied().max().unwrap() > 160);
    assert!(has_monotonic_edge_ramp(
        &alpha,
        glyph.size.x as usize,
        glyph.size.y as usize
    ));
    assert!(glyph.pixels.chunks_exact(4).any(|pixel| {
        let median = median_u8(pixel[0], pixel[1], pixel[2]);
        median.abs_diff(pixel[3]) >= 4
    }));
}

#[test]
fn text_msdf_preserves_sharp_corners() {
    let bytes = fixture_bytes();
    let glyph_id = glyph_id(&bytes, 'A');
    let sdf =
        generate_distance_field_glyph(&bytes, 0, glyph_id, SdfBakeParams::for_mode(SdfMode::Sdf))
            .unwrap();
    let msdf =
        generate_distance_field_glyph(&bytes, 0, glyph_id, SdfBakeParams::for_mode(SdfMode::Msdf))
            .unwrap();

    assert_eq!(sdf.size, msdf.size);
    let expected_tip_y = SdfBakeParams::default().spread_px_f32().floor() as usize;
    let sdf_tip_error = top_inside_row(&sdf)
        .expect("SDF A pixels")
        .abs_diff(expected_tip_y);
    let msdf_tip_error = top_inside_row(&msdf)
        .expect("MSDF A pixels")
        .abs_diff(expected_tip_y);

    assert!(
        msdf_tip_error <= sdf_tip_error,
        "MSDF apex must be at least as faithful as SDF: msdf={msdf_tip_error}, sdf={sdf_tip_error}"
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_msdf_dynamic_generation_applies_real_variable_width_axis() {
    let bytes =
        std::fs::read(r"C:\Windows\Fonts\bahnschrift.ttf").expect("Windows variable-font fixture");
    let face = Face::parse(&bytes, 0).expect("Bahnschrift face");
    let axis = face
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let axis_tag = u32::from_be_bytes(axis.tag.to_bytes());
    let glyph_id = glyph_id(&bytes, 'W');
    let params = SdfBakeParams::for_mode(SdfMode::Msdf);

    let narrow = generate_distance_field_glyph_with_variations(
        &bytes,
        0,
        glyph_id,
        params,
        &crate::text::VariationCoords(vec![(axis_tag, axis.min_value)]),
    )
    .expect("narrow variable glyph");
    let wide = generate_distance_field_glyph_with_variations(
        &bytes,
        0,
        glyph_id,
        params,
        &crate::text::VariationCoords(vec![(axis_tag, axis.max_value)]),
    )
    .expect("wide variable glyph");

    assert!(wide.advance > narrow.advance);
    assert_ne!(wide.pixels, narrow.pixels);
}

fn top_inside_row(glyph: &SdfGlyphData) -> Option<usize> {
    let width = glyph.size.x as usize;
    match glyph.mode {
        SdfMode::Sdf => glyph
            .pixels
            .chunks(width)
            .position(|row| row.iter().any(|sample| *sample > 127)),
        SdfMode::Msdf | SdfMode::Mtsdf => glyph.pixels.chunks(width * 4).position(|row| {
            row.chunks_exact(4)
                .any(|pixel| median_u8(pixel[0], pixel[1], pixel[2]) > 127)
        }),
    }
}

fn median_u8(red: u8, green: u8, blue: u8) -> u8 {
    red.max(green.min(blue)).min(green.max(blue))
}

fn has_monotonic_edge_ramp(alpha: &[u8], width: usize, height: usize) -> bool {
    let crosses_edge = |samples: [u8; 3]| {
        let increasing = samples[0] < samples[1] && samples[1] < samples[2];
        let decreasing = samples[0] > samples[1] && samples[1] > samples[2];
        (increasing || decreasing)
            && samples.iter().copied().min().unwrap() < 128
            && samples.iter().copied().max().unwrap() > 128
    };
    for y in 0..height {
        for x in 0..width.saturating_sub(2) {
            let offset = y * width + x;
            if crosses_edge([alpha[offset], alpha[offset + 1], alpha[offset + 2]]) {
                return true;
            }
        }
    }
    for x in 0..width {
        for y in 0..height.saturating_sub(2) {
            let offset = y * width + x;
            if crosses_edge([
                alpha[offset],
                alpha[offset + width],
                alpha[offset + width * 2],
            ]) {
                return true;
            }
        }
    }
    false
}
