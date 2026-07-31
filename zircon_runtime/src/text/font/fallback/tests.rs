use std::path::Path;
use std::sync::Arc;

use super::*;
use crate::text::{FontFaceDescriptor, FontFamilyName, SubFontRange};

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
fn text_fallback_emoji_zwj_controls_do_not_require_standalone_cmap_glyphs() {
    let mut database = FontDatabase::default();
    let primary = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let _partial = database
        .register_test_face_with_coverage(
            FontFaceDescriptor::regular("Emoji Partial"),
            &['\u{1F469}'],
        )
        .unwrap();
    let complete = database
        .register_test_face_with_coverage(
            FontFaceDescriptor::regular("Emoji Complete"),
            &['\u{1F469}', '\u{1F4BB}'],
        )
        .unwrap();
    let script = FontScript::Other('\u{1F469}' as u32);
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary"),
        sub_fonts: vec![
            SubFontRange {
                family: FontFamilyName::from("Emoji Partial"),
                scripts: vec![script],
                ranges: Vec::new(),
                cultures: Vec::new(),
            },
            SubFontRange {
                family: FontFamilyName::from("Emoji Complete"),
                scripts: vec![script],
                ranges: Vec::new(),
                cultures: Vec::new(),
            },
        ],
    };
    let query = FontQuery::single_family("Primary");
    let mut resolver = FallbackResolver::new(&database, &query, Some(&composite), None);

    let resolution = resolver.resolve(
        primary,
        script,
        &['\u{1F469}', '\u{200D}', '\u{1F4BB}', '\u{FE0F}'],
    );

    assert_eq!(resolution.face, complete);
    assert!(!resolution.missing);
    assert_eq!(resolution.source, FallbackResolutionSource::Fallback);
}

#[test]
fn text_fallback_emoji_tag_controls_do_not_require_standalone_cmap_glyphs() {
    let mut database = FontDatabase::default();
    let primary = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let emoji = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Emoji Tags"), &['\u{1F3F4}'])
        .unwrap();
    let script = FontScript::Other('\u{1F3F4}' as u32);
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Emoji Tags"),
            scripts: vec![script],
            ranges: Vec::new(),
            cultures: Vec::new(),
        }],
    };
    let query = FontQuery::single_family("Primary");
    let mut resolver = FallbackResolver::new(&database, &query, Some(&composite), None);

    let resolution = resolver.resolve(
        primary,
        script,
        &[
            '\u{1F3F4}',
            '\u{E0067}',
            '\u{E0062}',
            '\u{E0065}',
            '\u{E006E}',
            '\u{E0067}',
            '\u{E007F}',
        ],
    );

    assert_eq!(resolution.face, emoji);
    assert!(!resolution.missing);
    assert_eq!(resolution.source, FallbackResolutionSource::Fallback);
}

