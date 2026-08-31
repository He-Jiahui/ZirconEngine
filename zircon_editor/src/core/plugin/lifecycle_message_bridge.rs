//! Message-bus adapter for external editor-plugin lifecycle notifications.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::core::editor_message::{
    DocumentMessage, EditorMessageBusError, EditorMessageDelivery, EditorMessagePayload,
    EditorTopic, ModeMessage, PlayStateKind, SharedEditorMessageBus, TOPIC_DOCUMENT, TOPIC_MODE,
};

use super::manager::{EditorPluginManager, EditorPluginTransitionError};
use super::sdk::lifecycle::{EditorPluginLifecycleEvent, EditorPluginLifecycleStage};

/// Subscribes the plugin manager to the editor facts that have lifecycle semantics.
///
/// The bridge is pumped by the host rather than called by the bus while its lock is held. Plugin
/// callbacks can therefore publish further editor messages without re-entering the bus lock.
#[derive(Debug)]
pub struct EditorPluginLifecycleMessageBridge {
    subscriber: crate::core::editor_message::EditorSubscriberId,
    pending: Mutex<VecDeque<EditorMessageDelivery>>,
}

impl EditorPluginLifecycleMessageBridge {
    pub fn new(bus: &SharedEditorMessageBus) -> Result<Self, EditorMessageBusError> {
        let mode = EditorTopic::parse(TOPIC_MODE).expect("the built-in mode topic must be valid");
        let document =
            EditorTopic::parse(TOPIC_DOCUMENT).expect("the built-in document topic must be valid");
        let subscriber = bus.register_subscriber([mode, document])?;
        Ok(Self {
            subscriber,
            pending: Mutex::new(VecDeque::new()),
        })
    }

    pub fn pump(
        &self,
        bus: &SharedEditorMessageBus,
        manager: &EditorPluginManager,
    ) -> Result<EditorPluginLifecycleMessagePumpReport, EditorPluginTransitionError> {
        let deliveries = bus.drain_deliveries(self.subscriber);
        let mut result = EditorPluginLifecycleMessagePumpReport {
            drained_messages: deliveries.len(),
            ..EditorPluginLifecycleMessagePumpReport::default()
        };
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        process_lossless_queue(
            &mut pending,
            deliveries,
            |delivery| -> Result<(), EditorPluginTransitionError> {
                let Some(event) = lifecycle_event_for(delivery) else {
                    return Ok(());
                };
                let callback_report = manager.dispatch_lifecycle_event_to_active(event)?;
                result.lifecycle_messages = result.lifecycle_messages.saturating_add(1);
                result.plugin_callbacks = result
                    .plugin_callbacks
                    .saturating_add(callback_report.records().len());
                result.callback_failures = result
                    .callback_failures
                    .saturating_add(callback_report.diagnostics().len());
                Ok(())
            },
        )?;
        Ok(result)
    }
}

fn process_lossless_queue<T, E>(
    pending: &mut VecDeque<T>,
    fresh: Vec<T>,
    mut process: impl FnMut(&T) -> Result<(), E>,
) -> Result<(), E> {
    if pending.is_empty() {
        let mut fresh = fresh.into_iter();
        while let Some(item) = fresh.next() {
            if let Err(error) = process(&item) {
                pending.push_back(item);
                pending.extend(fresh);
                return Err(error);
            }
        }
        return Ok(());
    }

    pending.extend(fresh);
    while let Some(item) = pending.pop_front() {
        if let Err(error) = process(&item) {
            pending.push_front(item);
            return Err(error);
        }
    }
    Ok(())
}

/// Per-pump accounting for host diagnostics without exposing a second plugin state store.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorPluginLifecycleMessagePumpReport {
    drained_messages: usize,
    lifecycle_messages: usize,
    plugin_callbacks: usize,
    callback_failures: usize,
}

impl EditorPluginLifecycleMessagePumpReport {
    pub fn drained_messages(self) -> usize {
        self.drained_messages
    }

    pub fn lifecycle_messages(self) -> usize {
        self.lifecycle_messages
    }

    pub fn plugin_callbacks(self) -> usize {
        self.plugin_callbacks
    }

    pub fn callback_failures(self) -> usize {
        self.callback_failures
    }
}

