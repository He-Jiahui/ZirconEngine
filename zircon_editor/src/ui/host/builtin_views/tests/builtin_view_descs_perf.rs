use super::*;

#[test]
fn optimization_batch_20260830cx_builtin_descriptor_retain_matches_legacy_projection() {
    let snapshot = EditorCapabilitySnapshot::default();
    let mut candidates = activity_view_descriptors();
    candidates.extend(activity_window_descriptors());
    candidates.push(ui_asset_editor_view_descriptor());
    candidates.push(welcome_view_descriptor());
    let legacy = candidates
        .clone()
        .into_iter()
        .map(with_builtin_required_capabilities)
        .filter(|descriptor| snapshot.allows_all(&descriptor.required_capabilities))
        .collect::<Vec<_>>();

    finalize_builtin_view_descriptors(&mut candidates, &snapshot);
    assert_eq!(candidates, legacy);
}

#[test]
fn optimization_batch_20260830cx_builtin_descriptor_retain_source_contract() {
    let source = include_str!("../builtin_view_descriptors.rs");
    let builtin = source
        .split("pub(crate) fn builtin_view_descriptors")
        .nth(1)
        .expect("builtin descriptor projection")
        .split("pub(crate) fn with_builtin_required_capabilities")
        .next()
        .expect("bounded builtin descriptor projection");

    assert!(builtin.contains("finalize_builtin_view_descriptors(&mut descriptors, snapshot)"));
    assert!(source.contains("descriptors.retain("));
    assert!(!builtin.contains(".collect()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260830cx_editor_builtin_descriptor_retain_p95() {
    #[derive(Clone)]
    struct DescriptorFixture {
        id: String,
        enabled: bool,
    }

    fn measure(fixtures: &[DescriptorFixture], retain: bool) -> u128 {
        let batches = (0..64).map(|_| fixtures.to_vec()).collect::<Vec<_>>();
        let started = std::time::Instant::now();
        for mut descriptors in batches {
            if retain {
                descriptors.retain(|descriptor| descriptor.enabled);
                std::hint::black_box(descriptors);
            } else {
                std::hint::black_box(
                    descriptors
                        .into_iter()
                        .filter(|descriptor| descriptor.enabled)
                        .collect::<Vec<_>>(),
                );
            }
        }
        started.elapsed().as_nanos()
    }

    let fixtures = (0..4_096)
        .map(|index| DescriptorFixture {
            id: format!("editor.view.{index:05}"),
            enabled: index % 3 != 0,
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for sample_index in 0..17 {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&fixtures, false));
            optimized_samples.push(measure(&fixtures, true));
        } else {
            optimized_samples.push(measure(&fixtures, true));
            legacy_samples.push(measure(&fixtures, false));
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "EDITOR341_BUILTIN_DESCRIPTOR_RETAIN_BENCH_V1 descriptors={} legacy_p95_ns={} optimized_p95_ns={} checksum={} target_ratio_bp=7000",
        fixtures.len(),
        legacy_p95,
        optimized_p95,
        fixtures
            .iter()
            .map(|descriptor| descriptor.id.len())
            .sum::<usize>(),
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "in-place builtin descriptor retain P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
    );
}
