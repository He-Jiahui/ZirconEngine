use crate::ui::{
    ecs::{
        UiEcsDirtyDomainKind, UiEcsDirtyDomains, UiEcsNodeProjection, UiEcsProjectionSnapshot,
    },
    event_ui::{UiNodeId, UiTreeId},
    pipeline::UiPipelineStage,
};

#[test]
fn ecs_projection_node_falls_back_for_unsorted_wire_input() {
    let snapshot = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        Vec::new(),
        [9, 2, 7]
            .into_iter()
            .map(|node_id| UiEcsNodeProjection {
                node_id: UiNodeId::new(node_id),
                ..UiEcsNodeProjection::default()
            })
            .collect(),
    );

    assert_eq!(
        snapshot.node(UiNodeId::new(9)).map(|node| node.node_id),
        Some(UiNodeId::new(9))
    );
}

#[test]
#[ignore = "release-only ECS projection node lookup benchmark"]
fn ecs_projection_sorted_node_lookup_benchmark() {
    use std::{hint::black_box, time::Instant};

    const NODE_COUNT: u64 = 4_096;
    const PROBE_COUNT: usize = 100_000;
    const SAMPLE_COUNT: usize = 11;
    let snapshot = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        Vec::new(),
        (0..NODE_COUNT)
            .map(|node_id| UiEcsNodeProjection {
                node_id: UiNodeId::new(node_id),
                ..UiEcsNodeProjection::default()
            })
            .collect(),
    );
    let probes: Vec<_> = (0..PROBE_COUNT)
        .map(|probe| UiNodeId::new(NODE_COUNT - 1 - (probe as u64 % 16)))
        .collect();
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for node_id in &probes {
                black_box(
                    snapshot
                        .nodes
                        .iter()
                        .find(|node| node.node_id == *node_id)
                        .expect("benchmark probe must exist"),
                );
            }
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            for node_id in &probes {
                black_box(snapshot.node(*node_id).expect("benchmark probe must exist"));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    indexed_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_ECS_NODE_LOOKUP_BENCH_V1 nodes={NODE_COUNT} probes={PROBE_COUNT} samples={SAMPLE_COUNT} linear_p50_ns={} indexed_p50_ns={} linear_p95_ns={} indexed_p95_ns={}",
        linear_samples[p50],
        indexed_samples[p50],
        linear_samples[p95],
        indexed_samples[p95],
    );
    assert!(
        indexed_samples[p95].saturating_mul(5) <= linear_samples[p95].saturating_mul(4),
        "sorted lookup must improve P95 by at least 20%: linear={}ns indexed={}ns",
        linear_samples[p95],
        indexed_samples[p95],
    );
}

#[test]
#[ignore = "release-only ECS single-stage impact benchmark"]
fn ecs_projection_single_stage_impact_benchmark() {
    use std::{hint::black_box, time::Instant};

    const NODE_COUNT: u64 = 4_096;
    const PROBE_COUNT: usize = 100;
    const SAMPLE_COUNT: usize = 11;
    let snapshot = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        Vec::new(),
        (0..NODE_COUNT)
            .map(|node_id| UiEcsNodeProjection {
                node_id: UiNodeId::new(node_id),
                dirty: UiEcsDirtyDomains {
                    layout: true,
                    style: true,
                    text: true,
                    input: true,
                    picking: true,
                    accessibility: true,
                    render: true,
                    visible_range: true,
                },
                ..UiEcsNodeProjection::default()
            })
            .collect(),
    );
    let mut all_stage_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut focused_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_all_stages = || {
            let started = Instant::now();
            for _ in 0..PROBE_COUNT {
                black_box(
                    snapshot
                        .schedule_impacts()
                        .into_iter()
                        .find(|impact| impact.stage == UiPipelineStage::RenderExtract),
                );
            }
            started.elapsed().as_nanos()
        };
        let measure_focused = || {
            let started = Instant::now();
            for _ in 0..PROBE_COUNT {
                black_box(snapshot.schedule_impact(UiPipelineStage::RenderExtract));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            all_stage_samples.push(measure_all_stages());
            focused_samples.push(measure_focused());
        } else {
            focused_samples.push(measure_focused());
            all_stage_samples.push(measure_all_stages());
        }
    }

    all_stage_samples.sort_unstable();
    focused_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_STAGE_IMPACT_BENCH_V1 nodes={NODE_COUNT} probes={PROBE_COUNT} samples={SAMPLE_COUNT} all_stage_p50_ns={} focused_p50_ns={} all_stage_p95_ns={} focused_p95_ns={}",
        all_stage_samples[p50],
        focused_samples[p50],
        all_stage_samples[p95],
        focused_samples[p95],
    );
    assert!(
        focused_samples[p95].saturating_mul(5) <= all_stage_samples[p95].saturating_mul(4),
        "focused stage aggregation must improve P95 by at least 20%: all={}ns focused={}ns",
        all_stage_samples[p95],
        focused_samples[p95],
    );
}

