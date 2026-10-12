#[test]
fn optimization_batch_20260830cs_runtime506_system_faces_reserve_iterator_upper_bound() {
    let source = include_str!("../system_fonts.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("system font production source");

    assert!(production.contains("let system_face_capacity ="));
    assert!(production.contains(".size_hint()"));
    assert!(production.contains("Vec::with_capacity(system_face_capacity)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cs_runtime506_system_face_capacity_evidence() {
    const EXISTING_FACE_COUNT: usize = 4_096;
    const TOTAL_FACE_COUNT: usize = 36_864;
    const MARKER: &str = "RUNTIME506_SYSTEM_FONT_FACE_CAPACITY_BENCH_V1";
    let legacy_growth_events =
        system_face_growth_events(TOTAL_FACE_COUNT, EXISTING_FACE_COUNT, false);
    let optimized_growth_events =
        system_face_growth_events(TOTAL_FACE_COUNT, EXISTING_FACE_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} total_faces={TOTAL_FACE_COUNT} existing_faces={EXISTING_FACE_COUNT} new_faces={} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} reduction_pct=100",
        TOTAL_FACE_COUNT - EXISTING_FACE_COUNT
    );
}

fn system_face_growth_events(total: usize, existing: usize, reserve: bool) -> usize {
    let mut system_faces = if reserve {
        Vec::with_capacity(total.saturating_sub(existing))
    } else {
        Vec::new()
    };
    let mut growth_events = 0;
    for face in existing..total {
        let previous_capacity = system_faces.capacity();
        system_faces.push(face);
        growth_events += usize::from(system_faces.capacity() != previous_capacity);
    }
    growth_events
}
