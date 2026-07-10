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
    let mut resolver = FallbackResolver::new(&database, &query, None, None);

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
            cultures: Vec::new(),
        }],
    };
    let query = FontQuery::single_family("Inter");
    let mut resolver = FallbackResolver::new(&database, &query, Some(&composite), None);

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
            cultures: Vec::new(),
        }],
    };
    let query = FontQuery::single_family("Inter");
    let mut resolver =
        FallbackResolver::with_max_depth(&database, &query, Some(&composite), None, 0);

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
    let mut resolver = FallbackResolver::new(&database, &query, None, None);

    let resolution = resolver.resolve(primary, FontScript::Other(0x10FFFF), &['\u{10FFFF}']);

    assert_eq!(resolution.face, primary);
    assert!(resolution.missing);
    assert_eq!(resolution.source, FallbackResolutionSource::LastResort);
    assert_eq!(resolver.diagnostics().entries().len(), 1);
    assert_eq!(
        resolver.diagnostics().entries()[0],
        MissingGlyphDiagnostic {
            face: primary,
            script: FontScript::Other(0x10FFFF),
            codepoint: 0x10FFFF,
            reason: MissingGlyphReason::MissingGlyph,
            occurrence_count: 1,
        }
    );
}

#[test]
fn text_fallback_missing_log_deduplicates_and_bounds_entries() {
    let face = FontFaceId(7);
    let mut log = MissingGlyphLog::with_capacity(2);
    let diagnostic = MissingGlyphDiagnostic {
        face,
        script: FontScript::Latin,
        codepoint: 'A' as u32,
        reason: MissingGlyphReason::MissingGlyph,
        occurrence_count: 1,
    };

    log.push(diagnostic.clone());
    log.push(diagnostic);
    log.push(MissingGlyphDiagnostic {
        codepoint: 'B' as u32,
        ..MissingGlyphDiagnostic {
            face,
            script: FontScript::Latin,
            codepoint: 'A' as u32,
            reason: MissingGlyphReason::MissingGlyph,
            occurrence_count: 1,
        }
    });
    log.push(MissingGlyphDiagnostic {
        codepoint: 'C' as u32,
        ..MissingGlyphDiagnostic {
            face,
            script: FontScript::Latin,
            codepoint: 'A' as u32,
            reason: MissingGlyphReason::MissingGlyph,
            occurrence_count: 1,
        }
    });

    assert_eq!(log.entries().len(), 2);
    assert_eq!(log.entries()[0].occurrence_count, 2);
    assert!(log.overflowed());
    assert_eq!(log.dropped_count(), 1);
}

#[test]
fn text_fallback_partial_cluster_coverage_keeps_best_base_face() {
    let mut database = FontDatabase::default();
    let primary = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let fallback = database
        .register_test_face_with_coverage(
            FontFaceDescriptor::regular("Marks"),
            &['A', '\u{0300}', '\u{0301}'],
        )
        .unwrap();
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Marks"),
            scripts: vec![FontScript::Latin],
            ranges: Vec::new(),
            cultures: Vec::new(),
        }],
    };
    let query = FontQuery::single_family("Primary");
    let mut resolver = FallbackResolver::new(&database, &query, Some(&composite), None);

    let resolution = resolver.resolve(
        primary,
        FontScript::Latin,
        &['A', '\u{0300}', '\u{0301}', '\u{0302}'],
    );

    assert_eq!(resolution.face, fallback);
    assert!(resolution.missing);
    assert_eq!(resolution.source, FallbackResolutionSource::PartialCoverage);
    assert_eq!(resolver.diagnostics().entries().len(), 1);
    assert_eq!(resolver.diagnostics().entries()[0].face, fallback);
    assert_eq!(resolver.diagnostics().entries()[0].codepoint, 0x0302);
}
