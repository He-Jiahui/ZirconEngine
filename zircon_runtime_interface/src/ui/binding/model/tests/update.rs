use super::*;

const DIRTY_DOMAINS: [UiBindingDirtyDomain; 10] = [
    UiBindingDirtyDomain::Layout,
    UiBindingDirtyDomain::HitTest,
    UiBindingDirtyDomain::Render,
    UiBindingDirtyDomain::Style,
    UiBindingDirtyDomain::Text,
    UiBindingDirtyDomain::Input,
    UiBindingDirtyDomain::VisibleRange,
    UiBindingDirtyDomain::Accessibility,
    UiBindingDirtyDomain::Interaction,
    UiBindingDirtyDomain::Schedule,
];

#[test]
fn dirty_domain_bitset_preserves_first_seen_order_and_counts() {
    let report = UiBindingUpdateReport::from_updates(vec![
        UiBindingUpdate {
            status: UiBindingUpdateStatus::Applied,
            dirty: vec![
                UiBindingDirtyDomain::Render,
                UiBindingDirtyDomain::Layout,
                UiBindingDirtyDomain::Render,
            ],
            ..UiBindingUpdate::default()
        },
        UiBindingUpdate {
            status: UiBindingUpdateStatus::Unchanged,
            dirty: vec![UiBindingDirtyDomain::Text, UiBindingDirtyDomain::Layout],
            ..UiBindingUpdate::default()
        },
        UiBindingUpdate {
            status: UiBindingUpdateStatus::Rejected,
            dirty: vec![
                UiBindingDirtyDomain::Interaction,
                UiBindingDirtyDomain::Text,
            ],
            ..UiBindingUpdate::default()
        },
    ]);

    assert_eq!(report.applied_count, 1);
    assert_eq!(report.unchanged_count, 1);
    assert_eq!(report.rejected_count, 1);
    assert_eq!(
        report.dirty,
        vec![
            UiBindingDirtyDomain::Render,
            UiBindingDirtyDomain::Layout,
            UiBindingDirtyDomain::Text,
            UiBindingDirtyDomain::Interaction,
        ]
    );
}

#[test]
fn binding_dirty_flag_projection_preserves_domain_order() {
    let domains = UiBindingDirtyDomain::from_dirty_flags(UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        style: true,
        text: true,
        input: true,
        visible_range: true,
    });

    assert_eq!(domains.as_slice(), &DIRTY_DOMAINS[..7]);
}

#[test]
#[ignore = "release-only presized binding dirty-domain benchmark"]
fn presized_binding_dirty_domains_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const PROJECTION_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let flags = UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        style: true,
        text: true,
        input: true,
        visible_range: true,
    };
    let linear_projection = |flags: UiDirtyFlags| {
        let mut domains = Vec::new();
        if flags.layout {
            domains.push(UiBindingDirtyDomain::Layout);
        }
        if flags.hit_test {
            domains.push(UiBindingDirtyDomain::HitTest);
        }
        if flags.render {
            domains.push(UiBindingDirtyDomain::Render);
        }
        if flags.style {
            domains.push(UiBindingDirtyDomain::Style);
        }
        if flags.text {
            domains.push(UiBindingDirtyDomain::Text);
        }
        if flags.input {
            domains.push(UiBindingDirtyDomain::Input);
        }
        if flags.visible_range {
            domains.push(UiBindingDirtyDomain::VisibleRange);
        }
        domains
    };
    let mut unpresized_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut presized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_unpresized = || {
            let started = Instant::now();
            for _ in 0..PROJECTION_COUNT {
                black_box(linear_projection(black_box(flags)));
            }
            started.elapsed().as_nanos()
        };
        let measure_presized = || {
            let started = Instant::now();
            for _ in 0..PROJECTION_COUNT {
                black_box(UiBindingDirtyDomain::from_dirty_flags(black_box(flags)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            unpresized_samples.push(measure_unpresized());
            presized_samples.push(measure_presized());
        } else {
            presized_samples.push(measure_presized());
            unpresized_samples.push(measure_unpresized());
        }
    }

    unpresized_samples.sort_unstable();
    presized_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_PRESIZED_BINDING_DIRTY_DOMAINS_BENCH_V1 projections={PROJECTION_COUNT} domains=7 samples={SAMPLE_COUNT} unpresized_p50_ns={} presized_p50_ns={} unpresized_p95_ns={} presized_p95_ns={}",
        unpresized_samples[p50],
        presized_samples[p50],
        unpresized_samples[p95],
        presized_samples[p95],
    );
    assert!(
        presized_samples[p95].saturating_mul(5)
            <= unpresized_samples[p95].saturating_mul(4),
        "presized dirty-domain projection must improve P95 by at least 20%: unpresized={}ns presized={}ns",
        unpresized_samples[p95],
        presized_samples[p95],
    );
}

