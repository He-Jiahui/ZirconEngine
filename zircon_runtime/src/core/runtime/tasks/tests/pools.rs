use super::{resolve_thread_counts, TaskPoolOptions};

fn assert_counts_are_conservative(options: &TaskPoolOptions, available_parallelism: usize) {
    let counts = resolve_thread_counts(options, available_parallelism);
    assert_eq!(
        counts.total_threads,
        counts
            .io_threads
            .saturating_add(counts.async_compute_threads)
            .saturating_add(counts.compute_threads),
        "reported physical workers must equal the three created pools"
    );
    assert!(counts.io_threads >= options.io.minimum_threads());
    assert!(counts.async_compute_threads >= options.async_compute.minimum_threads());
    assert!(counts.compute_threads >= options.compute.minimum_threads());
}

#[test]
fn low_host_parallelism_is_raised_to_cover_each_pool_minimum() {
    let options = TaskPoolOptions::default();
    for available_parallelism in [0, 1, 2] {
        let counts = resolve_thread_counts(&options, available_parallelism);
        assert_eq!(counts.total_threads, 3);
        assert_counts_are_conservative(&options, available_parallelism);
    }
}

#[test]
fn explicit_physical_budget_is_conserved_across_pool_assignments() {
    let options = TaskPoolOptions::with_num_threads(8);
    let counts = resolve_thread_counts(&options, 1);

    assert_eq!(counts.total_threads, 8);
    assert_counts_are_conservative(&options, 1);
}
