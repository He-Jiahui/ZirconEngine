use std::collections::{HashMap, HashSet};

use crate::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceEventReceiver, ResourceEventTryRecvError,
    ResourceKind, UntypedResourceHandle,
};

pub(super) struct RenderSceneGeometryReplayState {
    receiver: Option<ResourceEventReceiver>,
    pending_revisions: HashMap<UntypedResourceHandle, u64>,
    applied_revisions: HashMap<UntypedResourceHandle, u64>,
    // Receiver order defines a remove/add lifecycle; content revisions may restart.
    removed_resources: HashSet<UntypedResourceHandle>,
    resync_required: bool,
    observed_active: Option<std::sync::Arc<crate::core::resource::ResourceActiveGeneration>>,
    pending_active: HashMap<UntypedResourceHandle, Option<crate::core::resource::ResourceActiveVersionKey>>,
    applied_active: HashMap<UntypedResourceHandle, Option<crate::core::resource::ResourceActiveVersionKey>>,
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
            removed_resources: HashSet::new(),
            resync_required: false,
            observed_active: None,
            pending_active: HashMap::new(),
            applied_active: HashMap::new(),
            reconnect_required: false,
        }
    }

    pub(super) fn observe_active(
        &mut self,
        active: &std::sync::Arc<crate::core::resource::ResourceActiveGeneration>,
        is_relevant: impl Fn(UntypedResourceHandle) -> bool,
    ) {
        if let Some(previous) = self.observed_active.as_ref() {
            if std::sync::Arc::ptr_eq(previous, active) { return; }
            if let Some(changed) = active.changed_since(previous) {
                for id in changed {
                    let row = active.row(id).or_else(|| previous.row(id));
                    let Some(row) = row else { continue; };
                    let resource = UntypedResourceHandle::new(id, row.record.kind);
                    if !is_relevant(resource) { continue; }
                    let key = active.key(id);
                    if self.pending_active.get(&resource) == Some(&key)
                        || (self.pending_active.get(&resource).is_none() && self.applied_active.get(&resource) == Some(&key))
                    { continue; }
                    self.pending_active.insert(resource, key);
                    // Active identity changes must bypass catalog-revision deduplication.
                    self.pending_revisions.insert(resource, row.record.revision);
                }
            } else { self.resync_required = true; }
        }
        self.observed_active = Some(std::sync::Arc::clone(active));
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
        self.applied_active.extend(self.pending_active.drain());
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
        self.pending_active.retain(|resource, _| is_relevant(*resource));
        self.applied_active.retain(|resource, _| is_relevant(*resource));
        self.removed_resources.retain(|resource| is_relevant(*resource));
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
        let recreated = event.kind == ResourceEventKind::Added
            && self.removed_resources.remove(&resource);
        let first_removal = event.kind == ResourceEventKind::Removed
            && self.removed_resources.insert(resource);
        if recreated || first_removal {
            // A committed removal cannot suppress a fresh cold Added event whose content
            // revision is equal or smaller. Only the ordered Removed -> Added edge resets
            // its old high-water mark; duplicate/stale content events retain normal dedup.
            if recreated {
                self.pending_revisions.remove(&resource);
                self.applied_revisions.remove(&resource);
            }
            self.pending_revisions.insert(resource, event.revision);
            return;
        }
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
#[path = "../tests/geometry_replay.rs"]
mod tests;