fn lifecycle_event_for(delivery: &EditorMessageDelivery) -> Option<EditorPluginLifecycleEvent> {
    match (delivery.topic().as_str(), delivery.message().payload()) {
        (TOPIC_MODE, EditorMessagePayload::Mode(ModeMessage::PlayStateChanged { from, to })) => {
            play_mode_lifecycle_event(*from, *to)
        }
        (
            TOPIC_DOCUMENT,
            EditorMessagePayload::Document(
                DocumentMessage::Opened { doc }
                | DocumentMessage::Closed { doc }
                | DocumentMessage::Saved { doc },
            ),
        ) => Some(
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::SceneChanged)
                .with_subject(doc.value().to_string()),
        ),
        _ => None,
    }
}

fn play_mode_lifecycle_event(
    from: PlayStateKind,
    to: PlayStateKind,
) -> Option<EditorPluginLifecycleEvent> {
    match (from == PlayStateKind::Playing, to == PlayStateKind::Playing) {
        (false, true) => Some(EditorPluginLifecycleEvent::new(
            EditorPluginLifecycleStage::EnteredPlayMode,
        )),
        (true, false) => Some(EditorPluginLifecycleEvent::new(
            EditorPluginLifecycleStage::ExitedPlayMode,
        )),
        (false, false) | (true, true) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;

    use zircon_runtime::plugin::PluginPackageManifest;

    use crate::core::editor_message::{
        DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic, ModeMessage,
        PlayStateKind, SharedEditorMessageBus, TOPIC_DOCUMENT, TOPIC_MODE,
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
        let bus = SharedEditorMessageBus::default();
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
            ]
        );
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::VecDeque;
    use std::hint::black_box;
    use std::time::Instant;

    use super::process_lossless_queue;

    #[test]
    fn optimization_batch_di_direct_lifecycle_queue_matches_success_order() {
        let mut pending = VecDeque::new();
        let mut processed = Vec::new();

        process_lossless_queue(&mut pending, vec![1_u32, 2, 3, 4], |value| {
            processed.push(*value);
            Ok::<_, &'static str>(())
        })
        .expect("successful delivery batch");

        assert_eq!(processed, [1, 2, 3, 4]);
        assert!(pending.is_empty());
    }

    #[test]
    fn optimization_batch_di_direct_lifecycle_queue_retains_failure_and_remainder() {
        let mut pending = VecDeque::new();
        let mut processed = Vec::new();

        let error = process_lossless_queue(&mut pending, vec![1_u32, 2, 3, 4], |value| {
            processed.push(*value);
            (*value != 3).then_some(()).ok_or("blocked")
        })
        .expect_err("the third delivery should be retained");

        assert_eq!(error, "blocked");
        assert_eq!(processed, [1, 2, 3]);
        assert_eq!(pending, VecDeque::from([3, 4]));

        process_lossless_queue(&mut pending, vec![5], |value| {
            processed.push(*value);
            Ok::<_, &'static str>(())
        })
        .expect("retained deliveries should retry before fresh work");
        assert_eq!(processed, [1, 2, 3, 3, 4, 5]);
        assert!(pending.is_empty());
    }

    #[test]
    fn optimization_batch_di_preexisting_lifecycle_queue_keeps_priority() {
        let mut pending = VecDeque::from([7_u32, 8]);
        let mut processed = Vec::new();

        process_lossless_queue(&mut pending, vec![9, 10], |value| {
            processed.push(*value);
            Ok::<_, &'static str>(())
        })
        .expect("queued and fresh deliveries should drain");

        assert_eq!(processed, [7, 8, 9, 10]);
        assert!(pending.is_empty());
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_di_direct_lifecycle_queue_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const DELIVERIES_PER_SAMPLE: usize = 65_536;

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_queue_transfer(DELIVERIES_PER_SAMPLE, true));
                optimized_samples.push(measure_queue_transfer(DELIVERIES_PER_SAMPLE, false));
            } else {
                optimized_samples.push(measure_queue_transfer(DELIVERIES_PER_SAMPLE, false));
                legacy_samples.push(measure_queue_transfer(DELIVERIES_PER_SAMPLE, true));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR345_DIRECT_LIFECYCLE_DELIVERY_QUEUE_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "direct lifecycle delivery p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_queue_transfer(deliveries: usize, legacy: bool) -> u128 {
        let fresh = (0..deliveries as u64).collect::<Vec<_>>();
        let mut pending = VecDeque::new();
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        if legacy {
            pending.extend(black_box(fresh));
            while let Some(value) = pending.pop_front() {
                checksum = checksum.wrapping_add(black_box(value));
            }
        } else {
            process_lossless_queue(&mut pending, black_box(fresh), |value| {
                checksum = checksum.wrapping_add(black_box(*value));
                Ok::<_, ()>(())
            })
            .expect("benchmark delivery processing succeeds");
        }
        black_box((checksum, pending));
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[index]
    }
}
