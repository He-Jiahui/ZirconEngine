use std::{hint::black_box, sync::Arc, time::Instant};

use zircon_runtime::core::framework::net::{
    NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetRequestId,
};

use crate::http::HttpRouteHandler;

use super::dispatch_local_http_route;

const BENCHMARK_BODY_BYTES: usize = 65_536;
const BENCHMARK_REQUEST_COUNT: usize = 128;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn moved_local_http_dispatch_preserves_handler_and_fallback_results() {
    let handler = Arc::new(|request: NetHttpRequestDescriptor| {
        NetHttpResponseDescriptor::new(request.request, 201, request.body)
    }) as HttpRouteHandler;
    let dynamic = dispatch_local_http_route(
        Some(handler),
        NetHttpResponseDescriptor::new(NetRequestId::new(0), 500, Vec::new()),
        NetHttpRequestDescriptor::new(
            NetRequestId::new(41),
            NetHttpMethod::Post,
            "http://127.0.0.1/echo",
        )
        .with_body(b"payload".to_vec()),
    );
    assert_eq!(dynamic.request, NetRequestId::new(41));
    assert_eq!(dynamic.status_code, 201);
    assert_eq!(dynamic.body, b"payload");

    let fallback = dispatch_local_http_route(
        None,
        NetHttpResponseDescriptor::new(NetRequestId::new(0), 202, b"queued".to_vec()),
        NetHttpRequestDescriptor::new(
            NetRequestId::new(42),
            NetHttpMethod::Get,
            "http://127.0.0.1/status",
        ),
    );
    assert_eq!(fallback.request, NetRequestId::new(42));
    assert_eq!(fallback.status_code, 202);
    assert_eq!(fallback.body, b"queued");
}

#[test]
#[ignore = "release-only performance evidence"]
fn moved_local_http_request_release_benchmark_evidence() {
    let requests = benchmark_requests();
    let handler = Arc::new(|request: NetHttpRequestDescriptor| {
        NetHttpResponseDescriptor::new(request.request, 204, Vec::new())
    }) as HttpRouteHandler;
    let fallback = NetHttpResponseDescriptor::new(NetRequestId::new(0), 500, Vec::new());
    assert_eq!(
        legacy_dispatch_batch(requests.clone(), &handler, &fallback),
        moved_dispatch_batch(requests.clone(), &handler, &fallback)
    );

    let (legacy_samples, optimized_samples) =
        benchmark_paired_samples(&requests, &handler, &fallback);
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_raw_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_raw_ns = benchmark_samples_csv(&optimized_samples);
    let legacy_body_copy_bytes = BENCHMARK_BODY_BYTES * BENCHMARK_REQUEST_COUNT;

    println!(
        "PERF_RESULT task=plugins10_moved_local_http_request requests={BENCHMARK_REQUEST_COUNT} body_bytes_per_request={BENCHMARK_BODY_BYTES} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_request_clones_per_sample={BENCHMARK_REQUEST_COUNT} optimized_request_clones_per_sample=0 legacy_body_copy_bytes_per_sample={legacy_body_copy_bytes} optimized_body_copy_bytes_per_sample=0 threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_raw_ns} optimized_raw_ns={optimized_raw_ns}"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
    );
}

fn benchmark_requests() -> Vec<NetHttpRequestDescriptor> {
    (1..=BENCHMARK_REQUEST_COUNT as u64)
        .map(|raw| {
            NetHttpRequestDescriptor::new(
                NetRequestId::new(raw),
                NetHttpMethod::Post,
                "http://127.0.0.1/benchmark",
            )
            .with_body(vec![raw as u8; BENCHMARK_BODY_BYTES])
        })
        .collect()
}

fn legacy_dispatch_batch(
    requests: Vec<NetHttpRequestDescriptor>,
    handler: &HttpRouteHandler,
    fallback: &NetHttpResponseDescriptor,
) -> usize {
    let mut checksum = 0;
    for request in requests {
        let request_id = request.request;
        let local_handler = Some(Arc::clone(handler));
        let response = fallback.clone();
        let response = local_handler
            .map(|handler| handler(request.clone()))
            .unwrap_or_else(|| response.for_request(request_id));
        checksum += black_box(response.status_code as usize + response.body_bytes);
    }
    black_box(checksum)
}

fn moved_dispatch_batch(
    requests: Vec<NetHttpRequestDescriptor>,
    handler: &HttpRouteHandler,
    fallback: &NetHttpResponseDescriptor,
) -> usize {
    let mut checksum = 0;
    for request in requests {
        let response =
            dispatch_local_http_route(Some(Arc::clone(handler)), fallback.clone(), request);
        checksum += black_box(response.status_code as usize + response.body_bytes);
    }
    black_box(checksum)
}

fn benchmark_paired_samples(
    requests: &[NetHttpRequestDescriptor],
    handler: &HttpRouteHandler,
    fallback: &NetHttpResponseDescriptor,
) -> (Vec<u128>, Vec<u128>) {
    black_box(benchmark_sample(
        requests,
        handler,
        fallback,
        legacy_dispatch_batch,
    ));
    black_box(benchmark_sample(
        requests,
        handler,
        fallback,
        moved_dispatch_batch,
    ));
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(
                requests,
                handler,
                fallback,
                legacy_dispatch_batch,
            ));
            optimized_samples.push(benchmark_sample(
                requests,
                handler,
                fallback,
                moved_dispatch_batch,
            ));
        } else {
            optimized_samples.push(benchmark_sample(
                requests,
                handler,
                fallback,
                moved_dispatch_batch,
            ));
            legacy_samples.push(benchmark_sample(
                requests,
                handler,
                fallback,
                legacy_dispatch_batch,
            ));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(
    requests: &[NetHttpRequestDescriptor],
    handler: &HttpRouteHandler,
    fallback: &NetHttpResponseDescriptor,
    dispatch: fn(
        Vec<NetHttpRequestDescriptor>,
        &HttpRouteHandler,
        &NetHttpResponseDescriptor,
    ) -> usize,
) -> u128 {
    let requests = requests.to_vec();
    let started = Instant::now();
    let checksum = black_box(dispatch(requests, handler, fallback));
    let elapsed = started.elapsed().as_nanos();
    black_box(checksum);
    elapsed
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
