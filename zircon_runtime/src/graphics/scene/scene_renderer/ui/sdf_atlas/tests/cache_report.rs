use super::*;

#[test]
fn sdf_atlas_cache_report_tracks_dirty_rects_by_page_key() {
    let previous = synthetic_plan(vec![
        slot_on_page('A', 0, sdf_rect(0, 0, 64, 64)),
        slot_on_page('B', 1, sdf_rect(0, 0, 64, 64)),
    ]);
    let current = synthetic_plan(vec![
        slot_on_page('A', 0, sdf_rect(0, 0, 64, 64)),
        slot_on_page('B', 1, sdf_rect(64, 0, 64, 64)),
        slot_on_page('C', 0, sdf_rect(128, 0, 64, 64)),
    ]);

    let report = cache_report_for_plan_transition(&previous, &current);

    assert_eq!(report.retained_slot_count, 2);
    assert_eq!(report.stable_slot_count, 1);
    assert_eq!(report.relocated_slot_count, 1);
    assert_eq!(report.added_slot_count, 1);
    assert_eq!(report.dirty_rect, Some(sdf_rect(128, 0, 64, 64)));
    assert_eq!(
        report.dirty_pages,
        vec![
            dirty_page(0, sdf_rect(128, 0, 64, 64)),
            dirty_page(1, sdf_rect(64, 0, 64, 64)),
        ]
    );
}

#[test]
fn sdf_atlas_cache_report_marks_rebuilt_pages_full_dirty() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 1);
    let previous = synthetic_plan(vec![slot_on_page('A', 1, sdf_rect(0, 0, 64, 64))]);
    let current = synthetic_plan_with_rebuilt_pages(
        vec![slot_on_page('A', 1, sdf_rect(0, 0, 64, 64))],
        vec![page_key],
    );

    let report = cache_report_for_plan_transition(&previous, &current);

    assert_eq!(report.retained_slot_count, 1);
    assert_eq!(report.stable_slot_count, 1);
    assert_eq!(report.relocated_slot_count, 0);
    assert_eq!(report.added_slot_count, 0);
    assert_eq!(report.evicted_slot_count, 0);
    assert_eq!(report.dirty_rect, None);
    assert_eq!(
        report.dirty_pages,
        vec![dirty_page(1, sdf_rect(0, 0, 256, 256))]
    );
}

#[test]
fn font_face_invalidation_rebuilds_stable_slots_as_dirty_pages() {
    let texts = [text_batch("AB", UiFrame::new(0.0, 0.0, 128.0, 32.0))];
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    atlas.prepare(&texts);
    atlas.prepare(&texts);
    assert_eq!(atlas.cache_report().stable_slot_count, 2);
    assert!(atlas.cache_report().dirty_pages.is_empty());

    atlas.invalidate_font_faces();
    atlas.prepare(&[]);
    atlas.mark_prepared_pages_uploaded();
    atlas.prepare(&texts);

    let report = atlas.cache_report();
    assert_eq!(report.previous_slot_count, 0);
    assert_eq!(report.current_slot_count, 2);
    assert_eq!(report.stable_slot_count, 0);
    assert_eq!(report.added_slot_count, 2);
    assert!(!report.dirty_pages.is_empty());
    assert!(report.dirty_pages.iter().all(|page| {
        page.dirty_rect.x == 0
            && page.dirty_rect.y == 0
            && page.dirty_rect.width == atlas.plan().atlas_size.x
            && page.dirty_rect.height == atlas.plan().atlas_size.y
    }));

    atlas.prepare(&texts);
    assert!(!atlas.cache_report().dirty_pages.is_empty());
    assert!(atlas.cache_report().dirty_pages.iter().all(|page| {
        page.dirty_rect.x == 0
            && page.dirty_rect.y == 0
            && page.dirty_rect.width == atlas.plan().atlas_size.x
            && page.dirty_rect.height == atlas.plan().atlas_size.y
    }));

    atlas.mark_prepared_pages_uploaded();
    atlas.prepare(&texts);
    assert!(atlas.cache_report().dirty_pages.is_empty());
}

