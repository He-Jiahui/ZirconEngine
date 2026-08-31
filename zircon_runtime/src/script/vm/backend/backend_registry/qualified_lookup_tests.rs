use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::*;
use crate::script::MockVmBackend;

const FAMILY_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

struct CountingFamily {
    name: String,
    selector: String,
    family_name_calls: AtomicUsize,
    resolve_calls: AtomicUsize,
}

impl CountingFamily {
    fn new(name: String, selector: String) -> Self {
        Self {
            name,
            selector,
            family_name_calls: AtomicUsize::new(0),
            resolve_calls: AtomicUsize::new(0),
        }
    }
}

impl VmBackendFamily for CountingFamily {
    fn family_name(&self) -> &str {
        self.family_name_calls.fetch_add(1, Ordering::Relaxed);
        &self.name
    }

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        self.resolve_calls.fetch_add(1, Ordering::Relaxed);
        if selector == self.selector {
            Ok(Arc::new(MockVmBackend))
        } else {
            Err(VmError::UnknownBackend(selector.to_string()))
        }
    }

    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str)) {
        visitor(&self.selector);
    }
}

#[test]
fn optimization_batch_20260826bf_qualified_family_lookup_preserves_fallback_semantics() {
    let registry = VmBackendRegistry::new();
    let unrelated = Arc::new(CountingFamily::new(
        "alpha".to_string(),
        "alpha:backend".to_string(),
    ));
    let target = Arc::new(CountingFamily::new(
        "zeta".to_string(),
        "zeta:backend".to_string(),
    ));
    let fallback = Arc::new(CountingFamily::new(
        "fallback".to_string(),
        "external:shared".to_string(),
    ));
    registry.register_family(unrelated.clone());
    registry.register_family(target.clone());
    registry.register_family(fallback.clone());
    unrelated.resolve_calls.store(0, Ordering::Relaxed);
    target.resolve_calls.store(0, Ordering::Relaxed);
    fallback.resolve_calls.store(0, Ordering::Relaxed);

    assert!(registry.resolve("zeta:backend").is_ok());
    assert_eq!(target.resolve_calls.load(Ordering::Relaxed), 1);
    assert_eq!(unrelated.resolve_calls.load(Ordering::Relaxed), 0);
    assert_eq!(fallback.resolve_calls.load(Ordering::Relaxed), 0);

    assert!(registry.resolve("external:shared").is_ok());
    assert_eq!(fallback.resolve_calls.load(Ordering::Relaxed), 1);
}

#[test]
fn optimization_batch_20260826bf_qualified_family_lookup_eliminates_registry_scan() {
    let (registry, families) = registry_with_counting_families(FAMILY_COUNT);
    for family in &families {
        family.family_name_calls.store(0, Ordering::Relaxed);
    }

    assert!(registry.resolve("family-4095:backend").is_ok());
    assert_eq!(
        families
            .iter()
            .map(|family| family.family_name_calls.load(Ordering::Relaxed))
            .sum::<usize>(),
        0
    );

    let source = include_str!("../backend_registry.rs");
    let qualified_branch = source
        .split("if let Some((family_name, _))")
        .nth(1)
        .expect("qualified selector branch must remain")
        .split("for family in families")
        .next()
        .expect("qualified selector branch must terminate");
    assert!(qualified_branch.contains(".get(family_name)"));
    assert!(!qualified_branch.contains(".iter().find"));
}

#[test]
#[ignore = "release-only managed performance gate"]
fn optimization_batch_20260826bf_qualified_family_lookup_p95() {
    let (registry, _) = registry_with_counting_families(FAMILY_COUNT);
    let selector = "family-4095:backend";
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            baseline.push(measure(|| legacy_resolve(&registry, selector)));
            optimized.push(measure(|| registry.resolve(selector)));
        } else {
            optimized.push(measure(|| registry.resolve(selector)));
            baseline.push(measure(|| legacy_resolve(&registry, selector)));
        }
    }

    let baseline_p50 = percentile(&mut baseline.clone(), 50);
    let baseline_p95 = percentile(&mut baseline, 95);
    let optimized_p50 = percentile(&mut optimized.clone(), 50);
    let optimized_p95 = percentile(&mut optimized, 95);
    let reduction = percent_reduction(baseline_p95, optimized_p95);
    println!(
        "RUNTIME07_VM_QUALIFIED_FAMILY_INDEX_BENCH_V1 baseline_p50_ns={} baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_reduction_percent={reduction:.2} family_arc_clones_before={FAMILY_COUNT} family_name_comparisons_before={FAMILY_COUNT} family_arc_clones_after=1 map_lookups_after=1",
        baseline_p50.as_nanos(),
        baseline_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        reduction >= 90.0,
        "expected at least 90% P95 reduction, got {reduction:.2}%"
    );
}

fn registry_with_counting_families(count: usize) -> (VmBackendRegistry, Vec<Arc<CountingFamily>>) {
    let registry = VmBackendRegistry::new();
    let mut families = Vec::with_capacity(count);
    for index in 0..count {
        let name = format!("family-{index:04}");
        let family = Arc::new(CountingFamily::new(name.clone(), format!("{name}:backend")));
        registry.register_family(family.clone());
        families.push(family);
    }
    (registry, families)
}

fn legacy_resolve(
    registry: &VmBackendRegistry,
    selector: &str,
) -> Result<Arc<dyn VmBackend>, VmError> {
    let families = registry
        .lock_families()
        .values()
        .cloned()
        .collect::<Vec<_>>();
    if let Some((family_name, _)) = selector.split_once(':') {
        if let Some(family) = families
            .iter()
            .find(|family| family.family_name() == family_name)
        {
            return family.resolve(selector);
        }
    }
    for family in families {
        if let Ok(backend) = family.resolve(selector) {
            return Ok(backend);
        }
    }
    Err(VmError::UnknownBackend(selector.to_string()))
}

fn measure<T>(work: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(work());
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * percentile / 100]
}

fn percent_reduction(before: Duration, after: Duration) -> f64 {
    if before.is_zero() {
        return 0.0;
    }
    100.0 * (before.as_secs_f64() - after.as_secs_f64()) / before.as_secs_f64()
}
