use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_TOOL,
    ToolMessage,
};
use crate::core::tools::{
    AcquireOutcome, AcquireSetOutcome, DEFAULT_MAX_QUEUE_PER_RESOURCE, ExclusiveResource,
    ReleaseAllOutcome, ReleaseOutcome, ReleaseSetOutcome, ToolId, ToolLifecycleEvent,
    ToolResourceSet, ToolScheduleReport, ToolScheduler, WithdrawOutcome, WithdrawSetOutcome,
};

/// Thread-safe owner of the one editor-wide exclusive-resource scheduler.
#[derive(Clone, Debug)]
pub struct ToolSchedulerService {
    scheduler: Arc<Mutex<ToolScheduler>>,
    bus: SharedEditorMessageBus,
}

impl ToolSchedulerService {
    pub fn new(bus: SharedEditorMessageBus) -> Self {
        Self::with_max_queue_per_resource(bus, DEFAULT_MAX_QUEUE_PER_RESOURCE)
    }

    pub fn with_max_queue_per_resource(
        bus: SharedEditorMessageBus,
        max_queue_per_resource: usize,
    ) -> Self {
        Self {
            scheduler: Arc::new(Mutex::new(ToolScheduler::new(max_queue_per_resource))),
            bus,
        }
    }

    pub fn acquire(
        &self,
        tool: ToolId,
        resource: ExclusiveResource,
    ) -> ToolScheduleReport<AcquireOutcome> {
        let report = {
            let mut scheduler = self.lock();
            scheduler.acquire(tool, resource)
        };
        self.publish_events(report.events());
        report
    }

    pub fn release(
        &self,
        tool: &ToolId,
        resource: ExclusiveResource,
    ) -> ToolScheduleReport<ReleaseOutcome> {
        let report = {
            let mut scheduler = self.lock();
            scheduler.release(tool, resource)
        };
        self.publish_events(report.events());
        report
    }

    pub fn acquire_set(
        &self,
        tool: ToolId,
        resources: ToolResourceSet,
    ) -> ToolScheduleReport<AcquireSetOutcome> {
        let report = {
            let mut scheduler = self.lock();
            scheduler.acquire_set(tool, resources)
        };
        self.publish_events(report.events());
        report
    }

    pub fn release_set(
        &self,
        tool: &ToolId,
        resources: &ToolResourceSet,
    ) -> ToolScheduleReport<ReleaseSetOutcome> {
        let report = {
            let mut scheduler = self.lock();
            scheduler.release_set(tool, resources)
        };
        self.publish_events(report.events());
        report
    }

    pub fn withdraw(
        &self,
        tool: &ToolId,
        resource: ExclusiveResource,
    ) -> ToolScheduleReport<WithdrawOutcome> {
        let report = {
            let mut scheduler = self.lock();
            scheduler.withdraw(tool, resource)
        };
        self.publish_events(report.events());
        report
    }

    pub fn withdraw_set(
        &self,
        tool: &ToolId,
        resources: &ToolResourceSet,
    ) -> ToolScheduleReport<WithdrawSetOutcome> {
        let report = {
            let mut scheduler = self.lock();
            scheduler.withdraw_set(tool, resources)
        };
        self.publish_events(report.events());
        report
    }

    pub fn release_all(&self, tool: &ToolId) -> ToolScheduleReport<ReleaseAllOutcome> {
        let report = {
            let mut scheduler = self.lock();
            scheduler.release_all(tool)
        };
        self.publish_events(report.events());
        report
    }

    pub fn holder(&self, resource: ExclusiveResource) -> Option<ToolId> {
        self.lock().holder(resource).cloned()
    }

    fn publish_events(&self, events: &[ToolLifecycleEvent]) {
        let topic = EditorTopic::parse(TOPIC_TOOL).expect("the built-in tool topic is valid");
        for event in events {
            self.bus.publish(
                topic.clone(),
                EditorMessage::new(EditorMessagePayload::Tool(ToolMessage::Lifecycle(
                    event.clone(),
                ))),
            );
        }
    }

    fn lock(&self) -> MutexGuard<'_, ToolScheduler> {
        self.scheduler
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
