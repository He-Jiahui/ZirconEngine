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

#[test]
fn dynamic_event_dispatch_fans_out_to_plugin_handlers_deterministically() {
    let sound = DefaultSoundManager::default();
    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: "sound.dynamic.weapon.fire".to_string(),
            display_name: "Weapon Fire".to_string(),
            payload_schema: "sound.dynamic.weapon_fire.v1".to_string(),
        })
        .unwrap();

    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: "timeline_sequence".to_string(),
            handler_id: "timeline-marker".to_string(),
            event_id: "sound.dynamic.weapon.fire".to_string(),
            display_name: "Timeline Marker".to_string(),
            priority: 10,
        })
        .unwrap();
    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: "gameplay_audio".to_string(),
            handler_id: "weapon-foley".to_string(),
            event_id: "sound.dynamic.weapon.fire".to_string(),
            display_name: "Weapon Foley".to_string(),
            priority: 20,
        })
        .unwrap();
    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: "analytics".to_string(),
            handler_id: "combat-counter".to_string(),
            event_id: "sound.dynamic.weapon.fire".to_string(),
            display_name: "Combat Counter".to_string(),
            priority: 20,
        })
        .unwrap();

    let invocation = SoundDynamicEventInvocation {
        event_id: "sound.dynamic.weapon.fire".to_string(),
        source_path: Some("Timeline/Combat/Weapon".to_string()),
        time_seconds: 4.0,
        payload_schema: "sound.dynamic.weapon_fire.v1".to_string(),
        payload: vec![7, 9],
    };
    sound.submit_dynamic_event(invocation.clone()).unwrap();

    let deliveries = sound.dispatch_dynamic_events().unwrap();
    assert_eq!(deliveries.len(), 3);
    assert_eq!(deliveries[0].handler.plugin_id, "analytics");
    assert_eq!(deliveries[0].handler.handler_id, "combat-counter");
    assert_eq!(deliveries[1].handler.plugin_id, "gameplay_audio");
    assert_eq!(deliveries[1].handler.handler_id, "weapon-foley");
    assert_eq!(deliveries[2].handler.plugin_id, "timeline_sequence");
    assert_eq!(deliveries[2].handler.handler_id, "timeline-marker");
    assert!(deliveries
        .iter()
        .all(|delivery| delivery.invocation == invocation));
    assert!(sound.dispatch_dynamic_events().unwrap().is_empty());
    assert!(sound.drain_dynamic_events().unwrap().is_empty());
}

#[test]
fn dynamic_event_handlers_validate_event_ownership_and_unregister_cleanly() {
    let sound = DefaultSoundManager::default();
    assert!(matches!(
        sound
            .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
                plugin_id: "timeline_sequence".to_string(),
                handler_id: "missing-event".to_string(),
                event_id: "sound.dynamic.missing".to_string(),
                display_name: "Missing Event".to_string(),
                priority: 0,
            })
            .unwrap_err(),
        SoundError::UnknownDynamicEvent { .. }
    ));

    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: "sound.dynamic.ambient.stinger".to_string(),
            display_name: "Ambient Stinger".to_string(),
            payload_schema: "sound.dynamic.ambient_stinger.v1".to_string(),
        })
        .unwrap();
    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: "ambience".to_string(),
            handler_id: "stinger".to_string(),
            event_id: "sound.dynamic.ambient.stinger".to_string(),
            display_name: "Stinger".to_string(),
            priority: 0,
        })
        .unwrap();
    assert_eq!(sound.dynamic_event_handlers().unwrap().len(), 1);

    sound
        .unregister_dynamic_event_handler("ambience", "stinger")
        .unwrap();
    assert!(matches!(
        sound
            .unregister_dynamic_event_handler("ambience", "stinger")
            .unwrap_err(),
        SoundError::UnknownDynamicEventHandler { .. }
    ));

    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: "ambience".to_string(),
            handler_id: "stinger".to_string(),
            event_id: "sound.dynamic.ambient.stinger".to_string(),
            display_name: "Stinger".to_string(),
            priority: 0,
        })
        .unwrap();
    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: "sound.dynamic.ambient.stinger".to_string(),
            source_path: None,
            time_seconds: 0.0,
            payload_schema: "sound.dynamic.ambient_stinger.v1".to_string(),
            payload: Vec::new(),
        })
        .unwrap();
    sound
        .unregister_dynamic_event("sound.dynamic.ambient.stinger")
        .unwrap();

    assert!(sound.dynamic_event_handlers().unwrap().is_empty());
    assert!(sound.dispatch_dynamic_events().unwrap().is_empty());
}