#[test]
fn text_fallback_rtl_cluster_routes_by_script_and_preserves_full_coverage() {
    let mut database = FontDatabase::default();
    let primary = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let arabic = database
        .register_test_face_with_coverage(
            FontFaceDescriptor::regular("Arabic UI"),
            &['\u{0645}', '\u{0631}', '\u{062D}', '\u{0628}', '\u{0627}'],
        )
        .unwrap();
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Arabic UI"),
            scripts: vec![FontScript::Arabic],
            ranges: Vec::new(),
            cultures: Vec::new(),
        }],
    };
    let query = FontQuery::single_family("Primary");
    let mut resolver = FallbackResolver::new(&database, &query, Some(&composite), Some("ar-EG"));

    let resolution = resolver.resolve(
        primary,
        FontScript::Arabic,
        &['\u{0645}', '\u{0631}', '\u{062D}', '\u{0628}', '\u{0627}'],
    );

    assert_eq!(resolution.face, arabic);
    assert!(!resolution.missing);
    assert_eq!(resolution.source, FallbackResolutionSource::Fallback);
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
fn text_fallback_missing_log_uses_indexed_deduplication_without_losing_order() {
    let source = include_str!("../fallback.rs");
    assert!(
        source.contains("entry_by_key: HashMap<MissingGlyphKey, usize>"),
        "missing-glyph deduplication must keep an indexed owner beside the ordered report"
    );
    assert!(
        !source.contains("entries.iter_mut().find("),
        "a missing-glyph diagnostic must not linearly scan every retained entry"
    );

    let face = FontFaceId(7);
    let mut log = MissingGlyphLog::with_capacity(2);
    let first = MissingGlyphDiagnostic {
        face,
        script: FontScript::Latin,
        codepoint: 'A' as u32,
        reason: MissingGlyphReason::MissingGlyph,
        occurrence_count: 1,
    };
    log.push(first.clone());
    log.push(MissingGlyphDiagnostic {
        codepoint: 'B' as u32,
        ..first.clone()
    });
    for _ in 0..10_000 {
        log.push(first.clone());
    }
    log.push(MissingGlyphDiagnostic {
        codepoint: 'C' as u32,
        ..first
    });

    assert_eq!(log.entries().len(), 2);
    assert_eq!(log.entries()[0].codepoint, 'A' as u32);
    assert_eq!(log.entries()[0].occurrence_count, 10_001);
    assert_eq!(log.entries()[1].codepoint, 'B' as u32);
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

#[test]
fn text_fallback_reuses_generation_owned_resolution_for_repeated_cluster() {
    let mut database = FontDatabase::default();
    let primary = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let fallback = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("CJK"), &['中'])
        .unwrap();
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("CJK"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x4E00, 0x9FFF)],
            cultures: Vec::new(),
        }],
    };
    let query = FontQuery::single_family("Primary");
    database.set_project_composite_font(Some(composite));

    for _ in 0..10_000 {
        let mut resolver = FallbackResolver::new(&database, &query, None, Some("zh-CN"));
        let resolution = resolver.resolve(primary, FontScript::Han, &['中']);
        assert_eq!(resolution.face, fallback);
        assert!(!resolution.missing);
    }

    let report = database.fallback_cache_report();
    assert_eq!(report.resolution_misses, 1);
    assert_eq!(report.resolution_hits, 9_999);
    assert_eq!(report.candidate_misses, 1);
    assert_eq!(report.composite_compile_count, 1);
    assert_eq!(report.composite_entry_count, 1);
    assert_eq!(report.family_sort_count, 2);
    assert_eq!(report.family_visit_count, 2);
    assert_eq!(report.face_visit_count, 2);
    assert_eq!(report.normalization_allocation_count, 0);
    assert_eq!(report.resolution_entry_count, 1);
    assert!(report.approximate_bytes > 0);
}

#[test]
fn text_fallback_cache_keeps_full_cluster_identity() {
    let mut database = FontDatabase::default();
    let primary = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let marks = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Marks"), &['A', '\u{0301}'])
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

    assert_eq!(
        resolver.resolve(primary, FontScript::Latin, &['A']).face,
        primary
    );
    assert_eq!(
        resolver
            .resolve(primary, FontScript::Latin, &['A', '\u{0301}'])
            .face,
        marks
    );
    assert_eq!(
        resolver
            .resolve(primary, FontScript::Latin, &['A', '\u{0301}', '\u{FE0F}'])
            .face,
        marks
    );

    let report = database.fallback_cache_report();
    assert_eq!(report.resolution_misses, 3);
    assert_eq!(report.resolution_entry_count, 3);
}

#[test]
fn text_fallback_cache_invalidates_when_font_generation_changes() {
    let mut database = FontDatabase::default();
    let primary = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("CJK"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x4E00, 0x9FFF)],
            cultures: Vec::new(),
        }],
    };
    let query = FontQuery::single_family("Primary");
    let first = FallbackResolver::new(&database, &query, Some(&composite), None)
        .resolve_codepoint(primary, '中');
    assert!(first.missing);

    let fallback = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("CJK"), &['中'])
        .unwrap();
    let second = FallbackResolver::new(&database, &query, Some(&composite), None)
        .resolve_codepoint(primary, '中');

    assert_eq!(second.face, fallback);
    assert!(!second.missing);
    let report = database.fallback_cache_report();
    assert_eq!(report.resolution_misses, 1);
    assert_eq!(report.resolution_hits, 0);
    assert_eq!(report.resolution_entry_count, 1);
    assert_eq!(report.composite_compile_count, 1);
    assert_eq!(report.composite_entry_count, 1);
}

#[test]
fn text_fallback_cache_isolated_after_cloned_database_generations_diverge() {
    let mut base = FontDatabase::default();
    let primary = base
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let mut han_generation = base.clone();
    let mut latin_generation = base;
    let han = han_generation
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Diverged"), &['中'])
        .unwrap();
    let latin = latin_generation
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Diverged"), &['B'])
        .unwrap();
    assert_eq!(
        han, latin,
        "diverged clones deliberately reuse one numeric face ID"
    );
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Diverged"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x4E00, 0x9FFF)],
            cultures: Vec::new(),
        }],
    };
    han_generation.set_project_composite_font(Some(composite.clone()));
    latin_generation.set_project_composite_font(Some(composite));
    let query = FontQuery::single_family("Primary");

    let han_resolution =
        FallbackResolver::new(&han_generation, &query, None, None).resolve_codepoint(primary, '中');
    assert_eq!(han_resolution.face, han);
    assert!(!han_resolution.missing);

    let latin_resolution = FallbackResolver::new(&latin_generation, &query, None, None)
        .resolve_codepoint(primary, '中');
    assert!(latin_resolution.missing);
    assert_ne!(latin_resolution.face, latin);
}

