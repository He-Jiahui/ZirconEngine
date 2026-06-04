use super::super::super::*;

#[test]
fn sound_plugin_registration_contributes_dynamic_event_catalog() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());
    let sound_event_catalog = report
        .extensions
        .plugin_event_catalogs()
        .iter()
        .find(|catalog| catalog.namespace == SOUND_DYNAMIC_EVENT_NAMESPACE)
        .expect("sound dynamic event catalog");

    assert_eq!(
        sound_event_catalog
            .events
            .iter()
            .map(|event| (event.id.as_str(), event.payload_schema.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("sound.dynamic_events.impact", "sound.dynamic.impact.v1"),
            ("sound.dynamic_events.marker", "sound.dynamic.marker.v1"),
            (
                "sound.dynamic_events.ambient_stinger",
                "sound.dynamic.ambient_stinger.v1",
            ),
        ]
    );
}