#[test]
fn ecs_projection_single_stage_impact_matches_full_table_for_snapshot_and_delta() {
    let previous = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        Vec::new(),
        Vec::new(),
    );
    let current = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        Vec::new(),
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(4),
            dirty: UiEcsDirtyDomains {
                text: true,
                render: true,
                ..UiEcsDirtyDomains::default()
            },
            ..UiEcsNodeProjection::default()
        }],
    );
    let delta = current.diff_from(&previous);
    let stage = UiPipelineStage::RenderExtract;
    let snapshot_expected = current
        .schedule_impacts()
        .into_iter()
        .find(|impact| impact.stage == stage);
    let delta_expected = delta
        .schedule_impacts()
        .into_iter()
        .find(|impact| impact.stage == stage);

    assert_eq!(current.schedule_impact(stage), snapshot_expected);
    assert_eq!(delta.schedule_impact(stage), delta_expected);
    assert!(
        current
            .schedule_impact(UiPipelineStage::Diagnostics)
            .is_none()
    );
    assert!(
        delta
            .schedule_impact(UiPipelineStage::Diagnostics)
            .is_none()
    );
}

#[test]
#[ignore = "release-only ECS single-domain impact benchmark"]
fn ecs_projection_single_domain_impact_benchmark() {
    use std::{hint::black_box, time::Instant};

    const NODE_COUNT: u64 = 4_096;
    const PROBE_COUNT: usize = 100;
    const SAMPLE_COUNT: usize = 11;
    let snapshot = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        Vec::new(),
        (0..NODE_COUNT)
            .map(|node_id| UiEcsNodeProjection {
                node_id: UiNodeId::new(node_id),
                dirty: UiEcsDirtyDomains {
                    layout: true,
                    style: true,
                    text: true,
                    input: true,
                    picking: true,
                    accessibility: true,
                    render: true,
                    visible_range: true,
                },
                ..UiEcsNodeProjection::default()
            })
            .collect(),
    );
    let mut all_domain_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut focused_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_all_domains = || {
            let started = Instant::now();
            for _ in 0..PROBE_COUNT {
                black_box(
                    snapshot
                        .dirty_domain_impacts()
                        .into_iter()
                        .find(|impact| impact.domain == UiEcsDirtyDomainKind::Render),
                );
            }
            started.elapsed().as_nanos()
        };
        let measure_focused = || {
            let started = Instant::now();
            for _ in 0..PROBE_COUNT {
                black_box(snapshot.dirty_domain_impact(UiEcsDirtyDomainKind::Render));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            all_domain_samples.push(measure_all_domains());
            focused_samples.push(measure_focused());
        } else {
            focused_samples.push(measure_focused());
            all_domain_samples.push(measure_all_domains());
        }
    }

    all_domain_samples.sort_unstable();
    focused_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_DOMAIN_IMPACT_BENCH_V1 nodes={NODE_COUNT} probes={PROBE_COUNT} samples={SAMPLE_COUNT} all_domain_p50_ns={} focused_p50_ns={} all_domain_p95_ns={} focused_p95_ns={}",
        all_domain_samples[p50],
        focused_samples[p50],
        all_domain_samples[p95],
        focused_samples[p95],
    );
    assert!(
        focused_samples[p95].saturating_mul(5) <= all_domain_samples[p95].saturating_mul(4),
        "focused domain aggregation must improve P95 by at least 20%: all={}ns focused={}ns",
        all_domain_samples[p95],
        focused_samples[p95],
    );
}

#[test]
fn ecs_projection_single_domain_impact_matches_full_table_for_snapshot_and_delta() {
    let previous = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        Vec::new(),
        Vec::new(),
    );
    let current = UiEcsProjectionSnapshot::from_nodes(
        UiTreeId::new("ui.ecs"),
        Vec::new(),
        vec![UiEcsNodeProjection {
            node_id: UiNodeId::new(4),
            dirty: UiEcsDirtyDomains {
                text: true,
                render: true,
                ..UiEcsDirtyDomains::default()
            },
            ..UiEcsNodeProjection::default()
        }],
    );
    let delta = current.diff_from(&previous);
    let domain = UiEcsDirtyDomainKind::Render;
    let snapshot_expected = current
        .dirty_domain_impacts()
        .into_iter()
        .find(|impact| impact.domain == domain);
    let delta_expected = delta
        .dirty_domain_impacts()
        .into_iter()
        .find(|impact| impact.domain == domain);

    assert_eq!(current.dirty_domain_impact(domain), snapshot_expected);
    assert_eq!(delta.dirty_domain_impact(domain), delta_expected);
    assert!(
        current
            .dirty_domain_impact(UiEcsDirtyDomainKind::Input)
            .is_none()
    );
    assert!(
        delta
            .dirty_domain_impact(UiEcsDirtyDomainKind::Input)
            .is_none()
    );
}
