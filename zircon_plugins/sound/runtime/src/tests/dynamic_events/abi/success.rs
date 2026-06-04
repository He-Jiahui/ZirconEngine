use super::super::*;

use super::support::{capture_abi_callback, register_abi_event_and_handler};

#[test]
fn dynamic_event_abi_callback_receives_projected_delivery_request() {
    let sound = DefaultSoundManager::default();
    register_abi_event_and_handler(&sound, "abi_plugin", "abi_handler");
    sound
        .register_dynamic_event_abi_callback("abi_plugin", "abi_handler", capture_abi_callback)
        .unwrap();
    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: "sound.dynamic.abi".to_string(),
            source_path: Some("Timeline/Combat/Impact".to_string()),
            time_seconds: 2.5,
            payload_schema: "sound.dynamic.abi.v1".to_string(),
            payload: b"payload".to_vec(),
        })
        .unwrap();

    let report = sound.execute_dynamic_events().unwrap();

    assert_eq!(report.executions.len(), 1);
    assert_eq!(
        report.executions[0].status,
        SoundDynamicEventExecutionStatus::Succeeded
    );
}
