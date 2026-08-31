use std::hint::black_box;
use std::time::Instant;

use super::{
    append_ime_host_requests_for_input_method_requests, reserve_ime_host_request_capacity,
    ImeHostRequest, MAX_HOST_REQUESTS_PER_INPUT_METHOD_REQUEST,
};
use zircon_runtime_interface::ui::{
    dispatch::{UiInputMethodRequest, UiInputMethodRequestKind, UiInputMethodSurroundingText},
    event_ui::UiNodeId,
    layout::UiFrame,
};

const SAMPLE_PAIRS: usize = 21;
const APPENDS_PER_SAMPLE: usize = 1_024;
const REQUESTS_PER_APPEND: usize = 256;

#[test]
fn runtime_hotpath_batch_runtime177_182_capacity_preserves_ime_request_expansion() {
    let owner = UiNodeId::new(7);
    let requests = vec![
        UiInputMethodRequest {
            kind: UiInputMethodRequestKind::Enable,
            owner,
            cursor_rect: Some(UiFrame::new(10.0, 20.0, 1.0, 18.0)),
            composition_rects: Vec::new(),
            surrounding_text: Some(
                UiInputMethodSurroundingText::new("input", 5, 5)
                    .expect("fixture surrounding text should be valid"),
            ),
        },
        UiInputMethodRequest {
            kind: UiInputMethodRequestKind::Disable,
            owner,
            cursor_rect: Some(UiFrame::new(0.0, 0.0, 1.0, 1.0)),
            composition_rects: Vec::new(),
            surrounding_text: None,
        },
    ];
    let mut output = Vec::new();

    append_ime_host_requests_for_input_method_requests(requests, &mut output);

    assert_eq!(output.len(), 4);
    assert!(matches!(&output[0], &ImeHostRequest::Enable));
    assert!(matches!(&output[1], &ImeHostRequest::SetCursorArea(_)));
    assert!(matches!(&output[2], &ImeHostRequest::SetSurroundingText(_)));
    assert!(matches!(&output[3], &ImeHostRequest::Disable));
}

#[test]
fn runtime_hotpath_batch_runtime177_182_batch_reserves_maximum_expansion_once() {
    let source = include_str!("../ime_host_requests.rs");
    let batch_start = source
        .find("pub(super) fn append_ime_host_requests_for_input_method_requests")
        .unwrap();
    let batch_end = source[batch_start..]
        .find("fn append_ime_host_requests_for_input_method_request")
        .map(|offset| batch_start + offset)
        .unwrap();
    let batch_source = &source[batch_start..batch_end];

    assert!(batch_source.contains("let requests = requests.into_iter();"));
    assert!(batch_source.contains("reserve_ime_host_request_capacity("));
    assert!(batch_source.contains("requests.size_hint().0"));
    assert!(source.contains("MAX_HOST_REQUESTS_PER_INPUT_METHOD_REQUEST: usize = 3"));
    assert!(source.contains("request_count.saturating_mul("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime_hotpath_batch_runtime177_182_ime_host_request_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME182_IME_HOST_REQUEST_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
appends_per_sample={APPENDS_PER_SAMPLE} requests_per_append={REQUESTS_PER_APPEND} \
host_requests_per_request={MAX_HOST_REQUESTS_PER_INPUT_METHOD_REQUEST} \
legacy_reservations_per_append=0 optimized_reservations_per_append=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved IME host request append P95 {optimized_p95_ns}ns must be at most 70% of growth-driven append P95 {legacy_p95_ns}ns"
    );
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..APPENDS_PER_SAMPLE {
        let mut output = Vec::new();
        if reserve {
            reserve_ime_host_request_capacity(&mut output, REQUESTS_PER_APPEND);
        }
        for _ in 0..REQUESTS_PER_APPEND {
            output.push(ImeHostRequest::Enable);
            output.push(ImeHostRequest::Enable);
            output.push(ImeHostRequest::Enable);
        }
        checksum ^= black_box(output.len() ^ output.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
