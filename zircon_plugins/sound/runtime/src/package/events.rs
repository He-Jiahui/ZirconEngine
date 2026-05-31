use zircon_runtime::plugin::{PluginEventCatalogManifest, PluginEventManifest};

pub const SOUND_DYNAMIC_EVENT_NAMESPACE: &str = "sound.dynamic_events";

pub fn sound_event_catalogs() -> Vec<PluginEventCatalogManifest> {
    vec![PluginEventCatalogManifest {
        namespace: SOUND_DYNAMIC_EVENT_NAMESPACE.to_string(),
        version: 1,
        events: vec![
            event(
                "sound.dynamic_events.impact",
                "Impact",
                "sound.dynamic.impact.v1",
            ),
            event(
                "sound.dynamic_events.marker",
                "Marker",
                "sound.dynamic.marker.v1",
            ),
            event(
                "sound.dynamic_events.ambient_stinger",
                "Ambient Stinger",
                "sound.dynamic.ambient_stinger.v1",
            ),
        ],
    }]
}

fn event(id: &str, display_name: &str, payload_schema: &str) -> PluginEventManifest {
    PluginEventManifest {
        id: id.to_string(),
        display_name: display_name.to_string(),
        payload_schema: payload_schema.to_string(),
    }
}
