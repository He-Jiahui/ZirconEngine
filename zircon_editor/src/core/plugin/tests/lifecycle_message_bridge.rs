use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::editor_message::{
    DocumentId, DocumentMessage, EditorMessage, EditorMessageInboxLimits, EditorMessagePayload,
    EditorTopic, ModeMessage, PlayStateKind, SharedEditorMessageBus, TOPIC_DOCUMENT, TOPIC_MODE,
};
use crate::core::plugin::sdk::lifecycle::{
    EditorPluginLifecycleError, EditorPluginLifecycleEvent, EditorPluginLifecycleStage,
};
use crate::core::plugin::{
    EditorPlugin, EditorPluginCatalog, EditorPluginDescriptor, EditorPluginLoadingPhase,
    EditorPluginTransitionError,
};

use super::{EditorPluginLifecycleMessageBridge, EditorPluginManager};

struct LifecycleProbe {
    descriptor: EditorPluginDescriptor,
    events: Mutex<Vec<EditorPluginLifecycleEvent>>,
}

struct BlockingLifecycleProbe {
    descriptor: EditorPluginDescriptor,
    entered: Arc<(Mutex<bool>, Condvar)>,
    release: Arc<(Mutex<bool>, Condvar)>,
    events: Mutex<Vec<EditorPluginLifecycleEvent>>,
}

struct FailingLifecycleProbe {
    descriptor: EditorPluginDescriptor,
}

impl EditorPlugin for LifecycleProbe {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn on_lifecycle_event(
        &self,
        event: &EditorPluginLifecycleEvent,
    ) -> Result<(), EditorPluginLifecycleError> {
        self.events
            .lock()
            .expect("lifecycle probe lock should not be poisoned")
            .push(event.clone());
        Ok(())
    }
}

impl EditorPlugin for BlockingLifecycleProbe {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn on_lifecycle_event(
        &self,
        event: &EditorPluginLifecycleEvent,
    ) -> Result<(), EditorPluginLifecycleError> {
        if event.stage() == &EditorPluginLifecycleStage::UiMessage {
            let (entered, entered_ready) = self.entered.as_ref();
            *entered
                .lock()
                .expect("lifecycle probe entered lock should not be poisoned") = true;
            entered_ready.notify_one();

            let (released, release_ready) = self.release.as_ref();
            let mut released = released
                .lock()
                .expect("lifecycle probe release lock should not be poisoned");
            while !*released {
                released = release_ready
                    .wait(released)
                    .expect("lifecycle probe release wait should not be poisoned");
            }
        }
        self.events
            .lock()
            .expect("lifecycle probe lock should not be poisoned")
            .push(event.clone());
        Ok(())
    }
}

impl EditorPlugin for FailingLifecycleProbe {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    fn on_lifecycle_event(
        &self,
        event: &EditorPluginLifecycleEvent,
    ) -> Result<(), EditorPluginLifecycleError> {
        if event.stage() == &EditorPluginLifecycleStage::EnteredPlayMode {
            return Err(EditorPluginLifecycleError::new(
                event.stage().clone(),
                "test lifecycle callback failure",
            ));
        }
        Ok(())
    }
}

