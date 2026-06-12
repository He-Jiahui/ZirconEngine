#[test]
fn sound_dynamic_event_catalog_contributes_stable_event_ids() {
    let runtime_manifest = crate::package_manifest();
    let sound_event_catalog = runtime_manifest
        .event_catalogs
        .iter()
        .find(|catalog| catalog.namespace == crate::SOUND_DYNAMIC_EVENT_NAMESPACE)
        .expect("sound dynamic event catalog");

    assert_eq!(
        sound_event_catalog
            .events
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "sound.dynamic_events.impact",
            "sound.dynamic_events.marker",
            "sound.dynamic_events.ambient_stinger",
        ]
    );
}
