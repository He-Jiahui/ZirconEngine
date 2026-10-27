use zircon_runtime_interface::runtime_api::{
    ZrRuntimeProjectSceneTransitionPolicyV1, ZrRuntimeProjectSceneTransitionRequestV1,
    ZrRuntimeProjectSceneTransitionStatusV1,
};
use zircon_runtime_interface::{
    ZrRuntimeCursorHostRequestV1, ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::core::framework::script::ScriptHostValue;
use crate::dynamic_api::session::registry::RuntimeFrameDemand;
use crate::scene::DefaultLevelManager;
use crate::script::{register_gameplay_host_module, CapabilitySet, HostExportRegistry};

#[path = "scene_transition_support.rs"]
mod support;
use support::*;

#[test]
fn scene_transition_result_getter_requires_transition_capability() {
    let exports = HostExportRegistry::default();
    register_gameplay_host_module(&exports).unwrap();
    let denied = exports
        .call_with_capabilities(
            "zr.zircon.gameplay",
            "scene_transition_result_json",
            vec![],
            &CapabilitySet::default(),
        )
        .expect_err("the capability gate must reject before requiring a runtime context");
    assert!(denied.to_string().contains("missing capability"));
}

#[test]
fn scene_transition_drain_replaces_play_world_runs_on_start_and_preserves_entry_on_stop() {
    let fixture = TransitionProject::new();
    let mut session = fixture.session();
    let authoring = DefaultLevelManager::default()
        .create_level(session.level.snapshot(), session.level.metadata());
    let original_handle = session.level.handle();
    let original_epoch = session.level.capture_world_replacement_epoch();
    let events = record_script_hooks(&session, false);
    session.tick_frame().expect("entry script should start");
    assert_eq!(
        host_call(&session, "scene_transition_result_json", vec![]),
        ScriptHostValue::String("null".to_owned())
    );

    let id = request(&session, TARGET);
    drain(&mut session);

    let completed = result(&session).expect("transition result should survive world replacement");
    assert_eq!(completed.request_id, id);
    assert_eq!(completed.scene_uri, TARGET);
    assert_eq!(
        completed.status,
        ZrRuntimeProjectSceneTransitionStatusV1::Succeeded
    );
    assert_eq!(completed.diagnostic, None);
    assert_eq!(session.level.handle(), original_handle);
    assert!(session.level.capture_world_replacement_epoch() > original_epoch);
    assert_eq!(session.level.metadata().asset_uri.as_deref(), Some(TARGET));
    assert_eq!(session.scene_uri.as_deref(), Some(TARGET));
    session.level.with_world(|world| {
        assert!(world.contains_entity(22));
        assert!(!world.contains_entity(11));
        assert!(world
            .get_resource::<ZrRuntimeProjectSceneTransitionRequestV1>()
            .is_none());
    });
    assert!(!events
        .lock()
        .unwrap()
        .iter()
        .any(|event| event == "target.onStart"));
    assert_eq!(session.frame_demand(), RuntimeFrameDemand::Immediate);

    session
        .tick_frame()
        .expect("replacement script should start");
    session
        .tick_frame()
        .expect("replacement script should keep updating");
    let recorded = events.lock().unwrap().clone();
    assert_eq!(
        recorded
            .iter()
            .filter(|event| *event == "target.onStart")
            .count(),
        1
    );
    let start = recorded
        .iter()
        .position(|event| event == "target.onStart")
        .unwrap();
    let update = recorded
        .iter()
        .position(|event| event == "target.onUpdate")
        .unwrap();
    assert!(
        start < update,
        "onStart must precede the incoming scene's first Update"
    );
    let ScriptHostValue::String(json) = host_call(&session, "scene_transition_result_json", vec![])
    else {
        panic!("result getter should return JSON");
    };
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap(),
        serde_json::to_value(completed).unwrap()
    );

    assert!(
        session.shutdown_before_library_unload(),
        "Stop should retire the Play session"
    );
    drop(session);
    authoring.with_world(|world| {
        assert!(
            world.contains_entity(11),
            "Stop must retain the original edit document"
        );
        assert!(!world.contains_entity(22));
    });
    assert_eq!(authoring.metadata().asset_uri.as_deref(), Some(ENTRY));
    let mut restarted = fixture.session();
    restarted.level.with_world(|world| {
        assert!(
            world.contains_entity(11),
            "a fresh Play must start at its original entry scene"
        );
        assert!(!world.contains_entity(22));
    });
    assert!(restarted.shutdown_before_library_unload());
}

