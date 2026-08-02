use std::path::Path;

use crate::core::framework::text::TextDirection;
use crate::text::{TextRange, TextStyle};

use crate::text::font::FontDatabase;
#[cfg(target_os = "windows")]
use crate::text::shaping::shape_horizontal_line;
#[cfg(target_os = "windows")]
use crate::text::VariationCoords;

use super::backend::shape_horizontal_run;
#[test]
fn text_horizontal_backend_shapes_static_face_without_language_override() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(&source, Some("Fira Sans Empty Locale Test"), 0)
        .expect("register tracked static font");

    let shaped = shape_horizontal_run(
        &database,
        face,
        None,
        "static text",
        TextDirection::LeftToRight,
        "Latn",
        Some(""),
        &[],
        true,
        400,
        18.0,
    )
    .expect("static face shapes through the canonical backend");

    assert!(!shaped.glyphs.is_empty());
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
        TextDirection::LeftToRight,
        "Latn",
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
        TextDirection::LeftToRight,
        "Latn",
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
        TextDirection::LeftToRight,
        "Cyrl",
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
        TextDirection::LeftToRight,
        "Cyrl",
        Some("sr"),
        &[],
        true,
        400,
        32.0,
    )
    .expect("shape Serbian localized form");
    let inferred_serbian = shape_horizontal_run(
        &database,
        face,
        None,
        "б",
        TextDirection::LeftToRight,
        "Zyyy",
        Some("sr"),
        &[],
        true,
        400,
        32.0,
    )
    .expect("shape Serbian localized form with an unresolved script");

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
    let inferred_serbian_ids = inferred_serbian
        .glyphs
        .iter()
        .map(|glyph| glyph.glyph_id)
        .collect::<Vec<_>>();
    assert_ne!(
        russian_ids, serbian_ids,
        "Calibri must select distinct Russian and Serbian locl glyphs for Cyrillic be"
    );
    assert_eq!(
        inferred_serbian_ids, serbian_ids,
        "Common script must retain RustyBuzz inference instead of suppressing Cyrillic locl"
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_horizontal_rustybuzz_backend_preserves_serbian_locl_in_mixed_script_text() {
    let source = Path::new(r"C:\Windows\Fonts\calibri.ttf");
    assert!(source.is_file(), "Windows locl font fixture is missing");
    let style = TextStyle {
        font_family: Some("Calibri".to_string()),
        language: Some("sr".to_string()),
        font_size: 32.0,
        line_height: 38.0,
        ..TextStyle::default()
    };
    let mixed_text = "aб";
    let mixed = shape_horizontal_line(
        mixed_text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: mixed_text.len(),
        },
    );
    let isolated_text = "б";
    let isolated = shape_horizontal_line(
        isolated_text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: isolated_text.len(),
        },
    );

    let mixed_cyrillic = mixed
        .lines
        .first()
        .expect("mixed shaped line")
        .glyphs
        .iter()
        .find(|glyph| glyph.source_range.start == 1)
        .expect("mixed Cyrillic glyph");
    let isolated_cyrillic = isolated
        .lines
        .first()
        .expect("isolated shaped line")
        .glyphs
        .first()
        .expect("isolated Cyrillic glyph");

    assert_eq!(mixed_cyrillic.script.iso15924, "Cyrl");
    assert_eq!(mixed_cyrillic.font_id, isolated_cyrillic.font_id);
    assert_eq!(
        mixed_cyrillic.glyph_id, isolated_cyrillic.glyph_id,
        "a Latin neighbor must not cause the Serbian Cyrillic cluster to be reshaped as Latin"
    );
}
