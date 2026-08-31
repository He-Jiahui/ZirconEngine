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
mod tests {
    use super::HierarchyWorldWatch;
    use crate::core::play::WorldDomain;
    use crate::core::sync::QualifiedWatchToken;
    use zircon_runtime_interface::world_sync::WatchToken;
    use zircon_runtime_interface::{GatewaySessionIdentity, ZrRuntimeSessionHandle};

    #[test]
    fn hierarchy_watch_only_belongs_to_its_issuing_gateway_identity() {
        let identity = GatewaySessionIdentity::new(3, ZrRuntimeSessionHandle::new(5), 7, None)
            .with_gateway_generation(11);
        let watch = HierarchyWorldWatch::new(
            WorldDomain::Edit,
            QualifiedWatchToken::new(WatchToken::new(7), identity.clone()),
        );

        assert!(watch.belongs_to(WorldDomain::Edit, &identity));
        assert!(!watch.belongs_to(
            WorldDomain::Edit,
            &identity.clone().with_play_instance(Some(13))
        ));
    }

    #[test]
    fn failed_projection_remains_pending_until_an_explicit_completion() {
        let identity = GatewaySessionIdentity::new(3, ZrRuntimeSessionHandle::new(5), 7, None);
        let mut watch = HierarchyWorldWatch::new(
            WorldDomain::Edit,
            QualifiedWatchToken::new(WatchToken::new(7), identity),
        );

        assert!(watch.projection_pending());
        assert!(watch.selection_revision_changed(4));
        watch.complete_projection(4);
        assert!(!watch.projection_pending());
        assert!(!watch.selection_revision_changed(4));
        watch.mark_projection_pending();
        assert!(watch.projection_pending());
    }
}
