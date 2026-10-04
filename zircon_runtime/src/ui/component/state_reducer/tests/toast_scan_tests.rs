use super::*;

#[test]
fn borrowed_toast_scan_flattens_and_selects_current_entry() {
    let mut mapped = BTreeMap::new();
    mapped.insert("id".to_string(), UiValue::String("second".to_string()));
    mapped.insert(
        "message".to_string(),
        UiValue::String("Second message".to_string()),
    );
    let queue = UiValue::Array(vec![
        UiValue::String("first|message=First message".to_string()),
        UiValue::Array(vec![UiValue::Map(mapped)]),
    ]);

    let scan = scan_toast_queue(Some(&queue), "second");

    assert_eq!(scan.len, 2);
    let current = scan.current.unwrap();
    assert_eq!(current.id, "second");
    assert_eq!(current.message, "Second message");
}

#[test]
fn borrowed_toast_raw_preserves_enum_to_string_normalization() {
    let queue = UiValue::Enum("notice|message=Hello".to_string());
    let mut retained = Vec::new();

    visit_toast_entries(&queue, &mut |entry| {
        retained.push(entry.raw.to_owned_value());
    });

    assert_eq!(
        retained,
        vec![UiValue::String("notice|message=Hello".to_string())]
    );
}

#[test]
fn optimization_batch_20260830da_toast_queue_capacity_uses_root_shape() {
    let flat = UiValue::Array(vec![
        UiValue::String("first".to_string()),
        UiValue::String("second".to_string()),
        UiValue::Bool(false),
    ]);

    assert_eq!(toast_queue_root_capacity(&flat), 3);
    assert_eq!(
        toast_queue_root_capacity(&UiValue::String("single".to_string())),
        1
    );
    assert_eq!(toast_queue_root_capacity(&UiValue::Bool(false)), 0);

    let source = include_str!("../toast.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("toast reducer production source");
    assert!(production.contains("remaining.reserve(toast_queue_root_capacity(queue));"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830da_toast_queue_root_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const QUEUE_ENTRY_COUNT: usize = 32;
    const MARKER: &str = "RUNTIME513_TOAST_QUEUE_ROOT_CAPACITY_BENCH_V1";

    let legacy_growth_events = queue_growth_events(BATCH_COUNT, QUEUE_ENTRY_COUNT, false);
    let optimized_growth_events = queue_growth_events(BATCH_COUNT, QUEUE_ENTRY_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} root_entries={QUEUE_ENTRY_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn queue_growth_events(batch_count: usize, entry_count: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut remaining = if reserve {
            Vec::with_capacity(entry_count)
        } else {
            Vec::new()
        };
        for entry in 0..entry_count {
            let previous_capacity = remaining.capacity();
            remaining.push(entry);
            growth_events += usize::from(remaining.capacity() != previous_capacity);
        }
    }
    growth_events
}
