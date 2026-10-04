use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};

use super::*;

#[test]
fn revoke_removes_only_the_matching_current_secure_text_lease() {
    let node = UiNodeId::new(7);
    let current = UiSecureTextValueRef::issue(UiTreeId::new("secure.revoke"), node, "value");
    let stale = UiSecureTextValueRef::issue(UiTreeId::new("secure.revoke"), node, "value");
    let mut store = UiSurfaceSecureTextValueStore::default();
    store.register(current.clone(), Some(11));

    assert!(!store.revoke(&stale));
    assert!(store.resolves(&current, Some(11)));
    assert!(store.revoke(&current));
    assert!(!store.resolves(&current, Some(11)));
    assert!(!store.revoke(&current));
}

#[test]
fn pending_model_text_is_surface_owned_and_debug_redacted() {
    let owner = UiNodeId::new(9);
    let mut store = UiSurfaceSecureTextValueStore::default();
    store.store_pending_model_update(owner, "pending-secret-model".to_string());

    let debug = format!("{store:?}");
    assert!(!debug.contains("pending-secret-model"));
    assert!(debug.contains("pending_model_update_count: 1"));
    assert_eq!(
        store.take_pending_model_update(owner).as_deref(),
        Some("pending-secret-model")
    );
    assert!(store.take_pending_model_update(owner).is_none());
}

#[test]
fn accepted_pending_model_text_moves_out_of_the_zeroizing_owner() {
    let owner = UiNodeId::new(10);
    let mut store = UiSurfaceSecureTextValueStore::default();
    store.store_pending_model_update(owner, "accepted-secret-model".to_string());

    let value = store
        .take_pending_model_update(owner)
        .expect("pending secure model value");

    assert_eq!(value, "accepted-secret-model");
    assert!(store.take_pending_model_update(owner).is_none());
}
