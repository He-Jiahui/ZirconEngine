use zircon_runtime_interface::GatewaySessionIdentity;

use crate::core::play::WorldDomain;
use crate::core::sync::QualifiedWatchToken;

/// One retained hierarchy subscription, bound to the runtime session that issued it.
///
/// Runtime watch tokens are opaque and session-local. Retaining the full gateway identity with
/// the token prevents a replacement session from receiving an unwatch for an unrelated value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HierarchyWorldWatch {
    domain: WorldDomain,
    token: QualifiedWatchToken,
    selection_revision: Option<u64>,
    projection_pending: bool,
}

impl HierarchyWorldWatch {
    pub(super) fn new(domain: WorldDomain, token: QualifiedWatchToken) -> Self {
        Self {
            domain,
            token,
            selection_revision: None,
            projection_pending: true,
        }
    }

    pub(super) const fn domain(&self) -> WorldDomain {
        self.domain
    }

    pub(super) const fn token(&self) -> &QualifiedWatchToken {
        &self.token
    }

    pub(super) fn belongs_to(
        &self,
        domain: WorldDomain,
        identity: &GatewaySessionIdentity,
    ) -> bool {
        self.domain == domain && self.token.identity() == identity
    }

    pub(super) fn selection_revision_changed(&self, revision: u64) -> bool {
        self.selection_revision != Some(revision)
    }

    pub(super) fn mark_projection_pending(&mut self) {
        self.projection_pending = true;
    }

    pub(super) const fn projection_pending(&self) -> bool {
        self.projection_pending
    }

    pub(super) fn complete_projection(&mut self, revision: u64) {
        self.selection_revision = Some(revision);
        self.projection_pending = false;
    }
}

#[cfg(test)]
#[path = "tests/hierarchy_world_watch.rs"]
mod tests;
