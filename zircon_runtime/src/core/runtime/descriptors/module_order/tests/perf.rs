use std::collections::HashSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

const MODULE_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn module_names() -> Vec<String> {
    (0..MODULE_COUNT)
        .map(|index| format!("runtime.generated.module.closure.identity.{index:05}"))
        .collect()
}

fn unreserved_closure_count(module_names: &[String]) -> usize {
    let mut closure = HashSet::new();
    module_names
        .iter()
        .filter(|name| closure.insert(name.as_str()))
        .count()
}

fn reserved_closure_count(module_names: &[String]) -> usize {
    let mut closure = HashSet::with_capacity(module_names.len());
    module_names
        .iter()
        .filter(|name| closure.insert(name.as_str()))
        .count()
}

#[test]
fn optimization_batch_im_runtime623_module_closures_preallocate_graph_membership() {
    let source = include_str!("../../module_order.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert_eq!(
        production.matches("let mut closure: HashSet<&str>").count(),
        2
    );
    assert!(production.contains("HashSet::with_capacity(self.module_dependencies.len())"));
    assert!(production.contains("HashSet::with_capacity(self.module_dependents.len())"));
    assert_eq!(production.matches("if !closure.insert(current)").count(), 2);
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_im_runtime623_preallocated_module_closure_performance_evidence() {
    let module_names = module_names();
    assert_eq!(
        unreserved_closure_count(&module_names),
        reserved_closure_count(&module_names)
    );

    black_box(unreserved_closure_count(black_box(&module_names)));
    black_box(reserved_closure_count(black_box(&module_names)));

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(unreserved_closure_count(black_box(&module_names)));
            unreserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(reserved_closure_count(black_box(&module_names)));
            reserved_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(reserved_closure_count(black_box(&module_names)));
            reserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(unreserved_closure_count(black_box(&module_names)));
            unreserved_samples.push(started.elapsed());
        }
    }

    let unreserved_p95 = percentile_95(&mut unreserved_samples);
    let reserved_p95 = percentile_95(&mut reserved_samples);
    println!(
        "RUNTIME623_PREALLOCATED_MODULE_CLOSURE_BENCH_V1 \
         modules={MODULE_COUNT} borrowed_identity=true \
         unreserved_p95_ns={} reserved_p95_ns={}",
        unreserved_p95.as_nanos(),
        reserved_p95.as_nanos(),
    );
    assert!(
        reserved_p95.as_nanos() * 100 <= unreserved_p95.as_nanos() * 85,
        "preallocated closure P95 {:?} exceeded 85% of unreserved P95 {:?}",
        reserved_p95,
        unreserved_p95,
    );
}
