use super::super::super::*;

use super::support::register_impact_event;

#[test]
fn dynamic_event_registry_projects_catalog_into_mixer_snapshot() {
    let sound = DefaultSoundManager::default();
    register_impact_event(&sound);

    let catalog = sound.dynamic_event_catalog().unwrap();

    assert_eq!(
        sound.mixer_snapshot().unwrap().graph.dynamic_events.events,
        catalog.events
    );
}
