use std::sync::Arc;

use super::super::EditorCommandPaletteCatalog;
use super::EditorCommandPaletteLocaleProjection;
use crate::core::commands::{CommandEvalCtx, EditorCommandDescriptor};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{EditorI18nService, EditorLocale};

const ACTUAL_SEED_COUNT: usize = 4_096;

#[test]
fn optimization_batch_20260919_editor812_palette_posting_capacity_source_contract() {
    let source = include_str!("../locale_projection.rs");
    assert!(source.contains("let mut posting_counts = [0usize; 256];"));
    assert!(source.contains("let mut search_postings = posting_counts.map(Vec::with_capacity);"));
    assert!(source.contains("search_postings[byte].push(index);"));
    assert!(!source.contains("let mut search_postings: [Vec<usize>; 256]"));

    let i18n = EditorI18nService::default();
    let locale = EditorLocale::parse("en").unwrap();
    for seed_count in [0, 1, ACTUAL_SEED_COUNT] {
        let catalog = fixture_catalog(seed_count);
        let projection =
            EditorCommandPaletteLocaleProjection::build(&i18n, &locale, &catalog.seeds);
        let legacy = collect_legacy_postings(&projection.search_documents);
        assert_eq!(projection.entries.len(), seed_count);
        assert_eq!(projection.search_documents.len(), seed_count);
        assert_actual_posting_capacity(&projection, &legacy);

        if seed_count == ACTUAL_SEED_COUNT {
            assert_representative_buckets(&projection);
            let window = catalog.query_window(
                &i18n,
                &locale,
                &CommandEvalCtx::interactive(),
                "zanzibar",
                0,
                1,
            );
            assert_eq!(window.total_match_count(), 1);
            assert_eq!(window.metrics().visited_entries, 1);
            assert_eq!(window.metrics().enablement_evaluations, 1);
            assert_eq!(
                window.entries().next().unwrap().id,
                "fixture.palette.command_4095"
            );
        }
    }
}

// Historical dense-bucket geometry only; this workload is not a UTF-8 product build.
#[test]
#[ignore = "Windows-native release performance evidence"]
fn optimization_batch_20260919_editor812_palette_posting_capacity_bench() {
    const SEED_COUNT: usize = 4_096;
    const BYTE_BUCKETS: usize = 256;
    let mut legacy_growth_events = 0;
    let mut capacity = 0;
    for length in 1..=SEED_COUNT {
        if length > capacity {
            capacity = if capacity == 0 { 4 } else { capacity * 2 };
            legacy_growth_events += 1;
        }
    }
    println!(
        "EDITOR812_PALETTE_LOCALE_POSTING_CAPACITY_BENCH_V1 seeds={} buckets={} legacy_growth_events={} optimized_growth_events=0 workload=dense_bucket_geometric_model",
        SEED_COUNT,
        BYTE_BUCKETS,
        legacy_growth_events * BYTE_BUCKETS,
    );
    assert_eq!(legacy_growth_events * BYTE_BUCKETS, 2_816);
}

#[test]
#[ignore = "Windows-native actual locale posting allocation evidence"]
fn optimization_batch_20261013_editor08_palette_locale_posting_actual_build_evidence_v2() {
    let catalog = fixture_catalog(ACTUAL_SEED_COUNT);
    let i18n = EditorI18nService::default();
    let locale = EditorLocale::parse("en").unwrap();
    let projection = EditorCommandPaletteLocaleProjection::build(&i18n, &locale, &catalog.seeds);
    let legacy = collect_legacy_postings(&projection.search_documents);
    assert_eq!(projection.entries.len(), ACTUAL_SEED_COUNT);
    assert_eq!(projection.search_documents.len(), ACTUAL_SEED_COUNT);
    assert_actual_posting_capacity(&projection, &legacy);
    assert_representative_buckets(&projection);

    let legacy_growth_events: usize = legacy.growth_events.iter().sum();
    let optimized_growth_events: usize = projection.posting_capacity.growth_events.iter().sum();
    let initial_posting_capacity: usize =
        projection.posting_capacity.initial_capacities.iter().sum();
    let final_posting_capacity: usize = projection.posting_capacity.final_capacities.iter().sum();
    let populated_buckets = projection
        .search_postings
        .iter()
        .filter(|posting| !posting.is_empty())
        .count();
    let posting_indices: usize = projection
        .search_postings
        .iter()
        .map(|posting| posting.len())
        .sum();
    println!(
        "EDITOR812_PALETTE_LOCALE_POSTING_ACTUAL_BUILD_BENCH_V2 seeds={ACTUAL_SEED_COUNT} buckets=256 populated_buckets={populated_buckets} posting_indices={posting_indices} initial_posting_capacity={initial_posting_capacity} final_posting_capacity={final_posting_capacity} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} growth_measurement=push_capacity_transition workload=valid_utf8_locale_build"
    );
    assert!(legacy_growth_events > optimized_growth_events);
}

