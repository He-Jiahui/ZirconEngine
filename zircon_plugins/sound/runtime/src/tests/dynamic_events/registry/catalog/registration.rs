use super::super::super::*;

use super::support::register_impact_event;

#[test]
fn dynamic_event_registry_accepts_descriptors_into_catalog() {
    let sound = DefaultSoundManager::default();
    assert!(sound.dynamic_event_catalog().unwrap().events.is_empty());

    register_impact_event(&sound);

    let catalog = sound.dynamic_event_catalog().unwrap();
    assert_eq!(catalog.namespace, "sound.dynamic_events");
    assert_eq!(catalog.version, 1);
    assert_eq!(catalog.events.len(), 1);
    assert_eq!(catalog.events[0].id, "sound.dynamic.impact");
}