#[test]
fn scene_transition_from_on_update_runs_destination_start_on_the_next_requested_frame() {
    let fixture = TransitionProject::new();
    let mut session = fixture.session();
    let events = record_script_hooks(&session, true);

    session
        .tick_frame()
        .expect("source onUpdate should request and complete travel");

    let completed =
        result(&session).expect("tick must consume gameplay travel without a host drain");
    assert_eq!(
        completed.status,
        ZrRuntimeProjectSceneTransitionStatusV1::Succeeded
    );
    assert_eq!(completed.scene_uri, TARGET);
    assert!(session
        .level
        .with_world(|world| world.contains_entity(22) && !world.contains_entity(11)));
    assert_eq!(session.frame_demand(), RuntimeFrameDemand::Immediate);
    assert_eq!(
        events.lock().unwrap().as_slice(),
        &["entry.onStart", "entry.onUpdate"]
    );

    session
        .tick_frame()
        .expect("requested frame must run destination lifecycle");

    assert_eq!(
        events.lock().unwrap().as_slice(),
        &[
            "entry.onStart",
            "entry.onUpdate",
            "target.onStart",
            "target.onUpdate",
        ]
    );
}

#[test]
fn scene_transition_failure_records_result_without_world_mutation_or_retry() {
    let fixture = TransitionProject::new();
    let mut session = fixture.session();
    let metadata = session.level.metadata();
    let scene_uri = session.scene_uri.clone();
    let generation = session.level.world_generation();
    let epoch = session.level.capture_world_replacement_epoch();
    let id = request(&session, "res://scenes/missing.scene.toml");

    drain(&mut session);

    let failed = result(&session).expect("failure should be observable");
    assert_eq!(failed.request_id, id);
    assert_eq!(
        failed.status,
        ZrRuntimeProjectSceneTransitionStatusV1::Failed
    );
    assert!(failed
        .diagnostic
        .as_ref()
        .is_some_and(|text| !text.is_empty()));
    assert_eq!(session.level.metadata(), metadata);
    assert_eq!(session.scene_uri, scene_uri);
    assert_eq!(session.level.world_generation(), generation);
    assert_eq!(session.level.capture_world_replacement_epoch(), epoch);
    session.level.with_world(|world| {
        assert!(world.contains_entity(11));
        assert!(!world.contains_entity(22));
        assert!(world
            .get_resource::<ZrRuntimeProjectSceneTransitionRequestV1>()
            .is_none());
    });
    drain(&mut session);
    assert_eq!(
        result(&session),
        Some(failed),
        "a consumed failure must not be retried"
    );
    assert_eq!(session.level.capture_world_replacement_epoch(), epoch);

    // Typed insertion/Deserialize can bypass the producer's lexical URI validation.
    session.level.with_world_mut(|world| {
        world.insert_resource(ZrRuntimeProjectSceneTransitionRequestV1 {
            request_id: 700,
            scene_uri: "res://scenes/../outside.scene.toml".to_owned(),
            policy: ZrRuntimeProjectSceneTransitionPolicyV1::ReplaceActive,
        });
    });
    drain(&mut session);
    let rejected = result(&session).expect("noncanonical requests must be reported");
    assert_eq!(rejected.request_id, 700);
    assert_eq!(
        rejected.status,
        ZrRuntimeProjectSceneTransitionStatusV1::Rejected
    );
    assert!(rejected
        .diagnostic
        .unwrap()
        .contains("canonical forward-slash segments"));
    assert_eq!(session.level.metadata(), metadata);
    assert_eq!(session.scene_uri, scene_uri);
    assert_eq!(session.level.world_generation(), generation);
    assert_eq!(session.level.capture_world_replacement_epoch(), epoch);
    assert!(session.level.with_world(|world| world.contains_entity(11)
        && world
            .get_resource::<ZrRuntimeProjectSceneTransitionRequestV1>()
            .is_none()));
}

#[test]
fn scene_transition_request_ids_continue_across_overwrites_and_world_replacements() {
    let fixture = TransitionProject::new();
    let mut session = fixture.session();
    let mut previous_id = 0;
    for uri in [TARGET, ENTRY, TARGET] {
        for _ in 0..32 {
            let id = request(&session, uri);
            assert!(
                id > previous_id,
                "request identity must survive world replacement"
            );
            previous_id = id;
        }
        drain(&mut session);
        let completed = result(&session).unwrap();
        assert_eq!(
            completed.request_id, previous_id,
            "the latest pending request wins"
        );
        assert_eq!(
            completed.status,
            ZrRuntimeProjectSceneTransitionStatusV1::Succeeded
        );
    }
}

#[test]
fn scene_transition_waits_for_in_flight_output_but_consumes_before_a_pending_platform_page() {
    let fixture = TransitionProject::new();
    let mut session = fixture.session();
    session.pending_host_request_output = Some(ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![ZrRuntimeHostRequestV1::cursor(
            ZrRuntimeCursorHostRequestV1::set_visible(false),
        )],
    ));
    let original_page = session.prepare_host_request_output().unwrap();
    let id = request(&session, TARGET);
    assert!(session.prepare_host_request_output().is_err());
    assert!(
        result(&session).is_none(),
        "an in-flight output still owns its transaction"
    );
    assert!(session.level.with_world(|world| world.contains_entity(11)));
    session.rollback_host_request_output();

    let retry_page = session.prepare_host_request_output().unwrap();
    assert_eq!(
        retry_page, original_page,
        "platform retry output must stay stable"
    );
    assert_eq!(result(&session).unwrap().request_id, id);
    assert!(session.level.with_world(|world| world.contains_entity(22)));
    session.commit_host_request_output();
}
