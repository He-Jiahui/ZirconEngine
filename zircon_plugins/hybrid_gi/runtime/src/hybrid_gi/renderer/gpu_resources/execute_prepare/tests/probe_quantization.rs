use std::collections::{BTreeMap, BTreeSet};

use super::*;

#[test]
fn probe_quantization_and_lineage_use_resolve_runtime_scene_truth() {
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_scene_data(BTreeMap::from([
            (3, HybridGiResolveProbeSceneData::new(2048, 2048, 2048, 192)),
            (7, HybridGiResolveProbeSceneData::new(2112, 2048, 2048, 96)),
        ]))
        .with_probe_parent_probes(BTreeMap::from([(7, 3)]))
        .with_trace_region_scene_data(BTreeMap::from([(
            9,
            HybridGiResolveTraceRegionSceneData::new(2112, 2048, 2048, 192, 128, [64, 96, 128]),
        )]))
        .build();

    assert_eq!(probe_position_x_q(Some(&runtime), 7), 2112);
    assert_eq!(probe_radius_q(Some(&runtime), 7), 96);
    assert_eq!(probe_parent_probe_id(Some(&runtime), 7), 3);
    assert_eq!(
        probe_resident_ancestors(Some(&runtime), &BTreeSet::from([3]), 7)[0],
        (3, 1)
    );
    assert!(probe_lineage_trace_support_q(Some(&runtime), &[9], 7) > 0);
    assert_eq!(
        probe_lineage_trace_lighting_rgb(Some(&runtime), &[9], 7),
        pack_rgb8([64, 96, 128])
    );
}

#[test]
fn scheduled_trace_regions_are_deduplicated_and_require_runtime_scene_data() {
    let runtime = HybridGiResolveRuntime::fixture()
        .with_trace_region_scene_data(BTreeMap::from([(
            9,
            HybridGiResolveTraceRegionSceneData::new(2048, 2048, 2048, 96, 128, [1, 2, 3]),
        )]))
        .build();

    assert_eq!(
        scheduled_live_trace_region_ids(Some(&runtime), &[9, 9, 404]),
        vec![9]
    );
    assert!(scheduled_live_trace_region_ids(None, &[9]).is_empty());
}
