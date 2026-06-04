use super::super::*;

use super::support::{failing_abi_callback, register_abi_event_and_handler};

#[test]
fn dynamic_event_abi_callback_failure_maps_to_handler_failure_detail() {
    let sound = DefaultSoundManager::default();
    register_abi_event_and_handler(&sound, "abi_plugin", "abi_handler");
    sound
        .register_dynamic_event_abi_callback("abi_plugin", "abi_handler", failing_abi_callback)
        .unwrap();
    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: "sound.dynamic.abi".to_string(),
            source_path: None,
            time_seconds: 0.0,
            payload_schema: "sound.dynamic.abi.v1".to_string(),
            payload: Vec::new(),
        })
        .unwrap();

    let report = sound.execute_dynamic_events().unwrap();

    assert_eq!(report.executions.len(), 1);
    assert_eq!(
        report.executions[0].status,
        SoundDynamicEventExecutionStatus::Failed
    );
    assert_eq!(
        report.executions[0].detail.as_deref(),
        Some("abi callback rejected event")
    );
}