#[test]
fn optimization_batch_20260826g_runtime11c_borrowed_cache_accounting_preserves_eviction_order() {
    let current_keys = ['D', 'E']
        .into_iter()
        .map(glyph_key)
        .collect::<BTreeSet<_>>();
    let mut cached_slots = [('C', 2_u64), ('A', 1), ('B', 1), ('D', 3), ('E', 3)]
        .into_iter()
        .map(|(glyph, last_seen_generation)| SdfAtlasCachedSlot {
            key: glyph_key(glyph),
            last_seen_generation,
        })
        .collect::<Vec<_>>();

    evict_inactive_slots(
        &mut cached_slots,
        &current_keys,
        SdfAtlasQuality {
            max_cached_slot_count: 3,
            ..SdfAtlasQuality::default()
        },
    );

    assert_eq!(
        cached_slots
            .iter()
            .map(|slot| slot.key.glyph)
            .collect::<Vec<_>>(),
        vec!['C', 'D', 'E']
    );
}

#[test]
fn optimization_batch_20260826g_runtime11c_cache_accounting_borrows_glyph_keys() {
    let source = include_str!("../../sdf_atlas.rs");
    for (start, end) in [
        ("fn insert_new_slots", "fn evict_inactive_slots"),
        (
            "fn evict_inactive_slots",
            "fn cache_report_for_plan_transition",
        ),
        (
            "fn cache_report_for_plan_transition",
            "fn dirty_pages_for_plan_transition",
        ),
    ] {
        let function = source
            .split(start)
            .nth(1)
            .unwrap_or_else(|| panic!("missing {start}"))
            .split(end)
            .next()
            .unwrap_or_else(|| panic!("missing boundary {end}"));
        assert!(
            !function.contains(".key.clone()"),
            "{start} must not clone glyph keys"
        );
    }
    assert!(source.contains("BTreeSet<&SdfAtlasGlyphKey>"));
    assert!(source.contains("BTreeMap<&SdfAtlasGlyphKey"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826g_runtime11c_borrowed_cache_accounting_performance_evidence() {
    use std::hint::black_box;
    use std::time::Instant;

    fn legacy_identity_counts(previous: &SdfAtlasPlan, current: &SdfAtlasPlan) -> [usize; 4] {
        let previous_keys = previous
            .slots
            .iter()
            .map(|slot| slot.key.clone())
            .collect::<BTreeSet<_>>();
        let current_keys = current
            .slots
            .iter()
            .map(|slot| slot.key.clone())
            .collect::<BTreeSet<_>>();
        let previous_slots = previous
            .slots
            .iter()
            .map(|slot| (slot.key.clone(), (slot.page_key, slot.rect)))
            .collect::<BTreeMap<_, _>>();
        let current_slots = current
            .slots
            .iter()
            .map(|slot| (slot.key.clone(), (slot.page_key, slot.rect)))
            .collect::<BTreeMap<_, _>>();
        let retained = current_keys.intersection(&previous_keys).count();
        let stable = current_keys
            .intersection(&previous_keys)
            .filter(|key| previous_slots.get(*key) == current_slots.get(*key))
            .count();
        let added = current_keys.difference(&previous_keys).count();
        let evicted = previous_keys.difference(&current_keys).count();
        [retained, stable, added, evicted]
    }

    let slots = (0..4_096_u32)
        .map(|index| {
            slot_on_page(
                char::from_u32(0x1_000 + index).expect("valid benchmark glyph"),
                index % 4,
                sdf_rect((index % 32) * 64, ((index / 32) % 32) * 64, 64, 64),
            )
        })
        .collect::<Vec<_>>();
    let previous = synthetic_plan(slots.clone());
    let current = synthetic_plan(slots);
    let mut legacy_samples = Vec::with_capacity(17);
    let mut borrowed_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        black_box(legacy_identity_counts(&previous, &current));
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        let report = cache_report_for_plan_transition(black_box(&previous), black_box(&current));
        black_box((
            report.retained_slot_count,
            report.stable_slot_count,
            report.added_slot_count,
            report.evicted_slot_count,
        ));
        borrowed_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let borrowed_p95 = borrowed_samples[16];
    println!(
        "RUNTIME11C_SDF_ATLAS_BORROWED_CACHE_ACCOUNTING_BENCH_V1 slots={} legacy_p95_ns={} borrowed_p95_ns={} legacy_key_clones={} borrowed_key_clones=0 legacy_owned_key_strings={} borrowed_owned_key_strings=0 target_ratio_bp=6000",
        previous.slots.len(),
        legacy_p95,
        borrowed_p95,
        previous.slots.len() * 4,
        previous.slots.len() * 8,
    );
    assert!(
        borrowed_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "borrowed cache accounting P95 {borrowed_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}