fn active_probe() -> (EditorPluginManager, Arc<LifecycleProbe>) {
    let probe = Arc::new(LifecycleProbe {
        descriptor: EditorPluginDescriptor::new(
            "plugin.lifecycle.message-bridge",
            "Lifecycle Message Bridge",
            "lifecycle_message_bridge",
        ),
        events: Mutex::default(),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([(
        Arc::clone(&probe) as Arc<dyn EditorPlugin + Send + Sync>,
        PluginPackageManifest::new(
            "plugin.lifecycle.message-bridge",
            "Lifecycle Message Bridge",
        ),
    )]))
    .expect("the lifecycle probe catalog should be valid");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate the lifecycle probe");
    (manager, probe)
}

fn active_blocking_probe(
    entered: Arc<(Mutex<bool>, Condvar)>,
    release: Arc<(Mutex<bool>, Condvar)>,
) -> (Arc<EditorPluginManager>, Arc<BlockingLifecycleProbe>) {
    let probe = Arc::new(BlockingLifecycleProbe {
        descriptor: EditorPluginDescriptor::new(
            "plugin.lifecycle.message-bridge-blocking",
            "Lifecycle Message Bridge Blocking",
            "lifecycle_message_bridge_blocking",
        ),
        entered,
        release,
        events: Mutex::default(),
    });
    let manager = Arc::new(
        EditorPluginManager::new(EditorPluginCatalog::from_plugins([(
            Arc::clone(&probe) as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(
                "plugin.lifecycle.message-bridge-blocking",
                "Lifecycle Message Bridge Blocking",
            ),
        )]))
        .expect("the blocking lifecycle probe catalog should be valid"),
    );
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate the blocking lifecycle probe");
    (manager, probe)
}

fn active_failing_probes() -> EditorPluginManager {
    let first = Arc::new(FailingLifecycleProbe {
        descriptor: EditorPluginDescriptor::new(
            "plugin.lifecycle.message-bridge-failure-one",
            "Lifecycle Message Bridge Failure One",
            "lifecycle_message_bridge_failure_one",
        ),
    });
    let second = Arc::new(FailingLifecycleProbe {
        descriptor: EditorPluginDescriptor::new(
            "plugin.lifecycle.message-bridge-failure-two",
            "Lifecycle Message Bridge Failure Two",
            "lifecycle_message_bridge_failure_two",
        ),
    });
    let manager = EditorPluginManager::new(EditorPluginCatalog::from_plugins([
        (
            first as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(
                "plugin.lifecycle.message-bridge-failure-one",
                "Lifecycle Message Bridge Failure One",
            ),
        ),
        (
            second as Arc<dyn EditorPlugin + Send + Sync>,
            PluginPackageManifest::new(
                "plugin.lifecycle.message-bridge-failure-two",
                "Lifecycle Message Bridge Failure Two",
            ),
        ),
    ]))
    .expect("the failing lifecycle probe catalog should be valid");
    manager
        .advance_loading_phase(EditorPluginLoadingPhase::Default)
        .expect("the default phase should activate the failing lifecycle probes");
    manager
}

#[test]
fn pump_maps_only_playing_boundary_crossings_to_play_lifecycle_events() {
    let bus = SharedEditorMessageBus::default();
    let bridge = EditorPluginLifecycleMessageBridge::new(&bus)
        .expect("the bridge subscriber should register");
    let (manager, probe) = active_probe();
    let topic = EditorTopic::parse(TOPIC_MODE).expect("the mode topic should be valid");

    for (from, to) in [
        (PlayStateKind::Edit, PlayStateKind::Building),
        (PlayStateKind::Building, PlayStateKind::Playing),
        (PlayStateKind::Playing, PlayStateKind::Playing),
        (PlayStateKind::Playing, PlayStateKind::Edit),
    ] {
        bus.publish(
            topic.clone(),
            EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
                from,
                to,
            })),
        );
    }

    let report = bridge
        .pump(&bus, &manager)
        .expect("external lifecycle dispatch should succeed");

    assert_eq!(report.drained_messages(), 4);
    assert_eq!(report.lifecycle_messages(), 2);
    assert_eq!(report.plugin_callbacks(), 2);
    assert_eq!(report.callback_failures(), 0);
    assert_eq!(
        probe
            .events
            .lock()
            .expect("lifecycle probe lock should not be poisoned")
            .iter()
            .map(|event| event.stage().clone())
            .collect::<Vec<_>>(),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::EnteredPlayMode,
            EditorPluginLifecycleStage::ExitedPlayMode,
        ]
    );
}

