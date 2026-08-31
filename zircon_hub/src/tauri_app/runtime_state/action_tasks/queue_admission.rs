use std::collections::VecDeque;

use crate::error::HubError;
use crate::tauri_app::HubActionRequest;

pub(super) const BACKGROUND_ACTION_QUEUE_CAPACITY: usize = 64;

pub(super) fn enqueue_background_action(
    queue: &mut VecDeque<HubActionRequest>,
    request: &HubActionRequest,
) -> Result<(), HubError> {
    if queue.len() >= BACKGROUND_ACTION_QUEUE_CAPACITY {
        return Err(HubError::BackgroundActionQueueFull {
            capacity: BACKGROUND_ACTION_QUEUE_CAPACITY,
        });
    }
    queue.push_back(request.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use serde_json::json;

    use super::*;

    fn request(action_id: &str) -> HubActionRequest {
        HubActionRequest {
            action_id: action_id.to_string(),
            target_id: Some("project".to_string()),
            payload: None,
        }
    }

    #[test]
    fn hub06_background_queue_rejects_request_at_capacity() {
        let request = request("build-project");
        let mut queue = VecDeque::new();

        for _ in 0..BACKGROUND_ACTION_QUEUE_CAPACITY {
            enqueue_background_action(&mut queue, &request).expect("request below capacity");
        }
        let error = enqueue_background_action(&mut queue, &request)
            .expect_err("request at capacity must be rejected before cloning");

        assert!(matches!(
            error,
            HubError::BackgroundActionQueueFull {
                capacity: BACKGROUND_ACTION_QUEUE_CAPACITY
            }
        ));
        assert_eq!(queue.len(), BACKGROUND_ACTION_QUEUE_CAPACITY);
    }

    #[test]
    fn hub06_background_queue_preserves_fifo_below_capacity() {
        let mut queue = VecDeque::new();
        enqueue_background_action(&mut queue, &request("build-project")).unwrap();
        enqueue_background_action(&mut queue, &request("package-project")).unwrap();

        assert_eq!(queue.pop_front().unwrap().action_id, "build-project");
        assert_eq!(queue.pop_front().unwrap().action_id, "package-project");
    }

    #[test]
    #[ignore = "release-only background queue admission benchmark"]
    fn hub06_background_queue_admission_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 21;
        const ADMISSIONS_PER_SAMPLE: usize = 10_000;
        const REQUEST_PAYLOAD_BYTES: usize = 1_024;

        fn benchmark_request() -> HubActionRequest {
            HubActionRequest {
                action_id: "build-project".to_string(),
                target_id: Some("project".to_string()),
                payload: Some(json!({ "padding": "x".repeat(REQUEST_PAYLOAD_BYTES) })),
            }
        }

        fn measure_legacy(request: &HubActionRequest) -> u128 {
            let mut queue = VecDeque::new();
            let started = Instant::now();
            for _ in 0..ADMISSIONS_PER_SAMPLE {
                queue.push_back(request.clone());
            }
            black_box(queue.len());
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(request: &HubActionRequest) -> u128 {
            let mut queue = VecDeque::new();
            let started = Instant::now();
            for _ in 0..ADMISSIONS_PER_SAMPLE {
                black_box(enqueue_background_action(&mut queue, request).is_ok());
            }
            black_box(queue.len());
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let request = benchmark_request();
        for _ in 0..4 {
            black_box(measure_legacy(&request));
            black_box(measure_optimized(&request));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&request));
                optimized_samples.push(measure_optimized(&request));
            } else {
                optimized_samples.push(measure_optimized(&request));
                legacy_samples.push(measure_legacy(&request));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "HUB06_BACKGROUND_QUEUE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
admissions_per_sample={ADMISSIONS_PER_SAMPLE} request_payload_bytes={REQUEST_PAYLOAD_BYTES} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_retained_requests=10000 \
optimized_retained_requests={BACKGROUND_ACTION_QUEUE_CAPACITY} \
retained_request_reduction_pct=99.360 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(35),
            "bounded queue admission must reduce overload P95 by at least 65%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