fn fixture_catalog(seed_count: usize) -> Arc<EditorCommandPaletteCatalog> {
    let descriptors = (0..seed_count)
        .map(|index| {
            let id = EditorOperationPath::parse(format!("fixture.palette.command_{index:04}"))
                .expect("the fixture uses canonical three-segment command IDs");
            let unicode_keyword = match index % 3 {
                0 => "ÉCOLE",
                1 => "ΟΣ",
                _ => "打开工程",
            };
            EditorCommandDescriptor::operation(id).with_keywords([
                unicode_keyword,
                "repeat repeat repeat",
                if index + 1 == seed_count {
                    "zanzibar"
                } else {
                    "common"
                },
            ])
        })
        .collect::<Vec<_>>();
    Arc::new(EditorCommandPaletteCatalog::from_descriptors(
        7,
        descriptors.iter(),
    ))
}

struct LegacyPostings {
    postings: [Vec<usize>; 256],
    growth_events: [usize; 256],
}

fn collect_legacy_postings(documents: &[Box<str>]) -> LegacyPostings {
    let mut postings: [Vec<usize>; 256] = std::array::from_fn(|_| Vec::new());
    let mut growth_events = [0; 256];
    for (index, document) in documents.iter().enumerate() {
        let mut indexed_bytes = [false; 256];
        for byte in document.bytes() {
            indexed_bytes[usize::from(byte)] = true;
        }
        for (byte, present) in indexed_bytes.into_iter().enumerate() {
            if present {
                let previous_capacity = postings[byte].capacity();
                postings[byte].push(index);
                if postings[byte].capacity() != previous_capacity {
                    growth_events[byte] += 1;
                }
            }
        }
    }
    LegacyPostings {
        postings,
        growth_events,
    }
}

fn assert_actual_posting_capacity(
    projection: &EditorCommandPaletteLocaleProjection,
    legacy: &LegacyPostings,
) {
    for (byte, expected) in legacy.postings.iter().enumerate() {
        let actual = &projection.search_postings[byte];
        assert_eq!(actual.as_ref(), expected.as_slice(), "bucket={byte}");
        assert_eq!(
            projection.posting_capacity.initial_capacities[byte],
            actual.len(),
            "initial bucket={byte}"
        );
        assert_eq!(
            projection.posting_capacity.final_capacities[byte],
            actual.len(),
            "final bucket={byte}"
        );
        assert_eq!(
            projection.posting_capacity.growth_events[byte], 0,
            "bucket={byte}"
        );
        assert!(
            actual.windows(2).all(|pair| pair[0] < pair[1]),
            "bucket={byte}"
        );
    }
}

fn assert_representative_buckets(projection: &EditorCommandPaletteLocaleProjection) {
    assert!(projection
        .search_documents
        .iter()
        .any(|document| !document.is_ascii()));
    assert!(projection.search_postings[usize::from(u8::MAX)].is_empty());
    assert_eq!(
        projection.search_postings[usize::from(b'z')].as_ref(),
        &[ACTUAL_SEED_COUNT - 1]
    );
    assert_eq!(
        projection.search_postings[usize::from(b'e')].as_ref(),
        (0..ACTUAL_SEED_COUNT).collect::<Vec<_>>().as_slice()
    );
}