#[test]
fn pump_ignores_broadcasts_that_do_not_use_a_lifecycle_topic() {
    let bus = SharedEditorMessageBus::default();
    let bridge = EditorPluginLifecycleMessageBridge::new(&bus)
        .expect("the bridge subscriber should register");
    let (manager, probe) = active_probe();
    let unrelated_topic =
        EditorTopic::parse("editor.focus").expect("the unrelated topic should be valid");

    bus.broadcast(
        unrelated_topic,
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
            from: PlayStateKind::Edit,
            to: PlayStateKind::Playing,
        })),
    );

    let report = bridge
        .pump(&bus, &manager)
        .expect("an unrelated broadcast should be ignored");

    assert_eq!(report.drained_messages(), 1);
    assert_eq!(report.lifecycle_messages(), 0);
    assert_eq!(report.plugin_callbacks(), 0);
    assert_eq!(
        probe
            .events
            .lock()
            .expect("lifecycle probe lock should not be poisoned")
            .iter()
            .map(|event| event.stage().clone())
            .collect::<Vec<_>>(),
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
        ]
    );
}

#[test]
fn pump_counts_each_failed_lifecycle_callback() {
    let bus = SharedEditorMessageBus::default();
    let bridge = EditorPluginLifecycleMessageBridge::new(&bus)
        .expect("the bridge subscriber should register");
    let manager = active_failing_probes();
    let topic = EditorTopic::parse(TOPIC_MODE).expect("the mode topic should be valid");

    bus.publish(
        topic,
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
            from: PlayStateKind::Edit,
            to: PlayStateKind::Playing,
        })),
    );

    let report = bridge
        .pump(&bus, &manager)
        .expect("callback failures should be retained as diagnostics");

    assert_eq!(report.drained_messages(), 1);
    assert_eq!(report.lifecycle_messages(), 1);
    assert_eq!(report.plugin_callbacks(), 2);
    assert_eq!(report.callback_failures(), 2);
}

#[test]
fn pump_maps_document_structure_events_but_not_dirty_or_focus_messages() {
    let bus = SharedEditorMessageBus::default();
    let bridge = EditorPluginLifecycleMessageBridge::new(&bus)
        .expect("the bridge subscriber should register");
    let (manager, probe) = active_probe();
    let topic = EditorTopic::parse(TOPIC_DOCUMENT).expect("the document topic should be valid");
    let document = DocumentId::new(42);

    for payload in [
        DocumentMessage::Opened { doc: document },
        DocumentMessage::DirtyChanged {
            doc: document,
            dirty: true,
        },
        DocumentMessage::FocusRequested { doc: document },
        DocumentMessage::Saved { doc: document },
        DocumentMessage::Closed { doc: document },
    ] {
        bus.publish(
            topic.clone(),
            EditorMessage::new(EditorMessagePayload::Document(payload)),
        );
    }

    let report = bridge
        .pump(&bus, &manager)
        .expect("external lifecycle dispatch should succeed");

    assert_eq!(report.drained_messages(), 5);
    assert_eq!(report.lifecycle_messages(), 3);
    assert_eq!(report.plugin_callbacks(), 3);
    let events = probe
        .events
        .lock()
        .expect("lifecycle probe lock should not be poisoned");
    let scene_events = events
        .iter()
        .filter(|event| event.stage() == &EditorPluginLifecycleStage::SceneChanged)
        .collect::<Vec<_>>();
    assert_eq!(scene_events.len(), 3);
    assert!(scene_events
        .iter()
        .all(|event| event.subject() == Some("42")));
}

