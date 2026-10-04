use super::UiSecureTextPresentation;

use crate::core::framework::text::TextDirection;

#[test]
fn runtime754_secure_text_capacity_preserves_mask_and_line_semantics() {
    let source = "alpha\r\nbeta\u{2028}gamma";
    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto)
        .expect("valid secure text should retain its presentation");

    assert_eq!(presentation.source_len(), source.len());
    assert_eq!(presentation.lines().len(), 3);
    assert_eq!(presentation.display_text().matches('\u{2022}').count(), 14);
    assert_eq!(super::secure_display_capacity(128), 384);
    assert_eq!(super::secure_display_capacity(usize::MAX), usize::MAX);
    assert!(presentation
        .clusters()
        .iter()
        .any(|cluster| cluster.is_hard_line_separator));
}

#[test]
#[ignore = "run in the managed Release validation batch"]
fn runtime754_secure_text_capacity_bench_v1() {
    let source = (0..64)
        .map(|index| format!("line-{index:04}-payload"))
        .collect::<Vec<_>>()
        .join("\n");
    let legacy_growth_events = growth_events(source.len(), None);
    let optimized_growth_events = growth_events(source.len(), Some(source.len()));
    let display_bytes = super::secure_display_capacity(source.len());
    let optimized_display_growth_events = growth_events(display_bytes, Some(display_bytes));
    eprintln!(
        "RUNTIME754_SECURE_TEXT_PRESENTATION_CAPACITY_BENCH_V1 bytes={} legacy_growth_events={} optimized_growth_events={} optimized_display_growth_events={}",
        source.len(),
        legacy_growth_events,
        optimized_growth_events,
        optimized_display_growth_events
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    assert_eq!(optimized_display_growth_events, 0);
}

fn growth_events(length: usize, initial_capacity: Option<usize>) -> usize {
    let mut values = initial_capacity.map_or_else(Vec::new, Vec::with_capacity);
    let mut previous_capacity = values.capacity();
    let mut growth_events = 0;
    for value in 0..length {
        values.push(value);
        if values.capacity() != previous_capacity {
            growth_events += 1;
            previous_capacity = values.capacity();
        }
    }
    growth_events
}
