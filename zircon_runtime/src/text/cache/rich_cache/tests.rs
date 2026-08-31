use std::sync::{Arc, Barrier, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::text::rich::{CompiledRichText, RichTextContentTrust, RichTextParserGeneration};
use crate::text::{RichParseResult, RichTextFormat};

use super::{CompiledRichTextCache, CompiledRichTextCacheOwner, RichTextCacheCounter};

const UNTRUSTED: RichTextContentTrust = RichTextContentTrust::Untrusted;

#[test]
fn compiled_rich_cache_reuses_exact_artifact_and_bounds_residency() {
    let mut cache = CompiledRichTextCache::new();
    cache.report.max_entries = 2;
    let generation = RichTextParserGeneration {
        parser_identity: 7,
        decorator_generation: 1,
        emoji_generation: 1,
    };
    let first = cache.lookup_or_insert(
        "[b]one[/b]",
        RichTextFormat::BbCodeV1,
        UNTRUSTED,
        generation,
    );
    record_compiled_cell(&mut cache, &first, RichTextFormat::BbCodeV1, generation);
    let repeated = cache.lookup_or_insert(
        "[b]one[/b]",
        RichTextFormat::BbCodeV1,
        UNTRUSTED,
        generation,
    );
    let second = cache.lookup_or_insert(
        "[b]two[/b]",
        RichTextFormat::BbCodeV1,
        UNTRUSTED,
        generation,
    );
    record_compiled_cell(&mut cache, &second, RichTextFormat::BbCodeV1, generation);
    let third = cache.lookup_or_insert(
        "[b]three[/b]",
        RichTextFormat::BbCodeV1,
        UNTRUSTED,
        generation,
    );
    record_compiled_cell(&mut cache, &third, RichTextFormat::BbCodeV1, generation);

    assert!(Arc::ptr_eq(&first, &repeated));
    assert!(!Arc::ptr_eq(&first, &second));
    assert!(!Arc::ptr_eq(&second, &third));
    assert_eq!(cache.report.hit_count, 1);
    assert_eq!(cache.report.miss_count, 3);
    assert_eq!(cache.report.eviction_count, 1);
    assert_eq!(cache.report.resident_entries, 2);
}

fn record_compiled_cell(
    cache: &mut CompiledRichTextCache,
    cell: &Arc<super::RichTextArtifactCell>,
    format: RichTextFormat,
    generation: RichTextParserGeneration,
) {
    let compiled = compiled_cell_artifact(cell, format, generation);
    cache.record_compiled(
        cell,
        format,
        UNTRUSTED,
        generation,
        compiled.estimated_bytes(),
    );
    assert!(cell.compiled.set(Ok(compiled)).is_ok());
}

fn compiled_cell_artifact(
    cell: &Arc<super::RichTextArtifactCell>,
    format: RichTextFormat,
    generation: RichTextParserGeneration,
) -> Arc<CompiledRichText> {
    compiled_cell_artifact_with_text(cell, format, generation, cell.markup.to_string())
}

fn compiled_cell_artifact_with_text(
    cell: &Arc<super::RichTextArtifactCell>,
    format: RichTextFormat,
    generation: RichTextParserGeneration,
    text: String,
) -> Arc<CompiledRichText> {
    Arc::new(
        CompiledRichText::new(
            Arc::clone(&cell.markup),
            format,
            generation,
            RichParseResult {
                text: text.into(),
                ..RichParseResult::default()
            },
        )
        .expect("test rich artifact fits indexed ranges"),
    )
}

#[test]
fn compiled_rich_cache_key_includes_format_and_registry_generations() {
    let mut cache = CompiledRichTextCache::new();
    let base = RichTextParserGeneration {
        parser_identity: 9,
        decorator_generation: 1,
        emoji_generation: 1,
    };
    let bbcode = cache.lookup_or_insert("same", RichTextFormat::BbCodeV1, UNTRUSTED, base);
    let trusted = cache.lookup_or_insert(
        "same",
        RichTextFormat::BbCodeV1,
        RichTextContentTrust::TrustedAuthoring,
        base,
    );
    let html = cache.lookup_or_insert("same", RichTextFormat::HtmlSubsetV1, UNTRUSTED, base);
    let decorated = cache.lookup_or_insert(
        "same",
        RichTextFormat::BbCodeV1,
        UNTRUSTED,
        RichTextParserGeneration {
            decorator_generation: 2,
            ..base
        },
    );

    assert!(!Arc::ptr_eq(&bbcode, &html));
    assert!(!Arc::ptr_eq(&bbcode, &trusted));
    assert!(!Arc::ptr_eq(&bbcode, &decorated));
}

#[test]
fn compiled_rich_cache_bypasses_a_second_in_flight_cell_when_budget_is_full() {
    let mut cache = CompiledRichTextCache::new();
    cache.report.max_entries = 1;
    let generation = RichTextParserGeneration::default();

    let first = cache.lookup_or_insert("first", RichTextFormat::Plain, UNTRUSTED, generation);
    let second = cache.lookup_or_insert("second", RichTextFormat::Plain, UNTRUSTED, generation);
    let repeated = cache.lookup_or_insert("first", RichTextFormat::Plain, UNTRUSTED, generation);

    assert!(Arc::ptr_eq(&first, &repeated));
    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(cache.report.resident_entries, 1);
    assert_eq!(cache.report.resident_bytes, first.markup.len());
    assert_eq!(cache.report.admission_bypass_count, 1);
}

#[test]
fn compiled_rich_cache_discards_a_completed_cell_that_exceeds_its_byte_budget() {
    let mut cache = CompiledRichTextCache::new();
    cache.report.max_bytes = 1;
    let generation = RichTextParserGeneration::default();

    let cell = cache.lookup_or_insert("x", RichTextFormat::Plain, UNTRUSTED, generation);
    let compiled = compiled_cell_artifact(&cell, RichTextFormat::Plain, generation);
    cache.record_compiled(
        &cell,
        RichTextFormat::Plain,
        UNTRUSTED,
        generation,
        compiled.estimated_bytes(),
    );
    assert!(cell.compiled.get().is_none());
    assert_eq!(cache.report.resident_entries, 0);
    assert_eq!(cache.report.resident_bytes, 0);
    assert!(cell.compiled.set(Ok(compiled)).is_ok());

    assert!(cell.compiled.get().is_some());
    assert_eq!(cache.report.eviction_count, 0);
    assert_eq!(cache.report.admission_bypass_count, 1);
}

#[test]
fn compiled_rich_cache_bypasses_markup_that_exceeds_its_byte_budget() {
    let mut cache = CompiledRichTextCache::new();
    cache.report.max_bytes = 3;

    let cell = cache.lookup_or_insert(
        "four",
        RichTextFormat::Plain,
        UNTRUSTED,
        RichTextParserGeneration::default(),
    );

    assert_eq!(cell.markup.as_ref(), "four");
    assert_eq!(cache.report.resident_entries, 0);
    assert_eq!(cache.report.resident_bytes, 0);
    assert_eq!(cache.report.admission_bypass_count, 1);
}

#[test]
fn oversized_completion_does_not_evict_a_healthy_compiled_entry() {
    let mut cache = CompiledRichTextCache::new();
    cache.report.max_entries = 2;
    let generation = RichTextParserGeneration::default();
    let retained = cache.lookup_or_insert("retained", RichTextFormat::Plain, UNTRUSTED, generation);
    record_compiled_cell(&mut cache, &retained, RichTextFormat::Plain, generation);
    let retained_bytes = cache.report.resident_bytes;
    cache.report.max_bytes = retained_bytes.saturating_add(1);

    let oversized = cache.lookup_or_insert("x", RichTextFormat::Plain, UNTRUSTED, generation);
    let compiled = compiled_cell_artifact_with_text(
        &oversized,
        RichTextFormat::Plain,
        UNTRUSTED,
        generation,
        "expanded".repeat(retained_bytes.saturating_add(1)),
    );
    let eviction_count = cache.report.eviction_count;
    cache.record_compiled(
        &oversized,
        RichTextFormat::Plain,
        UNTRUSTED,
        generation,
        compiled.estimated_bytes(),
    );
    assert!(oversized.compiled.set(Ok(compiled)).is_ok());

    let repeated = cache.lookup_or_insert("retained", RichTextFormat::Plain, UNTRUSTED, generation);
    assert!(Arc::ptr_eq(&retained, &repeated));
    assert_eq!(cache.report.eviction_count, eviction_count);
    assert_eq!(cache.report.resident_entries, 1);
    assert_eq!(cache.report.resident_bytes, retained_bytes);
    assert_eq!(cache.report.admission_bypass_count, 1);
}

#[test]
fn failed_pending_admission_does_not_partially_evict_completed_entries() {
    let mut cache = CompiledRichTextCache::new();
    cache.report.max_entries = 3;
    let generation = RichTextParserGeneration::default();
    let retained = cache.lookup_or_insert("retained", RichTextFormat::Plain, UNTRUSTED, generation);
    record_compiled_cell(&mut cache, &retained, RichTextFormat::Plain, generation);
    let retained_bytes = cache.report.resident_bytes;
    cache.report.max_bytes = retained_bytes.saturating_add(8);

    let pending = cache.lookup_or_insert("12345678", RichTextFormat::Plain, UNTRUSTED, generation);
    let eviction_count = cache.report.eviction_count;
    let oversized_markup = "x".repeat(cache.report.max_bytes);
    let bypassed = cache.lookup_or_insert(
        &oversized_markup,
        RichTextFormat::Plain,
        UNTRUSTED,
        generation,
    );

    assert!(!Arc::ptr_eq(&pending, &bypassed));
    assert_eq!(cache.report.eviction_count, eviction_count);
    assert_eq!(cache.report.resident_entries, 2);
    assert_eq!(cache.report.resident_bytes, cache.report.max_bytes);
    assert_eq!(cache.report.admission_bypass_count, 1);
    let repeated = cache.lookup_or_insert("retained", RichTextFormat::Plain, UNTRUSTED, generation);
    assert!(Arc::ptr_eq(&retained, &repeated));
}

#[test]
fn compiled_artifact_accounts_for_source_and_visible_text() {
    let compiled = CompiledRichText::new(
        Arc::from("[b]text[/b]"),
        RichTextFormat::BbCodeV1,
        RichTextParserGeneration::default(),
        RichParseResult {
            text: "text".into(),
            ..RichParseResult::default()
        },
    )
    .expect("test rich artifact fits indexed ranges");

    assert!(compiled.estimated_bytes() >= "[b]text[/b]".len() + "text".len());
}

#[test]
fn compiled_rich_cache_attributes_same_key_wait_to_the_non_initializer() {
    let owner = Arc::new(CompiledRichTextCacheOwner::default());
    let generation = RichTextParserGeneration::default();
    let initializer_ready = Arc::new(Barrier::new(2));
    let release_initializer = Arc::new((Mutex::new(false), Condvar::new()));
    let initializer = {
        let owner = Arc::clone(&owner);
        let initializer_ready = Arc::clone(&initializer_ready);
        let release_initializer = Arc::clone(&release_initializer);
        thread::spawn(move || {
            owner.compile(
                "same",
                RichTextFormat::Plain,
                UNTRUSTED,
                generation,
                |markup| {
                    initializer_ready.wait();
                    let (released, wake) = release_initializer.as_ref();
                    let mut released = released
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    while !*released {
                        released = wake
                            .wait(released)
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                    }
                    CompiledRichText::new(
                        markup,
                        RichTextFormat::Plain,
                        generation,
                        RichParseResult {
                            text: "same".into(),
                            ..RichParseResult::default()
                        },
                    )
                },
            )
        })
    };
    initializer_ready.wait();
    let waiter = {
        let owner = Arc::clone(&owner);
        thread::spawn(move || {
            owner.compile("same", RichTextFormat::Plain, UNTRUSTED, generation, |_| {
                panic!("same-key waiter must not execute the initializer")
            })
        })
    };

    let observe_deadline = Instant::now() + Duration::from_secs(5);
    while owner.report().compile_requests_in_flight != 2 {
        assert!(
            Instant::now() < observe_deadline,
            "same-key waiter did not enter single-flight before the test deadline"
        );
        thread::yield_now();
    }
    let (released, wake) = release_initializer.as_ref();
    *released
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
    wake.notify_all();

    let initialized = initializer
        .join()
        .expect("initializer thread")
        .expect("initializer result");
    let waited = waiter
        .join()
        .expect("waiter thread")
        .expect("waiter result");
    assert!(Arc::ptr_eq(&initialized, &waited));
    let report = owner.take_report();
    assert_eq!(report.compile_requests_in_flight, 0);
    assert_eq!(report.parse_count, 1);
    assert_eq!(report.single_flight_wait_count, 1);
    assert_eq!(
        report.single_flight_wait_nanos,
        report.single_flight_wait_max_nanos
    );
}

#[test]
fn compiled_rich_cache_snapshot_resets_events_and_preserves_gauges() {
    let mut cache = CompiledRichTextCache::new();
    cache.report.hit_count = u64::MAX;
    cache.report.record(RichTextCacheCounter::Hit, 1);
    cache.report.miss_count = 4;
    cache.report.compile_requests_in_flight = 2;
    cache.report.single_flight_wait_count = 3;
    cache.report.single_flight_wait_nanos = 80;
    cache.report.single_flight_wait_max_nanos = 50;
    cache.report.resident_entries = 3;
    cache.report.resident_bytes = 160;

    let interval = cache.take_report();

    assert_eq!(interval.hit_count, u64::MAX);
    assert_eq!(interval.miss_count, 4);
    assert_eq!(interval.compile_requests_in_flight, 2);
    assert_eq!(interval.single_flight_wait_count, 3);
    assert_eq!(interval.single_flight_wait_nanos, 80);
    assert_eq!(interval.single_flight_wait_max_nanos, 50);
    assert!(interval.telemetry_saturated);
    assert_eq!(interval.resident_entries, 3);
    assert_eq!(interval.resident_bytes, 160);
    assert_eq!(cache.report.hit_count, 0);
    assert_eq!(cache.report.miss_count, 0);
    assert_eq!(cache.report.compile_requests_in_flight, 2);
    assert_eq!(cache.report.single_flight_wait_count, 0);
    assert_eq!(cache.report.single_flight_wait_nanos, 0);
    assert_eq!(cache.report.single_flight_wait_max_nanos, 0);
    assert!(!cache.report.telemetry_saturated);
    assert_eq!(cache.report.resident_entries, 3);
    assert_eq!(cache.report.resident_bytes, 160);
}
