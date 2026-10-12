use super::*;

#[test]
fn take_neutral_readback_outputs_projects_and_consumes_gpu_readback() {
    let mut outputs = HybridGiReadbackOutputs::default();
    outputs.store_gpu_readback(Some(HybridGiGpuReadback::new(
        vec![(3, 5)],
        vec![11],
        vec![21],
        vec![(11, [7, 8, 9])],
        vec![(11, [10, 11, 12])],
        None,
    )));

    let neutral = outputs.take_neutral_readback_outputs();

    assert_eq!(neutral.cache_entries[0].key, 3);
    assert_eq!(neutral.cache_entries[0].value, 5);
    assert_eq!(neutral.completed_probe_ids, vec![11]);
    assert_eq!(neutral.completed_trace_region_ids, vec![21]);
    assert_eq!(neutral.probe_irradiance_rgb, vec![[7, 8, 9]]);
    assert_eq!(neutral.probe_rt_lighting_rgb, vec![[10, 11, 12]]);
    assert_eq!(
        outputs.take_neutral_readback_outputs(),
        RenderHybridGiReadbackOutputs::default()
    );
}
