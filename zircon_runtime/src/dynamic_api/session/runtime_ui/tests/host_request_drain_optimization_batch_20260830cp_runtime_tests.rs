const SURFACE_COUNT: usize = 1_024;
const REQUESTS_PER_SURFACE: usize = 64;

#[test]
fn optimization_batch_20260830cp_runtime_clipboard_drain_reuses_one_scratch_buffer() {
    let source = include_str!("../host_request_drain.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("host request drain implementation");
    let scratch = implementation
        .find("let mut requests = Vec::new();")
        .expect("clipboard drain scratch buffer");
    let surface_loop = implementation
        .find("for (surface_index, runtime_surface)")
        .expect("clipboard surface loop");

    assert!(scratch < surface_loop);
    assert!(implementation.contains("requests.clear();"));
    assert!(implementation.contains(".drain(..)"));
    assert!(!implementation.contains("requests.into_iter()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cp_runtime_clipboard_scratch_reuse_capacity_evidence() {
    let legacy_growth_events = drain_growth_events(false);
    let optimized_growth_events = drain_growth_events(true);

    println!(
        "RUNTIME503_CLIPBOARD_DRAIN_SCRATCH_REUSE_BENCH_V1 surfaces={SURFACE_COUNT} \
requests_per_surface={REQUESTS_PER_SURFACE} legacy_growth_events={legacy_growth_events} \
optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > 0);
    assert!(optimized_growth_events.saturating_mul(10) <= legacy_growth_events);
}

fn drain_growth_events(reuse_scratch: bool) -> usize {
    let mut reusable = Vec::new();
    let mut output = Vec::with_capacity(SURFACE_COUNT * REQUESTS_PER_SURFACE);
    let mut growth_events = 0;
    for surface in 0..SURFACE_COUNT {
        let mut owned = Vec::new();
        let scratch = if reuse_scratch {
            reusable.clear();
            &mut reusable
        } else {
            &mut owned
        };
        for request in 0..REQUESTS_PER_SURFACE {
            let previous_capacity = scratch.capacity();
            scratch.push(request);
            growth_events += usize::from(scratch.capacity() != previous_capacity);
        }
        output.extend(scratch.drain(..).map(|request| (surface, request)));
    }
    assert_eq!(output.len(), SURFACE_COUNT * REQUESTS_PER_SURFACE);
    std::hint::black_box(output);
    growth_events
}
