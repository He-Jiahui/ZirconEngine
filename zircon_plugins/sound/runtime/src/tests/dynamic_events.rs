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
fn dynamic_event_executor_registration_requires_registered_handler() {
    let sound = DefaultSoundManager::default();

    assert!(matches!(
        sound
            .register_dynamic_event_executor("missing", "handler", |_| Ok(()))
            .unwrap_err(),
        SoundError::UnknownDynamicEventHandler { .. }
    ));

    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: "sound.dynamic.registered".to_string(),
            display_name: "Registered".to_string(),
            payload_schema: "sound.dynamic.registered.v1".to_string(),
        })
        .unwrap();
    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: "registered_plugin".to_string(),
            handler_id: "registered_handler".to_string(),
            event_id: "sound.dynamic.registered".to_string(),
            display_name: "Registered Handler".to_string(),
            priority: 0,
        })
        .unwrap();

    sound
        .register_dynamic_event_executor("registered_plugin", "registered_handler", |_| Ok(()))
        .unwrap();
}

#[test]
fn dynamic_event_execution_reports_success_failure_and_missing_executors_in_order() {
    let sound = DefaultSoundManager::default();
    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: "sound.dynamic.weapon.fire".to_string(),
            display_name: "Weapon Fire".to_string(),
            payload_schema: "sound.dynamic.weapon_fire.v1".to_string(),
        })
        .unwrap();
    for (plugin_id, handler_id, priority) in [
        ("timeline_sequence", "timeline-marker", 10),
        ("gameplay_audio", "weapon-foley", 20),
        ("analytics", "combat-counter", 20),
    ] {
        sound
            .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
                plugin_id: plugin_id.to_string(),
                handler_id: handler_id.to_string(),
                event_id: "sound.dynamic.weapon.fire".to_string(),
                display_name: handler_id.to_string(),
                priority,
            })
            .unwrap();
    }

    let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let analytics_calls = calls.clone();
    sound
        .register_dynamic_event_executor("analytics", "combat-counter", move |delivery| {
            analytics_calls
                .lock()
                .unwrap()
                .push(delivery.handler.plugin_id.clone());
            Ok(())
        })
        .unwrap();
    let gameplay_calls = calls.clone();
    sound
        .register_dynamic_event_executor("gameplay_audio", "weapon-foley", move |delivery| {
            gameplay_calls
                .lock()
                .unwrap()
                .push(delivery.handler.plugin_id.clone());
            Err("foley unavailable".to_string())
        })
        .unwrap();

    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: "sound.dynamic.weapon.fire".to_string(),
            source_path: Some("Timeline/Combat/Weapon".to_string()),
            time_seconds: 4.0,
            payload_schema: "sound.dynamic.weapon_fire.v1".to_string(),
            payload: vec![7, 9],
        })
        .unwrap();

    let report = sound.execute_dynamic_events().unwrap();
    assert_eq!(report.executions.len(), 3);
    assert_eq!(report.executions[0].delivery.handler.plugin_id, "analytics");
    assert_eq!(
        report.executions[0].status,
        SoundDynamicEventExecutionStatus::Succeeded
    );
    assert_eq!(
        report.executions[1].delivery.handler.plugin_id,
        "gameplay_audio"
    );
    assert_eq!(
        report.executions[1].status,
        SoundDynamicEventExecutionStatus::Failed
    );
    assert_eq!(
        report.executions[1].detail.as_deref(),
        Some("foley unavailable")
    );
    assert_eq!(
        report.executions[2].delivery.handler.plugin_id,
        "timeline_sequence"
    );
    assert_eq!(
        report.executions[2].status,
        SoundDynamicEventExecutionStatus::SkippedMissingExecutor
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["analytics".to_string(), "gameplay_audio".to_string()]
    );
    assert!(sound
        .execute_dynamic_events()
        .unwrap()
        .executions
        .is_empty());
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

#[test]
fn dynamic_event_unregistering_event_removes_matching_executors() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundDynamicEventDescriptor {
        id: "sound.dynamic.cleanup".to_string(),
        display_name: "Cleanup".to_string(),
        payload_schema: "sound.dynamic.cleanup.v1".to_string(),
    };
    let handler = SoundDynamicEventHandlerDescriptor {
        plugin_id: "cleanup_plugin".to_string(),
        handler_id: "cleanup_handler".to_string(),
        event_id: "sound.dynamic.cleanup".to_string(),
        display_name: "Cleanup Handler".to_string(),
        priority: 0,
    };

    sound.register_dynamic_event(descriptor.clone()).unwrap();
    sound
        .register_dynamic_event_handler(handler.clone())
        .unwrap();
    sound
        .register_dynamic_event_executor("cleanup_plugin", "cleanup_handler", |_| Ok(()))
        .unwrap();
    sound
        .unregister_dynamic_event("sound.dynamic.cleanup")
        .unwrap();

    sound.register_dynamic_event(descriptor).unwrap();
    sound.register_dynamic_event_handler(handler).unwrap();
    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: "sound.dynamic.cleanup".to_string(),
            source_path: None,
            time_seconds: 0.0,
            payload_schema: "sound.dynamic.cleanup.v1".to_string(),
            payload: Vec::new(),
        })
        .unwrap();

    let report = sound.execute_dynamic_events().unwrap();
    assert_eq!(report.executions.len(), 1);
    assert_eq!(
        report.executions[0].status,
        SoundDynamicEventExecutionStatus::SkippedMissingExecutor
    );
}

#[test]
fn configure_mixer_removes_executors_for_removed_dynamic_events() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundDynamicEventDescriptor {
        id: "sound.dynamic.graph_cleanup".to_string(),
        display_name: "Graph Cleanup".to_string(),
        payload_schema: "sound.dynamic.graph_cleanup.v1".to_string(),
    };
    let handler = SoundDynamicEventHandlerDescriptor {
        plugin_id: "graph_plugin".to_string(),
        handler_id: "graph_handler".to_string(),
        event_id: "sound.dynamic.graph_cleanup".to_string(),
        display_name: "Graph Handler".to_string(),
        priority: 0,
    };

    sound.register_dynamic_event(descriptor.clone()).unwrap();
    sound
        .register_dynamic_event_handler(handler.clone())
        .unwrap();
    sound
        .register_dynamic_event_executor("graph_plugin", "graph_handler", |_| Ok(()))
        .unwrap();
    sound
        .configure_mixer(SoundMixerGraph::default_stereo(48_000))
        .unwrap();

    sound.register_dynamic_event(descriptor).unwrap();
    sound.register_dynamic_event_handler(handler).unwrap();
    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: "sound.dynamic.graph_cleanup".to_string(),
            source_path: None,
            time_seconds: 0.0,
            payload_schema: "sound.dynamic.graph_cleanup.v1".to_string(),
            payload: Vec::new(),
        })
        .unwrap();

    let report = sound.execute_dynamic_events().unwrap();
    assert_eq!(report.executions.len(), 1);
    assert_eq!(
        report.executions[0].status,
        SoundDynamicEventExecutionStatus::SkippedMissingExecutor
    );
}
