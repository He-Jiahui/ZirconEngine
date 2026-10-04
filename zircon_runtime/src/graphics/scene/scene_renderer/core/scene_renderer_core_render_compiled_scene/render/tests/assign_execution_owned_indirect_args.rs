use std::hint::black_box;
use std::time::Instant;

#[derive(Clone, Copy)]
struct TestDraw {
    transparent: bool,
    indirect: bool,
}

#[test]
fn execution_indirect_indices_keep_mesh_order_without_deferred_lighting() {
    let draws = test_draws();

    let indices = super::collect_execution_indirect_draw_indices(
        &draws,
        false,
        |draw| draw.transparent,
        |draw| draw.indirect,
    );

    assert_eq!(indices, vec![0, 1, 4]);
}

#[test]
fn execution_indirect_indices_submit_opaque_before_transparent_with_deferred_lighting() {
    let draws = test_draws();

    let indices = super::collect_execution_indirect_draw_indices(
        &draws,
        true,
        |draw| draw.transparent,
        |draw| draw.indirect,
    );

    assert_eq!(indices, vec![1, 4, 0]);
}

#[test]
fn optimization_batch_20260830es_runtime552_copies_and_assigns_in_one_index_traversal() {
    let production = include_str!("../assign_execution_owned_indirect_args.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(!production.contains("indirect_execution_draw_indices.iter()"));
    assert_eq!(
        production
            .matches("indirect_execution_draw_indices.into_iter()")
            .count(),
        1
    );
}

#[test]
#[ignore = "deterministic performance marker"]
fn optimization_batch_20260830es_runtime552_single_index_traversal_benchmark() {
    const DRAW_COUNT: usize = 65_536;
    const SAMPLES: usize = 9;
    let indices = (0..DRAW_COUNT).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for _ in 0..SAMPLES {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for index in indices.iter().copied() {
            checksum = checksum.wrapping_add(index.rotate_left(3));
        }
        for index in indices.iter().copied() {
            checksum = checksum.wrapping_add(index.rotate_left(7));
        }
        black_box(checksum);
        legacy_samples.push(started.elapsed());

        let started = Instant::now();
        let mut checksum = 0_usize;
        for index in indices.iter().copied() {
            checksum = checksum
                .wrapping_add(index.rotate_left(3))
                .wrapping_add(index.rotate_left(7));
        }
        black_box(checksum);
        optimized_samples.push(started.elapsed());
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy = legacy_samples[SAMPLES / 2];
    let optimized = optimized_samples[SAMPLES / 2];
    println!(
        "RUNTIME552_SINGLE_INDEX_TRAVERSAL_BENCH_V1 legacy={legacy:?} optimized={optimized:?}"
    );
}

fn test_draws() -> [TestDraw; 5] {
    [
        TestDraw {
            transparent: true,
            indirect: true,
        },
        TestDraw {
            transparent: false,
            indirect: true,
        },
        TestDraw {
            transparent: false,
            indirect: false,
        },
        TestDraw {
            transparent: true,
            indirect: false,
        },
        TestDraw {
            transparent: false,
            indirect: true,
        },
    ]
}
