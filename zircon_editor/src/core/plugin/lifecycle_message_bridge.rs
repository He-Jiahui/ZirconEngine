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

    /// 宿主帧驱动生命周期投递；有待处理批次时先重试，完成后再由后续帧从总线取新消息。
    /// 管理器忙时保留失败项及后续项；插件回调发布的新消息留待后续帧处理。
    pub fn pump(
        &self,
        bus: &SharedEditorMessageBus,
        manager: &EditorPluginManager,
    ) -> Result<EditorPluginLifecycleMessagePumpReport, EditorPluginTransitionError> {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let deliveries = if pending.is_empty() {
            bus.drain_deliveries(self.subscriber)
        } else {
            Vec::new()
        };
        let mut result = EditorPluginLifecycleMessagePumpReport {
            drained_messages: deliveries.len(),
            ..EditorPluginLifecycleMessagePumpReport::default()
        };
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

// 保持旧待处理项优先；处理失败后保留该项和后续项，避免总线已排空时丢失生命周期事实。
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

// 仅把匹配主题和事实类型的 Play 边界、文档开闭存转成外部生命周期；杂项广播不能仅凭载荷触发插件。
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
#[path = "tests/lifecycle_message_bridge.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/lifecycle_message_bridge_optimization_tests.rs"]
mod optimization_tests;
