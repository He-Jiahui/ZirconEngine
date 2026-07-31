use super::*;
use crate::text::font::TextDecorationMetricsCache;

#[test]
fn text_font_face_metadata_is_built_once_across_hot_consumers() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(&source_path, Some("Metadata Once"), 0)
        .expect("font fixture should register");

    assert_eq!(database.face_metadata_build_count(), 1);
    let source_identity = database.face_source_identity(face).unwrap();
    assert_ne!(source_identity, [0; 16]);
    let vertical_metrics = database
        .vertical_metrics(face, 16.0)
        .expect("registered face should expose shared metrics");
    for index in 0..10_000_u32 {
        let _ = database
            .effective_instance_variations_shared(face, None, 400)
            .expect("effective variations should resolve");
        let _ = vertical_metrics.glyph_advance_px(index % 256);
    }
    let mut decorations = TextDecorationMetricsCache::default();
    for display_px in [12.0, 14.0, 16.0, 24.0] {
        let _ = decorations.resolve(&database, face, display_px);
    }

    assert_eq!(
        database.face_metadata_build_count(),
        1,
        "variation, vertical-metric, and decoration consumers must share one owned face metadata artifact"
    );
    assert_eq!(
        database.face_source_identity(face).unwrap(),
        source_identity
    );
    let cache = database.effective_instance_cache_report();
    assert_eq!(cache.misses, 1);
    assert_eq!(cache.hits, 9_999);
    assert_eq!(cache.entry_count, 1);
}

#[test]
fn text_font_effective_variation_cache_is_bounded_and_reports_eviction() {
    let mut database = FontDatabase::default();
    let face = database
        .register_stored_face(
            FontFaceDescriptor::regular("Bounded Effective Variations"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .expect("test face should register");

    for font_weight in 0..300_u16 {
        let _ = database
            .effective_instance_variations_shared(face, None, font_weight)
            .expect("effective variation should resolve");
    }

    let cache = database.effective_instance_cache_report();
    assert!(cache.entry_count <= 256);
    assert!(cache.approximate_bytes <= 256 * 1024);
    assert!(cache.eviction_count > 0);
}

#[test]
#[ignore = "managed Text01 metadata scale evidence"]
fn text_font_metadata_scale_reports_one_build_and_latency_percentiles() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    for glyph_count in [1_usize, 100, 10_000] {
        let mut database = FontDatabase::default();
        let face = database
            .register_font_file(&source_path, Some("Metadata Scale"), 0)
            .expect("font fixture should register");
        let clone = database.clone();
        assert!(Arc::ptr_eq(
            &database.faces[0].metadata,
            &clone.faces[0].metadata
        ));
        let database_bytes = database.face_bytes(face).unwrap();
        let worker_bytes = clone.face_bytes(face).unwrap();
        let worker_duplicate_bytes = if Arc::ptr_eq(&database_bytes, &worker_bytes) {
            0
        } else {
            worker_bytes.len()
        };
        assert_eq!(worker_duplicate_bytes, 0);
        let vertical_metrics = database
            .vertical_metrics(face, 16.0)
            .expect("registered face should expose shared metrics");
        let mut samples = Vec::with_capacity(glyph_count);
        for glyph_id in 0..glyph_count {
            let started = std::time::Instant::now();
            let _ = database
                .effective_instance_variations_shared(face, None, 400)
                .expect("effective variations should resolve");
            let _ = vertical_metrics.glyph_advance_px((glyph_id % 256) as u32);
            samples.push(started.elapsed().as_nanos());
        }
        let cache = database.effective_instance_cache_report();
        assert_eq!(database.face_metadata_build_count(), 1);
        assert_eq!(cache.misses, 1);
        assert_eq!(cache.hits, glyph_count.saturating_sub(1) as u64);
        assert_eq!(cache.entry_count, 1);
        eprintln!(
            "text_font_metadata_scale glyphs={glyph_count} parse_builds={} cache_bytes={} worker_duplicate_bytes={worker_duplicate_bytes} p50_ns={} p95_ns={}",
            database.face_metadata_build_count(),
            cache.approximate_bytes,
            percentile_ns(&mut samples, 50),
            percentile_ns(&mut samples, 95),
        );
    }
}

fn percentile_ns(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let index = samples.len().saturating_sub(1).saturating_mul(percentile) / 100;
    samples[index]
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_effective_variation_cache_keys_explicit_instance_and_weight() {
    let source = Path::new(r"C:\Windows\Fonts\bahnschrift.ttf");
    let bytes = std::fs::read(source).expect("Windows variable-font fixture");
    let parsed = ttf_parser::Face::parse(&bytes, 0).expect("parse Bahnschrift test fixture");
    let width = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let tag = u32::from_be_bytes(width.tag.to_bytes());
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(source, Some("Explicit Instance Cache"), 0)
        .expect("variable font should register");
    let narrow = database
        .instance(face, &VariationCoords(vec![(tag, width.min_value)]))
        .expect("narrow instance should resolve");
    let wide = database
        .instance(face, &VariationCoords(vec![(tag, width.max_value)]))
        .expect("wide instance should resolve");

    let narrow_variations = database
        .effective_instance_variations(face, Some(narrow), 700)
        .expect("narrow effective variations should resolve");
    let wide_variations = database
        .effective_instance_variations(face, Some(wide), 700)
        .expect("wide effective variations should resolve");
    assert_ne!(narrow_variations, wide_variations);
    assert_eq!(
        database
            .effective_instance_variations(face, Some(narrow), 700)
            .unwrap(),
        narrow_variations
    );
    assert_eq!(
        database
            .effective_instance_variations(face, Some(wide), 700)
            .unwrap(),
        wide_variations
    );

    let cache = database.effective_instance_cache_report();
    assert_eq!(cache.misses, 2);
    assert_eq!(cache.hits, 2);
    assert_eq!(cache.entry_count, 2);
}
