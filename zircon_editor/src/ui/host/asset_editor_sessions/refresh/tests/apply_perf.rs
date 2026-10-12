use std::collections::BTreeSet;

use super::{merge_refresh_instance_sets, ViewInstanceId};

#[test]
fn optimization_batch_20260830cv_refresh_set_append_preserves_sorted_union() {
    let direct = test_set(["asset.c", "asset.a", "asset.shared"]);
    let imports = test_set(["asset.d", "asset.b", "asset.shared"]);

    let merged = merge_refresh_instance_sets(direct, imports);
    assert_eq!(
        merged
            .iter()
            .map(|instance| instance.0.as_str())
            .collect::<Vec<_>>(),
        vec!["asset.a", "asset.b", "asset.c", "asset.d", "asset.shared"]
    );
}

#[test]
fn optimization_batch_20260830cv_refresh_set_append_source_contract() {
    let source = include_str!("../apply.rs");
    let merge = source
        .split("fn merge_refresh_instance_sets")
        .nth(1)
        .expect("refresh set merge implementation")
        .split("#[cfg(test)]")
        .next()
        .expect("bounded refresh set merge implementation");

    assert!(merge.contains("direct_sync.append(&mut import_sync)"));
    assert!(!merge.contains(".chain("));
    assert!(!merge.contains(".collect()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260830cv_editor_refresh_set_append_p95() {
    fn legacy_merge(
        direct: BTreeSet<ViewInstanceId>,
        imports: BTreeSet<ViewInstanceId>,
    ) -> BTreeSet<ViewInstanceId> {
        direct.into_iter().chain(imports).collect()
    }

    fn measure(
        direct: BTreeSet<ViewInstanceId>,
        imports: BTreeSet<ViewInstanceId>,
        merge: impl Fn(BTreeSet<ViewInstanceId>, BTreeSet<ViewInstanceId>) -> BTreeSet<ViewInstanceId>,
    ) -> u128 {
        let started = std::time::Instant::now();
        std::hint::black_box(merge(direct, imports));
        started.elapsed().as_nanos()
    }

    let direct = test_set((0..32_768).map(|index| format!("asset.{:05}", index * 2)));
    let imports = test_set((0..32_768).map(|index| format!("asset.{:05}", index * 2 + 1)));
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for sample_index in 0..17 {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(direct.clone(), imports.clone(), legacy_merge));
            optimized_samples.push(measure(
                direct.clone(),
                imports.clone(),
                merge_refresh_instance_sets,
            ));
        } else {
            optimized_samples.push(measure(
                direct.clone(),
                imports.clone(),
                merge_refresh_instance_sets,
            ));
            legacy_samples.push(measure(direct.clone(), imports.clone(), legacy_merge));
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "EDITOR339_REFRESH_SET_APPEND_BENCH_V1 direct={} imports={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
        direct.len(),
        imports.len(),
        legacy_p95,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "append refresh-set union P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
    );
}

fn test_set<I, S>(ids: I) -> BTreeSet<ViewInstanceId>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ids.into_iter().map(ViewInstanceId::new).collect()
}
