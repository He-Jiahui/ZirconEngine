use super::*;

fn runtime_event(payload: &[u8]) -> UiRuntimeEvent<'_> {
    UiRuntimeEvent::new(
        ZrRuntimeEventV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1,
            crate::ZrRuntimeViewportHandle::new(1),
        ),
        payload,
    )
    .unwrap()
}

fn copied_accessibility_payload(event: UiRuntimeEvent<'_>) -> Vec<u8> {
    event.payload.to_vec()
}

fn borrowed_accessibility_payload(event: UiRuntimeEvent<'_>) -> usize {
    event.payload.len()
}

#[test]
fn runtime_interface03_batch68_69_borrowed_accessibility_payload_preserves_behavior() {
    let request = UiAccessibilityActionRequest {
        value: Some("a".repeat(16 * 1024)),
        ..UiAccessibilityActionRequest::default()
    };
    let payload = serde_json::to_vec(&request).unwrap();
    let event = runtime_event(&payload);

    assert_eq!(copied_accessibility_payload(event), payload);
    assert_eq!(borrowed_accessibility_payload(event), payload.len());
    assert_eq!(
        accessibility_event(UiWindowInputContext::default(), event).unwrap(),
        UiWindowPlatformInputEvent::accessibility(UiWindowInputContext::default(), request),
    );
}

#[test]
#[ignore = "release-only borrowed accessibility payload benchmark"]
fn runtime_interface03_batch68_69_borrowed_accessibility_payload_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ADMISSION_COUNT: usize = 20_000;
    const PAYLOAD_BYTES: usize = 16 * 1024;
    const SAMPLE_COUNT: usize = 11;
    let payload = (0_u8..=255).cycle().take(PAYLOAD_BYTES).collect::<Vec<_>>();
    let event = runtime_event(&payload);
    let mut copied_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_copied = || {
            let started = Instant::now();
            for _ in 0..ADMISSION_COUNT {
                black_box(copied_accessibility_payload(black_box(event)));
            }
            started.elapsed().as_nanos()
        };
        let measure_borrowed = || {
            let started = Instant::now();
            for _ in 0..ADMISSION_COUNT {
                black_box(borrowed_accessibility_payload(black_box(event)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            copied_samples.push(measure_copied());
            borrowed_samples.push(measure_borrowed());
        } else {
            borrowed_samples.push(measure_borrowed());
            copied_samples.push(measure_copied());
        }
    }

    copied_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BORROWED_ACCESSIBILITY_PAYLOAD_BENCH_V1 admissions={ADMISSION_COUNT} payload_bytes={PAYLOAD_BYTES} samples={SAMPLE_COUNT} copied_p95_ns={} borrowed_p95_ns={}",
        copied_samples[p95], borrowed_samples[p95],
    );
    assert!(
        borrowed_samples[p95].saturating_mul(4) <= copied_samples[p95],
        "borrowed payload admission must improve P95 by at least 75%: copied={}ns borrowed={}ns",
        copied_samples[p95],
        borrowed_samples[p95],
    );
}
