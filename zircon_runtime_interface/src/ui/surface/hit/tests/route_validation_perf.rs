use super::*;

fn validate_route_allocating(root_to_leaf: &[UiNodeId], bubble_route: &[UiNodeId]) -> bool {
    let expected_bubble_route: Vec<_> = root_to_leaf.iter().rev().copied().collect();
    bubble_route == expected_bubble_route
}

fn validate_route_borrowed(root_to_leaf: &[UiNodeId], bubble_route: &[UiNodeId]) -> bool {
    bubble_route
        .iter()
        .copied()
        .eq(root_to_leaf.iter().rev().copied())
}

#[test]
fn borrowed_route_validation_preserves_matching_and_mismatch_results() {
    let root_to_leaf = (0..64).map(UiNodeId::new).collect::<Vec<_>>();
    let mut bubble_route = root_to_leaf.iter().rev().copied().collect::<Vec<_>>();
    assert_eq!(
        validate_route_borrowed(&root_to_leaf, &bubble_route),
        validate_route_allocating(&root_to_leaf, &bubble_route),
    );

    bubble_route.pop();
    assert_eq!(
        validate_route_borrowed(&root_to_leaf, &bubble_route),
        validate_route_allocating(&root_to_leaf, &bubble_route),
    );

    bubble_route.push(UiNodeId::new(99));
    assert_eq!(
        validate_route_borrowed(&root_to_leaf, &bubble_route),
        validate_route_allocating(&root_to_leaf, &bubble_route),
    );
}

#[test]
#[ignore = "release-only borrowed hit-route validation benchmark"]
fn runtime_interface03_batch44_46_borrowed_hit_route_validation_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ROUTE_LENGTH: usize = 2048;
    const VALIDATION_COUNT: usize = 131_072;
    const SAMPLE_COUNT: usize = 11;
    let root_to_leaf = (0..ROUTE_LENGTH as u64)
        .map(UiNodeId::new)
        .collect::<Vec<_>>();
    let bubble_route = root_to_leaf.iter().rev().copied().collect::<Vec<_>>();
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..VALIDATION_COUNT {
                black_box(validate_route_allocating(
                    black_box(&root_to_leaf),
                    black_box(&bubble_route),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_borrowed = || {
            let started = Instant::now();
            for _ in 0..VALIDATION_COUNT {
                black_box(validate_route_borrowed(
                    black_box(&root_to_leaf),
                    black_box(&bubble_route),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            borrowed_samples.push(measure_borrowed());
        } else {
            borrowed_samples.push(measure_borrowed());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BORROWED_HIT_ROUTE_VALIDATION_BENCH_V1 route_length={ROUTE_LENGTH} validations={VALIDATION_COUNT} samples={SAMPLE_COUNT} allocating_p95_ns={} borrowed_p95_ns={}",
        allocating_samples[p95], borrowed_samples[p95],
    );
    assert!(
        borrowed_samples[p95].saturating_mul(5) <= allocating_samples[p95].saturating_mul(4),
        "borrowed hit-route validation must improve P95 by at least 20%: allocating={}ns borrowed={}ns",
        allocating_samples[p95],
        borrowed_samples[p95],
    );
}
