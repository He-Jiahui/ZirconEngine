use std::path::Path;

use ttf_parser::Face;

use super::*;

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
    assert!(first
        .pixels
        .chunks_exact(4)
        .any(|pixel| { pixel[0] != pixel[1] || pixel[1] != pixel[2] }));
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
