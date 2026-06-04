use super::super::super::super::super::*;

use super::super::fixture::CleanupFixture;

use super::descriptors::register_cleanup_event_and_handler;

pub(crate) fn register_cleanup_event_handler_and_executor(
    sound: &DefaultSoundManager,
    fixture: &CleanupFixture,
) {
    register_cleanup_event_and_handler(sound, fixture);
    sound
        .register_dynamic_event_executor(fixture.plugin_id(), fixture.handler_id(), |_| Ok(()))
        .unwrap();
}
