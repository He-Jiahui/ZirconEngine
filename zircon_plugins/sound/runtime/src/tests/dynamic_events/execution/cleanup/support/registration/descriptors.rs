use super::super::super::super::super::*;

use super::super::fixture::CleanupFixture;

pub(crate) fn register_cleanup_event_and_handler(
    sound: &DefaultSoundManager,
    fixture: &CleanupFixture,
) {
    sound.register_dynamic_event(descriptor(fixture)).unwrap();
    sound
        .register_dynamic_event_handler(handler(fixture))
        .unwrap();
}

fn descriptor(fixture: &CleanupFixture) -> SoundDynamicEventDescriptor {
    SoundDynamicEventDescriptor {
        id: fixture.event_id().to_string(),
        display_name: fixture.event_display_name().to_string(),
        payload_schema: fixture.payload_schema().to_string(),
    }
}

fn handler(fixture: &CleanupFixture) -> SoundDynamicEventHandlerDescriptor {
    SoundDynamicEventHandlerDescriptor {
        plugin_id: fixture.plugin_id().to_string(),
        handler_id: fixture.handler_id().to_string(),
        event_id: fixture.event_id().to_string(),
        display_name: fixture.handler_display_name().to_string(),
        priority: 0,
    }
}
