use super::{EventStore, EventSubscription, EventSubscriptionStatus};

#[test]
fn dormant_subscription_rejects_another_store_with_a_colliding_type_id() {
    let mut owner = EventStore::default();
    let mut subscription = EventSubscription::<u32>::new_dormant(&mut owner);
    let mut other_type = EventStore::default();
    assert_eq!(other_type.register::<u64>(), subscription.event_type_id());
    assert!(!subscription.connect(&mut other_type));
    assert_eq!(subscription.status(), EventSubscriptionStatus::Dormant);
    let mut same_type = EventStore::default();
    assert_eq!(same_type.register::<u32>(), subscription.event_type_id());
    assert!(!subscription.connect(&mut same_type));
    assert!(subscription.connect(&mut owner));
    assert!(subscription.disconnect(&mut owner));
}

#[test]
fn wrong_store_cannot_read_disconnect_or_advance_an_owner_cursor() {
    let mut owner = EventStore::default();
    let mut subscription = EventSubscription::<u32>::new_dormant(&mut owner);
    assert!(subscription.connect(&mut owner));
    assert!(owner.send(7_u32));
    owner.update::<u32>();
    let mut other = EventStore::default();
    assert!(other.send(99_u32));
    other.update::<u32>();
    assert_eq!(subscription.unread_count(&other), 0);
    assert_eq!(subscription.read(&other).next(), None);
    assert!(!subscription.disconnect(&mut other));
    assert!(subscription.is_connected());
    assert_eq!(subscription.unread_count(&owner), 1);
    assert_eq!(subscription.read(&owner).copied().collect::<Vec<_>>(), [7]);
    assert!(subscription.disconnect(&mut owner));
}
