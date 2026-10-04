use std::sync::Arc;

use super::{PlayDomainLink, PlayDomainLinkError, WorldDomain};
use crate::core::gateway::DetachedEditorRuntimeGateway;

#[test]
fn play_world_domain_roundtrip_rejects_the_reserved_zero_identity() {
    let domain = WorldDomain::Play(super::PlayInstanceId::for_test(7));
    let encoded = serde_json::to_string(&domain).expect("play domain should serialize");
    assert_eq!(
        serde_json::from_str::<WorldDomain>(&encoded).expect("play domain should roundtrip"),
        domain
    );
    assert!(serde_json::from_str::<WorldDomain>(r#"{"kind":"play","data":0}"#).is_err());
}

#[test]
fn identity_guard_refuses_to_detach_a_replaced_play_gateway() {
    let link = PlayDomainLink::default();
    let instance = link
        .attach(Arc::new(DetachedEditorRuntimeGateway))
        .expect("the test play link should attach");
    let gateway = link
        .gateway(instance)
        .expect("the attached play gateway should remain reachable");
    let captured_identity = gateway.identity();
    gateway
        .replace_for_play(Arc::new(DetachedEditorRuntimeGateway), Some(instance.raw()))
        .expect("the test should replace the stable play gateway");

    let error = link
        .detach_matching_identity(instance, &captured_identity)
        .expect_err("a shutdown capture must not detach the replacement gateway");

    assert!(matches!(
        error,
        PlayDomainLinkError::GatewayIdentityMismatch { .. }
    ));
    assert_eq!(link.attached_domain(), Some(WorldDomain::Play(instance)));
}
