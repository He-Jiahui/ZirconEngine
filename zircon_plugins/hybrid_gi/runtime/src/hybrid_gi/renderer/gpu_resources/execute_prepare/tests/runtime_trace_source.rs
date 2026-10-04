use std::collections::BTreeMap;

use super::*;

#[test]
fn runtime_irradiance_source_follows_runtime_parent_topology() {
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(BTreeMap::from([(300, 200)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([
            (
                100,
                HybridGiResolveRuntime::pack_rgb_and_weight([1.0, 0.0, 0.0], 1.0),
            ),
            (
                200,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.0, 0.0, 1.0], 1.0),
            ),
        ]))
        .build();

    let (support_q, rgb, scene_truth) = runtime_irradiance_source(Some(&runtime), 300);

    assert!(support_q > 0);
    assert_eq!(unpack_rgb8(rgb), [0, 0, 255]);
    assert!(!scene_truth);
}

#[test]
fn runtime_trace_source_does_not_walk_parent_lineage_when_runtime_topology_is_flat() {
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_rt_lighting_rgb(BTreeMap::from([(100, [240, 96, 48])]))
        .build();

    assert_eq!(runtime_trace_source(Some(&runtime), 300), (0, 0, false));
}

#[test]
fn runtime_trace_source_breaks_runtime_parent_cycles() {
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(BTreeMap::from([(200, 300), (300, 200)]))
        .with_probe_rt_lighting_rgb(BTreeMap::from([(200, [12, 24, 240])]))
        .build();

    let (support_q, rgb, scene_truth) = runtime_trace_source(Some(&runtime), 300);

    assert!(support_q > 0);
    assert_eq!(unpack_rgb8(rgb), [12, 24, 240]);
    assert!(!scene_truth);
}
