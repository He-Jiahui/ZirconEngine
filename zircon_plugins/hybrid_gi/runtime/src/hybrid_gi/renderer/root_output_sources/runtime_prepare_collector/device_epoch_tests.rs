use super::*;
use zircon_runtime::rhi::{DeviceGeneration, DeviceId};

fn device_epoch(generation: u64) -> RuntimePrepareDeviceEpoch {
    RuntimePrepareDeviceEpoch::new(DeviceId::new(23), DeviceGeneration::new(generation))
}

#[test]
fn hybrid_gi_device_epoch_change_is_explicit_and_preserves_frame_ordering() {
    let mut state = HybridGiRuntimePrepareCollectorState::default();
    state.collector_frame_index = 19;

    assert!(state.activate_device_epoch(device_epoch(5)));
    assert!(!state.activate_device_epoch(device_epoch(5)));
    assert!(state.activate_device_epoch(device_epoch(6)));
    assert_eq!(state.active_device_epoch, Some(device_epoch(6)));
    assert!(state.gpu_resources.is_none());
    assert!(state.radiance_cache_instances.is_empty());
    assert_eq!(state.collector_frame_index, 19);
}

#[test]
fn hybrid_gi_activates_device_epoch_before_feature_early_returns() {
    let source = include_str!("../runtime_prepare_collector.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let collect = source
        .find("fn collect(")
        .expect("hybrid GI collector implementation");
    let collect = &source[collect..];
    let activation = collect
        .find("state.activate_device_epoch(context.device_epoch())")
        .expect("hybrid GI device epoch activation");
    let prepared_frame = collect
        .find("let Some(prepared_frame)")
        .expect("prepared-frame early return");
    let extract = collect
        .find("let Some(extract)")
        .expect("feature extract early return");
    let admission = collect
        .find("if !context.gpu_work_admitted()")
        .expect("shared GPU work admission gate");
    let ensure_instance = collect
        .find("state.ensure_instance(")
        .expect("GPU instance creation");

    assert!(activation < prepared_frame);
    assert!(activation < extract);
    assert!(admission < ensure_instance);
    assert!(source.contains("self.gpu_resources = None;"));
    assert!(source.contains("self.radiance_cache_instances.clear();"));
}
