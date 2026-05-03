use super::*;

#[test]
fn dynamic_event_registry_accepts_descriptors_and_drains_invocations() {
    let sound = DefaultSoundManager::default();
    assert!(sound.dynamic_event_catalog().unwrap().events.is_empty());

    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: "sound.dynamic.impact".to_string(),
            display_name: "Impact".to_string(),
            payload_schema: "sound.dynamic.impact.v1".to_string(),
        })
        .unwrap();

    let catalog = sound.dynamic_event_catalog().unwrap();
    assert_eq!(catalog.namespace, "sound.dynamic_events");
    assert_eq!(catalog.version, 1);
    assert_eq!(catalog.events.len(), 1);
    assert_eq!(catalog.events[0].id, "sound.dynamic.impact");
    assert_eq!(
        sound.mixer_snapshot().unwrap().graph.dynamic_events.events,
        catalog.events
    );

    let invocation = SoundDynamicEventInvocation {
        event_id: "sound.dynamic.impact".to_string(),
        source_path: Some("Timeline/Combat/Impact".to_string()),
        time_seconds: 1.25,
        payload_schema: "sound.dynamic.impact.v1".to_string(),
        payload: vec![1, 2, 3, 4],
    };
    sound.submit_dynamic_event(invocation.clone()).unwrap();

    assert_eq!(sound.drain_dynamic_events().unwrap(), vec![invocation]);
    assert!(sound.drain_dynamic_events().unwrap().is_empty());
}

#[test]
fn dynamic_event_registry_rejects_unknown_or_invalid_invocations() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: String::new(),
            display_name: "Invalid".to_string(),
            payload_schema: "sound.dynamic.invalid.v1".to_string(),
        })
        .unwrap_err()
        .to_string()
        .contains("descriptor"));

    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: "sound.dynamic.marker".to_string(),
            display_name: "Marker".to_string(),
            payload_schema: "sound.dynamic.marker.v1".to_string(),
        })
        .unwrap();

    assert!(matches!(
        sound
            .submit_dynamic_event(SoundDynamicEventInvocation {
                event_id: "sound.dynamic.missing".to_string(),
                source_path: None,
                time_seconds: 0.0,
                payload_schema: "sound.dynamic.marker.v1".to_string(),
                payload: Vec::new(),
            })
            .unwrap_err(),
        SoundError::UnknownDynamicEvent { .. }
    ));
    assert!(sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: "sound.dynamic.marker".to_string(),
            source_path: None,
            time_seconds: f32::NAN,
            payload_schema: "sound.dynamic.marker.v1".to_string(),
            payload: Vec::new(),
        })
        .unwrap_err()
        .to_string()
        .contains("time"));
    assert!(sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: "sound.dynamic.marker".to_string(),
            source_path: None,
            time_seconds: 0.0,
            payload_schema: "sound.dynamic.other.v1".to_string(),
            payload: Vec::new(),
        })
        .unwrap_err()
        .to_string()
        .contains("expects payload schema"));

    sound
        .unregister_dynamic_event("sound.dynamic.marker")
        .unwrap();
    assert!(matches!(
        sound
            .unregister_dynamic_event("sound.dynamic.marker")
            .unwrap_err(),
        SoundError::UnknownDynamicEvent { .. }
    ));
}
