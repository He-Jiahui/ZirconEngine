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
