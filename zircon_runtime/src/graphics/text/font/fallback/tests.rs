use std::path::Path;
use std::sync::Arc;

use super::*;
use crate::core::framework::render::{FontFaceDescriptor, SubFontRange};

#[test]
fn text_fallback_primary_face_covers_all_codepoints() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let primary = database
        .register_font_file(&source_path, Some("Inter"), 0)
        .unwrap();
    let query = FontQuery::single_family("Inter");
    let mut resolver = FallbackResolver::new(&database, &query, None);

    let resolution = resolver.resolve(primary, FontScript::Latin, &['A', 'B']);

    assert_eq!(resolution.face, primary);
    assert!(!resolution.missing);
    assert_eq!(resolution.source, FallbackResolutionSource::Primary);
    assert!(resolver.diagnostics().entries().is_empty());
}

#[test]
fn text_fallback_cjk_resolves_to_cjk_font() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let primary = database
        .register_font_file(&source_path, Some("Inter"), 0)
        .unwrap();
    let cjk = database
        .register_test_face(
            FontFaceDescriptor::regular("Noto Sans CJK SC"),
            Arc::from([4_u8, 5, 6].as_slice()),
        )
        .unwrap();
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Inter"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Noto Sans CJK SC"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x4E00, 0x9FFF)],
        }],
    };
    let query = FontQuery::single_family("Inter");
    let mut resolver = FallbackResolver::new(&database, &query, Some(&composite));

    let resolution = resolver.resolve(primary, FontScript::Han, &['中', '文']);

    assert_eq!(resolution.face, cjk);
    assert!(!resolution.missing);
    assert_eq!(resolution.source, FallbackResolutionSource::Fallback);
    assert!(resolver.diagnostics().entries().is_empty());
}

#[test]
fn text_fallback_depth_limited() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let primary = database
        .register_font_file(&source_path, Some("Inter"), 0)
        .unwrap();
    let cjk = database
        .register_test_face(
            FontFaceDescriptor::regular("Noto Sans CJK SC"),
            Arc::from([7_u8, 8, 9].as_slice()),
        )
        .unwrap();
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Inter"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Noto Sans CJK SC"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x4E00, 0x9FFF)],
        }],
    };
    let query = FontQuery::single_family("Inter");
    let mut resolver = FallbackResolver::with_max_depth(&database, &query, Some(&composite), 0);

    let resolution = resolver.resolve(primary, FontScript::Han, &['界']);

    assert_ne!(resolution.face, cjk);
    assert_eq!(resolution.face, primary);
    assert!(resolution.missing);
    assert_eq!(
        resolution.source,
        FallbackResolutionSource::DepthLimitExceeded
    );
    assert_eq!(
        resolver.diagnostics().entries()[0].reason,
        MissingGlyphReason::DepthLimitExceeded
    );
}

#[test]
fn text_fallback_missing_codepoint_reports_diagnostic() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let primary = database
        .register_font_file(&source_path, Some("Inter"), 0)
        .unwrap();
    let query = FontQuery::single_family("Inter");
    let mut resolver = FallbackResolver::new(&database, &query, None);

    let resolution = resolver.resolve(primary, FontScript::Other(0x10FFFF), &['\u{10FFFF}']);

    assert_eq!(resolution.face, primary);
    assert!(resolution.missing);
    assert_eq!(resolution.source, FallbackResolutionSource::LastResort);
    assert_eq!(resolver.diagnostics().entries().len(), 1);
    assert_eq!(
        resolver.diagnostics().entries()[0],
        MissingGlyphDiagnostic {
            script: FontScript::Other(0x10FFFF),
            codepoints: vec![0x10FFFF],
            reason: MissingGlyphReason::MissingGlyph,
        }
    );
}
