use super::super::super::*;

use super::support::{
    assert_next_execution_skipped_missing_executor, register_cleanup_event_and_handler,
    register_cleanup_event_handler_and_executor, submit_cleanup_invocation, CleanupFixture,
};

#[test]
fn dynamic_event_unregistering_event_removes_matching_executors() {
    let sound = DefaultSoundManager::default();
    let fixture = CleanupFixture::event_unregister();

    register_cleanup_event_handler_and_executor(&sound, &fixture);
    sound.unregister_dynamic_event(fixture.event_id()).unwrap();

    register_cleanup_event_and_handler(&sound, &fixture);
    submit_cleanup_invocation(&sound, &fixture);

    assert_next_execution_skipped_missing_executor(&sound);
}
