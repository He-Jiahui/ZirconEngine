const BENCHMARK_MARKER: &str = "EDITOR798_BUILTIN_TEMPLATE_DOCUMENT_ID_CAPACITY_BENCH_V1";

#[test]
fn editor798_builtin_template_document_id_capacity_is_input_bounded() {
    let source = include_str!("../../build_session.rs");
    let body = source
        .split("pub(super) fn load_builtin_host_templates_for_document_ids")
        .nth(1)
        .and_then(|body| {
            body.split("fn load_builtin_host_templates_for_documents")
                .next()
        })
        .expect("builtin document-id filter implementation");

    assert!(body.contains("HashSet::with_capacity(document_ids.len())"));
    assert!(body.contains("requested_document_ids.extend(document_ids.iter().copied())"));
    assert!(!body.contains("collect::<HashSet<_>>()"));
}

#[test]
#[ignore = "run in the managed Release validation batch"]
fn optimization_batch_20260918_editor798_builtin_template_document_id_capacity_bench() {
    const DOCUMENT_ID_COUNT: usize = 4_096;
    let legacy_growth_events = geometric_growth_events(DOCUMENT_ID_COUNT);
    let optimized_growth_events = 0;
    eprintln!(
        "{BENCHMARK_MARKER} document_id_count={DOCUMENT_ID_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
}

fn geometric_growth_events(length: usize) -> usize {
    let mut capacity = 0usize;
    let mut growth_events = 0usize;
    for current_length in 1..=length {
        if current_length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            growth_events += 1;
        }
    }
    growth_events
}