#[test]
#[ignore = "release-only binding dirty-domain bitset dedup benchmark"]
fn binding_dirty_domain_bitset_dedup_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const UPDATE_COUNT: usize = 4_096;
    const SAMPLE_COUNT: usize = 11;
    let updates = (0..UPDATE_COUNT)
        .map(|index| UiBindingUpdate {
            status: match index % 3 {
                0 => UiBindingUpdateStatus::Applied,
                1 => UiBindingUpdateStatus::Unchanged,
                _ => UiBindingUpdateStatus::Rejected,
            },
            dirty: DIRTY_DOMAINS
                .iter()
                .cycle()
                .skip(index % DIRTY_DOMAINS.len())
                .take(DIRTY_DOMAINS.len())
                .copied()
                .collect(),
            ..UiBindingUpdate::default()
        })
        .collect::<Vec<_>>();
    let mut linear_dirty = Vec::with_capacity(DIRTY_DOMAINS.len());
    let mut bitset_report = UiBindingUpdateReport {
        updates: updates.clone(),
        dirty: Vec::with_capacity(DIRTY_DOMAINS.len()),
        ..UiBindingUpdateReport::default()
    };
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut bitset_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let mut measure_linear = || {
            let started = Instant::now();
            let mut applied_count = 0u64;
            let mut unchanged_count = 0u64;
            let mut rejected_count = 0u64;
            linear_dirty.clear();
            for update in &updates {
                match update.status {
                    UiBindingUpdateStatus::Applied => applied_count += 1,
                    UiBindingUpdateStatus::Unchanged => unchanged_count += 1,
                    UiBindingUpdateStatus::Rejected => rejected_count += 1,
                }
                for domain in &update.dirty {
                    if !linear_dirty.contains(domain) {
                        linear_dirty.push(*domain);
                    }
                }
            }
            black_box((
                applied_count,
                unchanged_count,
                rejected_count,
                linear_dirty.len(),
            ));
            started.elapsed().as_nanos()
        };
        let mut measure_bitset = || {
            let started = Instant::now();
            bitset_report.recompute();
            black_box((
                bitset_report.applied_count,
                bitset_report.unchanged_count,
                bitset_report.rejected_count,
                bitset_report.dirty.len(),
            ));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            bitset_samples.push(measure_bitset());
        } else {
            bitset_samples.push(measure_bitset());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    bitset_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BINDING_DIRTY_DOMAIN_BITSET_DEDUP_BENCH_V1 updates={UPDATE_COUNT} domains={} samples={SAMPLE_COUNT} linear_p50_ns={} bitset_p50_ns={} linear_p95_ns={} bitset_p95_ns={}",
        DIRTY_DOMAINS.len(),
        linear_samples[p50],
        bitset_samples[p50],
        linear_samples[p95],
        bitset_samples[p95],
    );
    assert!(
        bitset_samples[p95].saturating_mul(5) <= linear_samples[p95].saturating_mul(4),
        "bitset dirty-domain dedup must improve P95 by at least 20%: linear={}ns bitset={}ns",
        linear_samples[p95],
        bitset_samples[p95],
    );
}