#[test]
fn pump_retains_a_lossless_message_when_the_manager_is_mutating() {
    let bus = SharedEditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(1, 1, 1));
    let bridge = EditorPluginLifecycleMessageBridge::new(&bus)
        .expect("the bridge subscriber should register");
    let entered = Arc::new((Mutex::new(false), Condvar::new()));
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let (manager, probe) = active_blocking_probe(Arc::clone(&entered), Arc::clone(&release));

    let mutating_manager = Arc::clone(&manager);
    let mutating_callback = thread::spawn(move || {
        mutating_manager.dispatch_lifecycle_event_to_active(EditorPluginLifecycleEvent::new(
            EditorPluginLifecycleStage::UiMessage,
        ))
    });
    let (entered_lock, entered_ready) = entered.as_ref();
    let mut entered_lock = entered_lock
        .lock()
        .expect("lifecycle probe entered lock should not be poisoned");
    while !*entered_lock {
        entered_lock = entered_ready
            .wait(entered_lock)
            .expect("lifecycle probe entered wait should not be poisoned");
    }
    drop(entered_lock);

    let topic = EditorTopic::parse(TOPIC_MODE).expect("the mode topic should be valid");
    bus.publish(
        topic,
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
            from: PlayStateKind::Edit,
            to: PlayStateKind::Playing,
        })),
    );
    let error = bridge
        .pump(&bus, manager.as_ref())
        .expect_err("a concurrent manager mutation should defer the lossless delivery");
    assert!(matches!(
        error,
        EditorPluginTransitionError::MutationInProgress
    ));

    let topic = EditorTopic::parse(TOPIC_MODE).expect("the mode topic should be valid");
    bus.publish(
        topic.clone(),
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
            from: PlayStateKind::Playing,
            to: PlayStateKind::Edit,
        })),
    );
    let error = bridge
        .pump(&bus, manager.as_ref())
        .expect_err("a retained lifecycle delivery should be retried before draining more");
    assert!(matches!(
        error,
        EditorPluginTransitionError::MutationInProgress
    ));
    assert_eq!(
        bus.inbox_stats(bridge.subscriber)
            .expect("the bridge inbox should remain registered")
            .lossless_depth(),
        1,
        "the newer delivery should remain in the bus while the older item is pending"
    );
    let backpressured = bus.publish(
        topic.clone(),
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
            from: PlayStateKind::Edit,
            to: PlayStateKind::Playing,
        })),
    );
    assert!(backpressured.backpressured().contains(&bridge.subscriber));

    let (released, release_ready) = release.as_ref();
    *released
        .lock()
        .expect("lifecycle probe release lock should not be poisoned") = true;
    release_ready.notify_one();
    mutating_callback
        .join()
        .expect("the manager mutation thread should not panic")
        .expect("the blocking lifecycle callback should complete");

    let report = bridge
        .pump(&bus, manager.as_ref())
        .expect("the deferred lifecycle delivery should retry on the next pump");
    assert_eq!(report.drained_messages(), 0);
    assert_eq!(report.lifecycle_messages(), 1);
    assert_eq!(report.plugin_callbacks(), 1);
    assert_eq!(
        bus.inbox_stats(bridge.subscriber)
            .expect("the bridge inbox should remain registered")
            .lossless_depth(),
        1,
        "the newer delivery should wait in the bus until the next pump"
    );

    let report = bridge
        .pump(&bus, manager.as_ref())
        .expect("the newer lifecycle delivery should drain on the following pump");
    assert_eq!(report.drained_messages(), 1);
    assert_eq!(report.lifecycle_messages(), 1);
    assert_eq!(report.plugin_callbacks(), 1);

    let accepted_after_drain = bus.publish(
        topic,
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
            from: PlayStateKind::Edit,
            to: PlayStateKind::Playing,
        })),
    );
    assert!(accepted_after_drain
        .delivered()
        .contains(&bridge.subscriber));
    let report = bridge
        .pump(&bus, manager.as_ref())
        .expect("the delivery rejected by bus backpressure can be republished after space frees");
    assert_eq!(report.drained_messages(), 1);
    assert_eq!(report.lifecycle_messages(), 1);

    let stages = probe
        .events
        .lock()
        .expect("lifecycle probe lock should not be poisoned")
        .iter()
        .map(|event| event.stage().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        stages,
        vec![
            EditorPluginLifecycleStage::Loaded,
            EditorPluginLifecycleStage::Enabled,
            EditorPluginLifecycleStage::UiMessage,
            EditorPluginLifecycleStage::EnteredPlayMode,
            EditorPluginLifecycleStage::ExitedPlayMode,
            EditorPluginLifecycleStage::EnteredPlayMode,
        ]
    );
}
