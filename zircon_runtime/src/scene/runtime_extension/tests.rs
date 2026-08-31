use super::*;

#[test]
fn failed_merge_preserves_the_previous_plan() {
    let base = WorldRuntimeExtensionPlan::from_registrations([registration("scene.tick")])
        .expect("base plan");
    let contribution = WorldRuntimeExtensionPlan::from_registrations([registration("scene.tick")])
        .expect("standalone contribution");

    assert!(base.try_merge(contribution).is_err());
    assert_eq!(base.registration_count(), 1);
}

#[test]
fn uniqueness_validation_borrows_registration_keys() {
    let source = include_str!("plan.rs");

    assert!(source.contains(".map(WorldRuntimeExtensionRegistration::key)"));
    assert!(source.contains("drop(keys);"));
    assert!(!source.contains(concat!("registration.key().", "clone()")));
}

fn registration(key: &str) -> WorldRuntimeExtensionRegistration {
    WorldRuntimeExtensionRegistration::new(key, |_| Ok(()))
}
