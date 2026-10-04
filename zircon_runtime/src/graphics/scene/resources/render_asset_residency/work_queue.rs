use std::collections::{BTreeMap, VecDeque};

use super::{
    RenderAssetResidencyMutation, RenderAssetResidencyReleaseKind, RenderAssetResidencyRoute,
    RenderAssetResidencyTicket,
};

#[derive(Default)]
pub(crate) struct RenderAssetResidencyWorkQueue {
    semantic_blocks: BTreeMap<u64, RenderAssetResidencyTicket>,
    canonical_mesh_sets: BTreeMap<u64, RenderAssetResidencyTicket>,
    prepared_dependencies: BTreeMap<u64, RenderAssetResidencyTicket>,
    semantic_cancellations: VecDeque<RenderAssetResidencyTicket>,
}

impl RenderAssetResidencyWorkQueue {
    /// 本地移除未开始的请求；已派发的语义请求转入取消队列，GPU 退休类 release 留给提交终态与资源销毁路径。
    pub(crate) fn retain_mutation(&mut self, mutation: &RenderAssetResidencyMutation) {
        for release in mutation.releases() {
            match release.kind() {
                RenderAssetResidencyReleaseKind::CancelPending
                | RenderAssetResidencyReleaseKind::DropTerminal => {
                    let ticket = release.ticket();
                    let was_queued = self.remove_request(&ticket);
                    if !was_queued && ticket.route() == RenderAssetResidencyRoute::SemanticBlocks {
                        self.semantic_cancellations.push_back(ticket);
                    }
                }
                RenderAssetResidencyReleaseKind::RetireInFlight
                | RenderAssetResidencyReleaseKind::RetireResident => {}
            }
        }
        for request in mutation.requests().iter().cloned() {
            self.request_route_mut(request.route())
                .insert(request.id().raw(), request);
        }
    }

    pub(crate) fn pending_request_count(&self) -> usize {
        self.semantic_blocks
            .len()
            .saturating_add(self.canonical_mesh_sets.len())
            .saturating_add(self.prepared_dependencies.len())
    }

    pub(crate) fn pending_request_count_for_route(
        &self,
        route: RenderAssetResidencyRoute,
    ) -> usize {
        self.request_route(route).len()
    }

    pub(crate) fn pending_semantic_cancellation_count(&self) -> usize {
        self.semantic_cancellations.len()
    }

    pub(crate) fn pop_next_semantic_cancellation(&mut self) -> Option<RenderAssetResidencyTicket> {
        self.semantic_cancellations.pop_front()
    }

    pub(crate) fn restore_semantic_cancellation_front(
        &mut self,
        ticket: RenderAssetResidencyTicket,
    ) {
        debug_assert_eq!(ticket.route(), RenderAssetResidencyRoute::SemanticBlocks);
        self.semantic_cancellations.push_front(ticket);
    }

    /// Keeps the request in place until the target owner has accepted it. This gives admission
    /// failure a rollback-free path and preserves stable ticket order under backpressure.
    pub(crate) fn try_admit_next_semantic<E>(
        &mut self,
        admit: impl FnOnce(RenderAssetResidencyTicket) -> Result<(), E>,
    ) -> Result<Option<RenderAssetResidencyTicket>, RenderAssetResidencyWorkQueueAdmissionFailure<E>>
    {
        let Some(ticket) = self
            .semantic_blocks
            .first_key_value()
            .map(|(_, ticket)| ticket.clone())
        else {
            return Ok(None);
        };
        if let Err(error) = admit(ticket.clone()) {
            return Err(RenderAssetResidencyWorkQueueAdmissionFailure { ticket, error });
        }
        let removed = self.remove_request(&ticket);
        debug_assert!(
            removed,
            "accepted semantic ticket must remain queued until commit"
        );
        Ok(Some(ticket))
    }

    fn remove_request(&mut self, ticket: &RenderAssetResidencyTicket) -> bool {
        let requests = self.request_route_mut(ticket.route());
        if requests
            .get(&ticket.id().raw())
            .is_none_or(|queued| queued != ticket)
        {
            return false;
        }
        requests.remove(&ticket.id().raw()).is_some()
    }

    fn request_route(
        &self,
        route: RenderAssetResidencyRoute,
    ) -> &BTreeMap<u64, RenderAssetResidencyTicket> {
        match route {
            RenderAssetResidencyRoute::SemanticBlocks => &self.semantic_blocks,
            RenderAssetResidencyRoute::CanonicalMeshSet => &self.canonical_mesh_sets,
            RenderAssetResidencyRoute::PreparedDependencies => &self.prepared_dependencies,
        }
    }

    fn request_route_mut(
        &mut self,
        route: RenderAssetResidencyRoute,
    ) -> &mut BTreeMap<u64, RenderAssetResidencyTicket> {
        match route {
            RenderAssetResidencyRoute::SemanticBlocks => &mut self.semantic_blocks,
            RenderAssetResidencyRoute::CanonicalMeshSet => &mut self.canonical_mesh_sets,
            RenderAssetResidencyRoute::PreparedDependencies => &mut self.prepared_dependencies,
        }
    }
}

#[derive(Debug)]
pub(crate) struct RenderAssetResidencyWorkQueueAdmissionFailure<E> {
    ticket: RenderAssetResidencyTicket,
    error: E,
}

impl<E> RenderAssetResidencyWorkQueueAdmissionFailure<E> {
    pub(crate) fn ticket(&self) -> RenderAssetResidencyTicket {
        self.ticket.clone()
    }

    pub(crate) const fn error(&self) -> &E {
        &self.error
    }

    pub(crate) fn into_parts(self) -> (RenderAssetResidencyTicket, E) {
        (self.ticket, self.error)
    }
}
