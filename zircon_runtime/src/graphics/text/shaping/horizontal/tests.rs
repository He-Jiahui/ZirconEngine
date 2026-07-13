use std::path::Path;

use zircon_runtime_interface::ui::surface::UiTextDirection;

use crate::core::framework::render::TextOrientation;
#[cfg(target_os = "windows")]
use crate::core::framework::render::VariationCoords;
use crate::graphics::text::font::FontDatabase;

use super::backend::shape_horizontal_run;
use super::projection::should_apply_horizontal_backend;

#[test]
fn text_horizontal_backend_skips_vertical_requests() {
    assert!(should_apply_horizontal_backend(TextOrientation::Horizontal));
    assert!(!should_apply_horizontal_backend(TextOrientation::Vertical));
}

#[test]
fn text_horizontal_backend_skips_static_face_for_empty_language_tag() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(&source, Some("Fira Sans Empty Locale Test"), 0)
        .expect("register tracked static font");

    assert!(shape_horizontal_run(
        &database,
        face,
        None,
        "static text",
        UiTextDirection::LeftToRight,
        Some(""),
        &[],
        true,
        400,
        18.0,
    )
    .is_none());
}

#[cfg(target_os = "windows")]
#[test]
fn text_horizontal_rustybuzz_backend_applies_real_variable_width_axis() {
    let source = Path::new(r"C:\Windows\Fonts\bahnschrift.ttf");
    assert!(source.is_file(), "Windows variable-font fixture is missing");
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(source, Some("Bahnschrift Variable Test"), 0)
        .expect("register Windows variable font");
    let bytes = database.face_bytes(face).expect("variable font bytes");
    let parsed = ttf_parser::Face::parse(bytes.as_ref(), 0).expect("parse variable font");
    let axis = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    assert!(axis.min_value < axis.max_value);
    let tag = u32::from_be_bytes(axis.tag.to_bytes());
    let narrow = database
        .instance(face, &VariationCoords(vec![(tag, axis.min_value)]))
        .expect("narrow instance");
    let wide = database
        .instance(face, &VariationCoords(vec![(tag, axis.max_value)]))
        .expect("wide instance");

    let narrow_run = shape_horizontal_run(
        &database,
        face,
        Some(narrow),
        "VARIABLE WIDTH",
        UiTextDirection::LeftToRight,
        Some("en"),
        &[],
        true,
        400,
        32.0,
    )
    .expect("shape narrow instance");
    let wide_run = shape_horizontal_run(
        &database,
        face,
        Some(wide),
        "VARIABLE WIDTH",
        UiTextDirection::LeftToRight,
        Some("en"),
        &[],
        true,
        400,
        32.0,
    )
    .expect("shape wide instance");

    let narrow_advance = narrow_run
        .glyphs
        .iter()
        .map(|glyph| glyph.advance.abs())
        .sum::<f32>();
    let wide_advance = wide_run
        .glyphs
        .iter()
        .map(|glyph| glyph.advance.abs())
        .sum::<f32>();
    assert!(
        wide_advance > narrow_advance,
        "width axis must change shaped advance: narrow={narrow_advance}, wide={wide_advance}"
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_horizontal_rustybuzz_backend_applies_real_per_run_locl_language() {
    let source = Path::new(r"C:\Windows\Fonts\calibri.ttf");
    assert!(source.is_file(), "Windows locl font fixture is missing");
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(source, Some("Calibri Locale Test"), 0)
        .expect("register Windows locale font");

    let russian = shape_horizontal_run(
        &database,
        face,
        None,
        "б",
        UiTextDirection::LeftToRight,
        Some("ru"),
        &[],
        true,
        400,
        32.0,
    )
    .expect("shape Russian localized form");
    let serbian = shape_horizontal_run(
        &database,
        face,
        None,
        "б",
        UiTextDirection::LeftToRight,
        Some("sr"),
        &[],
        true,
        400,
        32.0,
    )
    .expect("shape Serbian localized form");

    let russian_ids = russian
        .glyphs
        .iter()
        .map(|glyph| glyph.glyph_id)
        .collect::<Vec<_>>();
    let serbian_ids = serbian
        .glyphs
        .iter()
        .map(|glyph| glyph.glyph_id)
        .collect::<Vec<_>>();
    assert_ne!(
        russian_ids, serbian_ids,
        "Calibri must select distinct Russian and Serbian locl glyphs for Cyrillic be"
    );
}
