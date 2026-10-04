use std::collections::HashMap;

use crate::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceEventReceiver, ResourceEventTryRecvError,
    ResourceKind, UntypedResourceHandle,
};

pub(super) struct RenderSceneGeometryReplayState {
    receiver: Option<ResourceEventReceiver>,
    pending_revisions: HashMap<UntypedResourceHandle, u64>,
    applied_revisions: HashMap<UntypedResourceHandle, u64>,
    resync_required: bool,
    // A replacement cursor is required before a disconnect resync can be acknowledged.
    reconnect_required: bool,
}

pub(super) struct RenderSceneGeometryReplayBatch {
    pub(super) resources: Vec<(UntypedResourceHandle, u64)>,
    pub(super) resync: bool,
}

impl RenderSceneGeometryReplayState {
    pub(super) fn new(receiver: Option<ResourceEventReceiver>) -> Self {
        Self {
            receiver,
            pending_revisions: HashMap::new(),
            applied_revisions: HashMap::new(),
            resync_required: false,
            reconnect_required: false,
        }
    }

    // 事件先折叠为每项资源的最新 pending 修订；完整重放、重同步和重连条件满足后，提交才推进 applied 游标。
    pub(super) fn drain(
        &mut self,
        limit: usize,
        is_relevant: impl Fn(UntypedResourceHandle) -> bool,
    ) -> RenderSceneGeometryReplayBatch {
        if limit == 0 {
            return self.batch();
        }
        let mut events = Vec::new();
        let mut disconnected = false;
        if let Some(receiver) = self.receiver.as_ref().filter(|_| !self.resync_required) {
            for _ in 0..limit {
                match receiver.try_recv() {
                    Ok(event) => events.push(event),
                    Err(ResourceEventTryRecvError::Empty) => break,
                    Err(ResourceEventTryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                    Err(ResourceEventTryRecvError::Lagged(_))
                    | Err(ResourceEventTryRecvError::SequenceExhausted) => {
                        self.require_resync();
                        break;
                    }
                }
            }
        }
        if disconnected {
            self.receiver = None;
            self.reconnect_required = true;
            self.require_resync();
        }
        for event in events {
            if is_relevant(UntypedResourceHandle::new(event.id, event.resource_kind)) {
                self.record_event(event);
            }
        }
        self.batch()
    }

    pub(super) fn receiver_missing(&self) -> bool {
        self.receiver.is_none()
    }

    pub(super) fn install_receiver(&mut self, receiver: ResourceEventReceiver) {
        self.receiver = Some(receiver);
        self.reconnect_required = false;
    }

    fn batch(&self) -> RenderSceneGeometryReplayBatch {
        RenderSceneGeometryReplayBatch {
            resources: self
                .pending_revisions
                .iter()
                .map(|(resource, revision)| (*resource, *revision))
                .collect(),
            resync: self.resync_required,
        }
    }

    pub(super) fn commit(&mut self, resync_complete: bool, replay_complete: bool) {
        if !replay_complete || self.reconnect_required || (self.resync_required && !resync_complete)
        {
            return;
        }
        self.applied_revisions
            .extend(self.pending_revisions.drain());
        self.resync_required = false;
    }

    pub(super) fn retain_dependencies(
        &mut self,
        is_relevant: impl Fn(UntypedResourceHandle) -> bool,
    ) {
        self.pending_revisions
            .retain(|resource, _| is_relevant(*resource));
        self.applied_revisions
            .retain(|resource, _| is_relevant(*resource));
    }

    #[cfg(test)]
    fn tracked_revision_count(&self) -> usize {
        self.pending_revisions.len() + self.applied_revisions.len()
    }

    fn require_resync(&mut self) {
        self.resync_required = true;
    }

    fn record_event(&mut self, event: ResourceEvent) {
        if !matches!(
            event.resource_kind,
            ResourceKind::Mesh | ResourceKind::Model
        ) || !matches!(
            event.kind,
            ResourceEventKind::Added
                | ResourceEventKind::Updated
                | ResourceEventKind::Removed
                | ResourceEventKind::ReloadFailed
        ) {
            return;
        }
        let resource = UntypedResourceHandle::new(event.id, event.resource_kind);
        if self
            .pending_revisions
            .get(&resource)
            .is_some_and(|revision| *revision >= event.revision)
            || self
                .applied_revisions
                .get(&resource)
                .is_some_and(|revision| *revision >= event.revision)
        {
            return;
        }
        self.pending_revisions.insert(resource, event.revision);
    }
}

#[cfg(test)]
#[path = "tests/geometry_replay.rs"]
mod tests;