#[test]
fn text_fallback_cache_is_bounded_and_reports_eviction() {
    let mut database = FontDatabase::default();
    let primary = database
        .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
        .unwrap();
    let query = FontQuery::single_family("Primary");

    for offset in 0..1_100_u32 {
        let codepoint = char::from_u32(0x1000 + offset).unwrap();
        let mut resolver = FallbackResolver::new(&database, &query, None, None);
        let _ = resolver.resolve(primary, script_for_char(codepoint), &[codepoint]);
    }

    let report = database.fallback_cache_report();
    assert!(report.resolution_entry_count <= 1_024);
    assert!(report.candidate_entry_count <= 1_024);
    assert!(report.approximate_bytes <= 2 * 1024 * 1024);
    assert!(report.eviction_count > 0);
}

#[test]
#[ignore = "managed Text01 fallback scale evidence"]
fn text_fallback_scale_reports_stable_sort_visit_probe_and_latency_percentiles() {
    for family_count in [1_usize, 8, 64] {
        for cluster_count in [1_usize, 100, 10_000] {
            let mut database = FontDatabase::default();
            let primary = database
                .register_test_face_with_coverage(FontFaceDescriptor::regular("Primary"), &['A'])
                .unwrap();
            let mut sub_fonts = Vec::with_capacity(family_count);
            let mut expected = None;
            for family_index in 0..family_count {
                let family = FontFamilyName::from(format!("Fallback {family_index}"));
                let face = database
                    .register_test_face_with_coverage(
                        FontFaceDescriptor::regular(family.as_str()),
                        &['中'],
                    )
                    .unwrap();
                expected.get_or_insert(face);
                sub_fonts.push(SubFontRange {
                    family,
                    scripts: vec![FontScript::Han],
                    ranges: vec![(0x4E00, 0x9FFF)],
                    cultures: Vec::new(),
                });
            }
            database.set_project_composite_font(Some(CompositeFontDescriptor {
                default_family: FontFamilyName::from("Primary"),
                sub_fonts,
            }));
            let query = FontQuery::single_family("Primary");
            let mut samples = Vec::with_capacity(cluster_count);
            for _ in 0..cluster_count {
                let started = std::time::Instant::now();
                let resolution = FallbackResolver::new(&database, &query, None, Some("zh-CN"))
                    .resolve_codepoint(primary, '中');
                samples.push(started.elapsed().as_nanos());
                assert_eq!(resolution.face, expected.unwrap());
                assert!(!resolution.missing);
            }
            let report = database.fallback_cache_report();
            assert_eq!(report.composite_compile_count, 1);
            assert_eq!(report.candidate_misses, 1);
            assert_eq!(report.resolution_misses, 1);
            assert_eq!(
                report.resolution_hits,
                cluster_count.saturating_sub(1) as u64
            );
            assert_eq!(report.family_sort_count, (family_count + 1) as u64);
            assert_eq!(report.family_visit_count, (family_count + 1) as u64);
            assert_eq!(
                report.face_visit_count,
                (family_count + 1) as u64,
                "the first candidate build must inspect each registered family face once"
            );
            assert_eq!(
                report.normalization_allocation_count, 0,
                "fixed family/query identities must not allocate normalized strings"
            );
            assert!(report.coverage_probe_count >= (family_count + 2) as u64);
            assert!(report.approximate_bytes <= 2 * 1024 * 1024);
            eprintln!(
                "text_fallback_scale families={family_count} clusters={cluster_count} normalization_allocs={} family_visits={} face_visits={} sorts={} probes={} cache_bytes={} p50_ns={} p95_ns={}",
                report.normalization_allocation_count,
                report.family_visit_count,
                report.face_visit_count,
                report.family_sort_count,
                report.coverage_probe_count,
                report.approximate_bytes,
                fallback_percentile_ns(&mut samples, 50),
                fallback_percentile_ns(&mut samples, 95),
            );
        }
    }
}

fn fallback_percentile_ns(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let index = samples.len().saturating_sub(1).saturating_mul(percentile) / 100;
    samples[index]
}
