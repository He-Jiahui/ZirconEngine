use std::collections::BTreeMap;

use super::*;

#[test]
fn trace_region_inputs_project_only_prepared_runtime_scene_data() {
    let prepare = HybridGiPrepareFrame {
        scheduled_trace_region_ids: vec![9, 9, 404],
        ..HybridGiPrepareFrame::default()
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_trace_region_scene_data(BTreeMap::from([(
            9,
            HybridGiResolveTraceRegionSceneData::new(2016, 2048, 2080, 144, 96, [16, 32, 64]),
        )]))
        .build();

    let inputs = trace_region_inputs(&prepare, Some(&runtime));

    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].region_id, 9);
    assert_eq!(inputs[0].center_x_q, 2016);
    assert_eq!(inputs[0].center_z_q, 2080);
    assert_eq!(inputs[0].radius_q, 144);
    assert_eq!(inputs[0].coverage_q, 96);
    assert_eq!(inputs[0].rt_lighting_rgb, pack_rgb8([16, 32, 64]));
}
